pub mod ast {
    use crate::ltree;
    use crate::ttree;
    use crate::scan::SourceSpan;

    #[derive(Debug)]
    pub struct Paragraph<'a> {
        pub content: ttree::ast::Root<'a>
    }

    #[derive(Debug)]
    pub struct DirectiveInvocation<'a> {
        pub line: ltree::ast::Line<'a>
    }

    #[derive(Debug)]
    pub struct Heading<'a> {
        pub hlevel: usize,
        pub content: ttree::ast::Root<'a>
    }

    #[derive(Debug)]
    pub struct Block<'a> {
        pub children: Vec<BlockChild<'a>>
    }

    #[derive(Debug)]
    pub enum BlockChild<'a> {
        Paragraph(Paragraph<'a>),
        DirectiveInvocation(DirectiveInvocation<'a>),
        Heading(Heading<'a>),
        Block(Block<'a>),
        List(List<'a>),
        Verbatim(Verbatim<'a>)
    }

    #[derive(Debug)]
    pub struct List<'a> {
        pub elements: Vec<ListElement<'a>>
    }

    #[derive(Debug)]
    pub struct ListElement<'a> {
        pub content: Block<'a>
    }

    #[derive(Debug)]
    pub struct Root<'a> {
        pub block: Block<'a>
    }

    #[derive(Debug)]
    pub struct Verbatim<'a> {
        pub trailing_qualifier: Option<ttree::ast::TrailingQualifier<'a>>,
        pub lines: Vec<SourceSpan<'a>>
    }
}

use crate::ltree;
use crate::report::Issue;
use crate::scan::SourceSpan;
use crate::ttree::{self, verify_ttree, AnyTTreeIssue, make_ttree};

pub fn make_mtree<'a, 'b>(ltree: &'a ltree::ast::Root<'b>) -> ast::Root<'b> {
    let block = make_block(&ltree.block);
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
    let hlevel = cursor.repeat_match_symbol("#") - 1;
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
