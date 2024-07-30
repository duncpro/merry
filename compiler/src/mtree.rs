pub mod ast {
    use crate::ttree;
    use crate::scan::SourceSpan;

    #[derive(Debug, Clone)]
    pub struct Paragraph<'a> {
        pub content: ttree::ast::Root<'a>
    }

    #[derive(Debug, Clone)]
    pub struct DirectiveInvocation<'a> {
        pub args: Vec<SourceSpan<'a>>,
        pub is_missing_end_quote: bool    
    }

    #[derive(Debug, Clone)]
    pub struct Heading<'a> {
        pub hlevel: usize,
        pub content: ttree::ast::Root<'a>
    }

    #[derive(Debug, Clone)]
    pub struct Block<'a> {
        pub children: Vec<BlockChild<'a>>
    }

    #[derive(Debug, Clone)]
    pub struct Section<'a> {
        pub heading: Heading<'a>,
        pub children: Vec<BlockChild<'a>>
    }

    #[derive(Debug, Clone)]
    pub enum BlockChild<'a> {
        Paragraph(Paragraph<'a>),
        DirectiveInvocation(DirectiveInvocation<'a>),
        Heading(Heading<'a>),
        Block(Block<'a>),
        List(List<'a>),
        Verbatim(Verbatim<'a>),
        Section(Section<'a>)
    }

    #[derive(Debug, Clone)]
    pub struct List<'a> {
        pub elements: Vec<ListElement<'a>>
    }

    #[derive(Debug, Clone)]
    pub struct ListElement<'a> {
        pub content: Block<'a>
    }

    #[derive(Debug, Clone)]
    pub struct Root<'a> {
        pub block: Block<'a>
    }

    #[derive(Debug, Clone)]
    pub struct Verbatim<'a> {
        pub trailing_qualifier: Option<ttree::ast::TrailingQualifier<'a>>,
        pub lines: Vec<SourceSpan<'a>>
    }
}

use crate::{ltree, assert_matches};
use crate::report::Issue;
use crate::scan::{SourceSpan, SourceLocation};
use crate::ttree::{self, verify_ttree, AnyTTreeIssue, make_ttree};

pub fn make_mtree<'a, 'b>(ltree: &'a ltree::ast::Root<'b>) -> ast::Root<'b> {
    let mut block = make_block(&ltree.block);
    sectionize_block(&mut block, 0);
    return ast::Root { block }
}

fn make_block<'a, 'b>(ltree_block: &'a ltree::ast::Block<'b>) -> ast::Block<'b> {
    let mut mtree_children: Vec<ast::BlockChild<'b>> = Vec::new();
    let mut paragraph_lines: Vec<SourceSpan<'b>> = Vec::new();

    macro_rules! push_paragraph { () => {
        if paragraph_lines.len() > 0 {
            let content = ttree::make_ttree(paragraph_lines.as_slice());
            let paragraph = ast::Paragraph { content };
            mtree_children.push(ast::BlockChild::Paragraph(paragraph));
            paragraph_lines.clear();
        }
    }}
    
    for ltree_child in &ltree_block.children {
        if let ltree::ast::BlockChild::Line(line) = ltree_child {
            if line.line_content.begin().at_symbol("#") {
                push_paragraph!();
                let heading = make_heading(line);
                mtree_children.push(ast::BlockChild::Heading(heading));
                continue;
            }
            if line.line_content.begin().at_symbol("|") {
                push_paragraph!();
                let node = parse_directive_invocation(line);
                mtree_children.push(ast::BlockChild::DirectiveInvocation(node));
                continue;
            }
            paragraph_lines.push(line.line_content);
            continue;
        }
        push_paragraph!();
        if let ltree::ast::BlockChild::Block(ltree_child_block) = ltree_child {
            let mtree_child_block = make_block(ltree_child_block);
            mtree_children.push(ast::BlockChild::Block(mtree_child_block));
            continue;
        }
        if let ltree::ast::BlockChild::Verbatim(ltree_verbatim) = ltree_child {
            let mtree_verbatim = make_verbatim(ltree_verbatim);
            mtree_children.push(ast::BlockChild::Verbatim(mtree_verbatim));
            continue;
        }
        if let ltree::ast::BlockChild::List(ltree_list) = ltree_child {
            let mtree_list = make_list(ltree_list);
            mtree_children.push(ast::BlockChild::List(mtree_list));
            continue;
        }
    }
    push_paragraph!();
    return ast::Block { children: mtree_children }
}

fn make_list<'a, 'b>(ltree_list: &'a ltree::ast::List<'b>) -> ast::List<'b> {
    let mut elements: Vec<ast::ListElement<'b>> = Vec::new();
    for list_child in &ltree_list.children {
        let content = make_block(&list_child.content);
        elements.push(ast::ListElement { content });
    }
    return ast::List { elements };   
}

fn make_heading<'a, 'b>(line: &'a ltree::ast::Line<'b>) -> ast::Heading<'b> {
    let mut cursor = line.line_content.begin();
    let hlevel = cursor.repeat_match_symbol("#");
    cursor.match_symbol(" ");
    let tail = cursor.pop_line();
    let content = make_ttree(&[tail]);
    return ast::Heading { hlevel, content };
}

fn make_verbatim<'a, 'b>(ltree_verbatim: &'a ltree::ast::Verbatim<'b>) -> ast::Verbatim<'b>
{
    let mut trailing_qualifier: Option<ttree::ast::TrailingQualifier<'b>> = None;
    if let Some(tail) = ltree_verbatim.tail {
        trailing_qualifier = ttree::parse_misc_trailing_qualifier(&[tail]);
    }
    let lines: Vec<SourceSpan<'b>> = ltree_verbatim.lines.clone();
    return ast::Verbatim { lines, trailing_qualifier };
}

#[allow(unused_assignments)]
fn parse_directive_invocation<'a, 'b>(line: &'a ltree::ast::Line<'b>) 
-> ast::DirectiveInvocation<'b>
{
    let mut cursor = line.line_content.begin();
    assert!(cursor.match_symbol("|").is_some());
    let mut args: Vec<SourceSpan<'b>> = Vec::new();
    let mut current_arg_begin: Option<SourceLocation> = None;
    let mut is_quoted: bool = false;

    macro_rules! push_arg { () => {
        if let Some(begin) = current_arg_begin {
            args.push(SourceSpan { source: cursor.source, begin, end: cursor.pos() });
            current_arg_begin = None;
        }
        is_quoted = false;
    }}
    
    loop {
        if cursor.is_end() { break; }
        if is_quoted && cursor.at_symbol("\"") { 
            push_arg!(); 
            assert!(cursor.match_symbol("\"").is_some());
            continue; 
        }
        if !is_quoted && cursor.at_symbol("\"") {
            push_arg!();
            assert!(cursor.match_symbol("\"").is_some());
            current_arg_begin = Some(cursor.pos());
            is_quoted = true;
            continue;
        }
        if cursor.at_symbol(" ") {
            if !is_quoted { push_arg!(); }
            assert!(cursor.match_symbol(" ").is_some());
            continue;
        }
        if current_arg_begin.is_none() { current_arg_begin = Some(cursor.pos()); }
        cursor.pop_grapheme().unwrap();
    }

    let is_missing_end_quote = is_quoted;
    push_arg!();
    return ast::DirectiveInvocation { args, is_missing_end_quote };
}

fn sectionize_block<'a, 'b>(block: &'a mut ast::Block<'b>, hlevel_lb: usize) {
    let mut i: usize = 0;
    while i < block.children.len() {
        if let ast::BlockChild::Heading(next_heading) = &block.children[i] {
            if next_heading.hlevel > hlevel_lb { sectionize(block, i); }
            // Otherwise,
            // This is an implicit section break, however we cannot break out
            // to this extent as the section is an ancestor to the current block. 
            // In other words, we cannot break to that section without breaking this block. 
            // Put symbolically, <section><block></section></block> is illegal.
            // We cannot intersperse elements in this way. So, we skip sectionizing
            // and just interpret this as a strangely placed header.
        }
        i += 1;
    }
}

fn sectionize<'a, 'b>(block: &'a mut ast::Block<'b>, pos: usize) {
    assert_matches!(block.children[pos], ast::BlockChild::Heading(ref heading_ref));
    let section_heading = heading_ref.clone();
    let mut children: Vec<ast::BlockChild<'b>> = Vec::new();

    let i: usize = pos + 1;
    while i < block.children.len() {
        if let ast::BlockChild::Heading(next_heading) = &block.children[i] {
            if next_heading.hlevel <= section_heading.hlevel { break; }
            sectionize(block, i);
        }
        if let ast::BlockChild::Block(next_block) = &mut block.children[i] {
            sectionize_block(next_block, section_heading.hlevel);
        }
        if let ast::BlockChild::List(next_list) = &mut block.children[i] {
            for element in &mut next_list.elements {
                sectionize_block(&mut element.content, section_heading.hlevel);
            }
        }
        children.push(block.children.remove(i));
    }

    block.children[pos] = ast::BlockChild::Section(ast::Section { 
        heading: section_heading, children });
}

pub enum AnyMTreeIssue<'a, 'b> {
    AnyTTreeIssue(AnyTTreeIssue<'a, 'b>)
}

impl<'a, 'b> From<AnyMTreeIssue<'a, 'b>> for Issue<'b> {
    fn from(value: AnyMTreeIssue<'a, 'b>) -> Self {
        match value {
            AnyMTreeIssue::AnyTTreeIssue(spec) => spec.into(),
        }
    }
}

pub fn verify_mtree<'a, 'b>(root: &'a ast::Root<'b>) -> Vec<AnyMTreeIssue<'a, 'b>> {
    let mut issues: Vec<AnyMTreeIssue<'a, 'b>> = Vec::new();
    verify_block(&root.block, &mut issues);
    return issues;
}

pub fn verify_block<'a, 'b>(block: &'a ast::Block<'b>, issues: &mut Vec<AnyMTreeIssue<'a, 'b>>) {
    for child in &block.children {
        if let ast::BlockChild::Block(child_block) = child {
            verify_block(child_block, issues);
            continue;
        }
        if let ast::BlockChild::Paragraph(paragraph) = child {
            verify_paragraph(paragraph, issues);
            continue;
        }
        if let ast::BlockChild::Verbatim(verbatim) = child {
            verify_verbatim(verbatim, issues);
            continue;
        }
    }
}

pub fn verify_paragraph<'a, 'b>(paragraph: &'a ast::Paragraph<'b>,
    issues: &mut Vec<AnyMTreeIssue<'a, 'b>>) 
{
    let ttree_issues = verify_ttree(&paragraph.content);
    for issue in ttree_issues { 
        issues.push(AnyMTreeIssue::AnyTTreeIssue(issue));
    }
}

pub fn verify_verbatim<'a, 'b>(verbatim: &'a ast::Verbatim<'b>, 
    issues: &mut Vec<AnyMTreeIssue<'a, 'b>>)
{
    // TODO: RemoveAllocation
    let mut ttree_issues: Vec<AnyTTreeIssue<'a, 'b>> = Vec::new();
    ttree::verify_trailing_qualifier(&verbatim.trailing_qualifier, &mut ttree_issues);
    for ttree_issue in ttree_issues {
        issues.push(AnyMTreeIssue::AnyTTreeIssue(ttree_issue));
    }
}
