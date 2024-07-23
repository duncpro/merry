//! This module provides the facilities needed to construct an *LTree* given some source text. 
//! Summarily, an *LTree* represents the hierarchical structure of the source text logically in
//! memory. 
//! 
//! For every line in the source text there is a node in the *LTree*. Specifically, for every
//! *contentful lines*, there is a [`ast::Line`] node. And for every blank line, there is a
//! [`ast::VerticalSpace`] node.
//! 
//! However, unlike the flat source text, a document's *LTree* groups these lines together into
//! *blocks*, and arranges these *blocks* in a hierarchy according to a few simple rules....
//!
//! - Lines with the same indentation, not separated by more than one blank line, constitute a
//!   *block*.
//! - A non-blank line, not separated by more than one blank line from the preceeding
//!   *contentful line*, and with a deeper indentation, becomes a child to the preceding line.
//!   More accurately, the *block* containing the subsequent line, is a child
//!   of the *block* containing the preceeding line. 
//! - A *n*-indented line starting with a *list declarator*, begins a new *list element*
//!   (represented by [`ast::ListElement`]). 
//! - A *list element* is comprised of a complete *block*, and that *list element* is
//!   terminated when the *block* it contains is terminated.
//! - Consecutive *list elements* in the same *block* are grouped into a list
//!   (represented by [`Ast::List`]).
//!
//! Beyond this hierarchy, there is one final and important property of the *LTree*.
//! That is, recoverability. Given an unmodified *LTree*, the source text can be reproduced 
//! mostly verbatim. Not only the *content* of the document, but the original line indentations,
//! and the blank separating lines too.

pub mod ast {
    use crate::parse::SourceSpan;

    #[derive(Debug, Default)]
    pub struct Root<'a> { pub children: Vec<RootChild<'a>> }

    #[derive(Debug)]
    pub enum RootChild<'a> {
        Block(Block<'a>),
        VerticalSpace(VerticalSpace<'a>)
    }

    #[derive(Debug)]
    pub struct Block<'a> { 
        pub children: Vec<BlockChild<'a>>,
        pub indent: usize,
        pub span: SourceSpan<'a>
    }    

    #[derive(Debug)]
    pub struct VerticalSpace<'a> { pub span: SourceSpan<'a> }

    #[derive(Debug)]
    pub enum BlockChild<'a> {
        List(List<'a>),
        Line(Line<'a>),
        VerticalSpace(VerticalSpace<'a>),
        Block(Block<'a>)
    }

    #[derive(Debug, Default)]
    pub struct List<'a> {
        pub children: Vec<ListElement<'a>>
    }

    #[derive(Debug)]
    pub struct ListElement<'a> {
        pub content: Block<'a>
    }

    #[derive(Debug)]
    pub struct Line<'a> { 
        pub indent_span: SourceSpan<'a>,
        pub line_content: SourceSpan<'a> 
    }
}

use crate::parse::{ForwardCursor, SourceSpan};

/// Constructs an *LTree* from the entirety of the given source text. The module-level 
/// documentation contains an explanation of *LTree*.
pub fn make_ltree<'a>(source: &'a String) -> ast::Root<'a> {
    let mut ctx = ParseContext { cursor: ForwardCursor::new(source) };
    let mut root: ast::Root<'a> = <ast::Root as Default>::default();
    loop {
        if ctx.cursor.is_end() { break; }
        if let Some(line_span) = ctx.cursor.match_scan(blank_line()) { 
            let vspace = ast::VerticalSpace { span: line_span };
            root.children.push(ast::RootChild::VerticalSpace(vspace));
            continue; 
        }
        let indent = ctx.cursor.peek_spaces();
        let block = parse_block(&mut ctx, indent).node;
        root.children.push(ast::RootChild::Block(block));
    }
    return root;
}


// *LTree* Praser

pub struct ParseContext<'a> { cursor: ForwardCursor<'a> }

#[derive(Debug)]
enum TreeDestin { Root, Parent }

#[derive(Debug)]
struct ParseResult<N> { destin: TreeDestin, node: N }

/// Calls `$use` with the [`ParseResult`]'s `node` and then short-circuits the enclosing loop
/// either through `break` or `continue`.
///
/// Practically speaking, the loop is "broken" when a lineage terminating sequence is 
/// encountered. That is, two blank lines or EOF.
macro_rules! use_result {
    ($result:expr, $use:expr) => {
        let ParseResult { destin, node } = $result;
        $use(node);
        match destin {
            TreeDestin::Root => break TreeDestin::Root,
            TreeDestin::Parent => continue
        }
    };
}

/// Advances the cursor past the next *block* and assembles an [`ast::Block`] representing the
/// content.
/// 
/// This procedure will *never* return in the middle of a line. In other words, the caller can
/// assume that the cursor is placed at the beginning of a subsequent line (or EOF) after `parse`
/// returns.
fn parse_block<'a, 'b>(ctx: &'a mut ParseContext<'b>, indent: usize) 
-> ParseResult<ast::Block<'b>>
{
    let mut children: Vec<ast::BlockChild<'b>> = Vec::new();
    let mut consec_blank_line_count: usize = 0;
    let begin = ctx.cursor.pos();
    let destin = loop {
        if ctx.cursor.is_end() { break TreeDestin::Root; }
        if let Some(line_span) = ctx.cursor.match_scan(blank_line()) {
            consec_blank_line_count += 1;
            let vspace = ast::VerticalSpace { span: line_span };
            children.push(ast::BlockChild::VerticalSpace(vspace));
            if consec_blank_line_count == 2 { break TreeDestin::Root; }
            continue;
        }
        consec_blank_line_count = 0;
        if let Some(decl) = ctx.cursor.at_scan(nested_block_decl(indent)) {
            use_result!(parse_block(ctx, decl.end.colu_pos),
                 |child| children.push(ast::BlockChild::Block(child)));
        }
        if ctx.cursor.at_scan(list_decl(indent)).is_some() {
            use_result!(parse_list(ctx, indent), 
                |list| children.push(ast::BlockChild::List(list)));
        }
        if let Some(indent_span) = ctx.cursor.match_scan(block_continuation(indent)) {
            let line_content = ctx.cursor.pop_line();
            children.push(ast::BlockChild::Line(ast::Line { line_content, indent_span }));
            continue;
        }
        break TreeDestin::Parent;    
    };
    let end = ctx.cursor.pos();
    let block_span: SourceSpan<'b> = SourceSpan { source: ctx.cursor.source, begin, end };
    let node: ast::Block<'b> = ast::Block { children, indent, span: block_span };
    return ParseResult { destin, node };
}

/// Advances the cursor past the next *list* and assembles an [`ast::List`] to represent the
/// content.
/// 
/// This procedure will *never* return in the middle of a line. In other words, the caller can
/// assume that the cursor is placed at the beginning of a subsequent line (or EOF) after
/// `parse` returns.
fn parse_list<'a, 'b>(ctx: &'a mut ParseContext<'b>, level: usize) 
-> ParseResult<ast::List<'b>>
{
    let mut node: ast::List<'b> = <ast::List as Default>::default();
    let destin: TreeDestin = loop {
        // We can leave this match and not change to at, as long as we
        // are fine with just not showing the list declarator in the block
        // quote. We can just pad n spaces to the beginning of the block
        // for the column number n. I like that.
        if !ctx.cursor.match_scan(list_decl(level)).is_some() { break TreeDestin::Parent; }
        use_result!(parse_block(ctx, level + 3), 
            |content| node.children.push(ast::ListElement { content }));
    };
    return ParseResult {destin, node };
}


// Token Scanners

use crate::scanner;

scanner! { 
    /// Scans for an *n*-indented *list element declarator*.
    list_decl (indent: usize) |cursor| {
        cursor.pop_spaces();
        if cursor.pos().colu_pos != indent { return false }
        if !cursor.match_symbol("-- ") { return false }
        return true;
    }
}

scanner! {
    blank_line () |cursor| {
        cursor.pop_spaces();
        return cursor.match_linebreak();
    }
}

scanner! {
    nested_block_decl (indent: usize) |cursor| {
        cursor.pop_spaces();
        return cursor.pos().colu_pos > indent;
    }
}

scanner! {
    block_continuation (indent: usize) |cursor| {
        cursor.pop_spaces();
        return cursor.pos().colu_pos == indent;
    }
}

// Verification

#[derive(Clone, Copy, Debug)]
pub enum AnyLTreeWarning<'a, 'b> {
    /// This warning is raised for every *block* that is indented less than three spaces
    /// past its parent block.
    ///
    /// The three-space convention aligns child block indentation with child list
    /// indentation. Hopefully, making the document more readable. 
    InsufficientIndent(InsufficientIndentWarning<'a, 'b>),

    /// This warning is raised for every *block* that is indented more than three spaces
    /// past its parent block.
    ///
    /// The three-space convention aligns child block indentation with child list
    /// indentation. Hopefully, making the document more readable. 
    ExcessiveIndent(ExcessiveIndentWarning<'a, 'b>),

    /// This warning is raised for every *vertical space* which appears in the root.
    ///
    /// Well-formatted source text contains exactly two vertical spaces between adjacent
    /// *blocks*. This is the minimum number of vertical spaces required to terminate
    /// the preceeding block. 
    ///
    /// In an *LTree*, the lineage terminating sequence is associated with the 
    /// most deeply-nested *block*. Therefore, a [`ast::VerticalSpace`] only appears in the
    /// *root* when a sequence of blank lines longer than the terminating sequence is encountered.
    ExcessiveVerticalSpace(ExcessiveVerticalSpaceWarning<'a, 'b>),
}

#[derive(Clone, Copy, Debug)]
pub struct InsufficientIndentWarning<'a, 'b> { 
    pub expect_indent: usize,
    pub block: &'a ast::Block<'b>
}

#[derive(Clone, Copy, Debug)]
pub struct ExcessiveIndentWarning<'a, 'b> { 
    pub expect_indent: usize,
    pub block: &'a ast::Block<'b>
}

#[derive(Clone, Copy, Debug)]
pub struct ExcessiveVerticalSpaceWarning<'a, 'b> {
    pub vspace: &'a ast::VerticalSpace<'b>
}

pub fn verify_ltree<'a, 'b>(root: &'a ast::Root<'b>) -> Vec<AnyLTreeWarning<'a, 'b>> {
    let mut report: Vec<AnyLTreeWarning<'a, 'b>> = Vec::new();
    for child in &root.children {
        match child {
            ast::RootChild::Block(block) => {
                verify_block(block, &mut report, 0);
            },
            ast::RootChild::VerticalSpace(vspace) => { 
                let evs_warning = ExcessiveVerticalSpaceWarning { vspace };  
                report.push(AnyLTreeWarning::ExcessiveVerticalSpace(evs_warning));     
            },
        }
    }
    return report;
}

fn verify_block<'a, 'b>(block: &'a ast::Block<'b>, report: &mut Vec<AnyLTreeWarning<'a, 'b>>,
    expect_indent: usize) 
{
    let actual_indent = block.indent;
    if actual_indent > expect_indent {
        let ei_warning = ExcessiveIndentWarning { expect_indent, block };
        report.push(AnyLTreeWarning::ExcessiveIndent(ei_warning));
    }
    if actual_indent < expect_indent {
        let ii_warning = InsufficientIndentWarning { expect_indent, block };
        report.push(AnyLTreeWarning::InsufficientIndent(ii_warning));
    }
    for child in &block.children {
        if let ast::BlockChild::Block(child_block) = child {
            verify_block(child_block, report, block.indent + 3);
        }
        if let ast::BlockChild::List(list) = child {
            for element in &list.children {
                verify_block(&element.content, report, block.indent + 3);
            }
        }
    }
}

use crate::report::{Issue, AnnotatedSourceSection, Severity};

impl<'a, 'b> From<AnyLTreeWarning<'a, 'b>> for Issue<'b> {
    fn from(any: AnyLTreeWarning<'a, 'b>) -> Self {
        match any {
            AnyLTreeWarning::InsufficientIndent(spec) => spec.into(),
            AnyLTreeWarning::ExcessiveIndent(spec) => spec.into(),
            AnyLTreeWarning::ExcessiveVerticalSpace(spec) => spec.into(),
        }
    }
}

impl<'a, 'b> From<InsufficientIndentWarning<'a, 'b>> for Issue<'b> {
    fn from(value: InsufficientIndentWarning<'a, 'b>) -> Self {
        let mut quote = AnnotatedSourceSection::from_span(&value.block.span);
        quote.extend_up(3);
        quote.place_barrier_before(value.block.span.begin.line_pos, 0,
            value.expect_indent, "expected indent in all these columns");
        quote.limit = Some(value.block.span.begin.line_pos + 3);
        Issue {
            quote,
            title: "Too few spaces before block lines",
            subtext: "Conventionally, a block's indent is exactly three greater than its parent's.",
            severity: Severity::Warning,
        }
    }
}

impl<'a, 'b> From<ExcessiveIndentWarning<'a, 'b>> for Issue<'b> {
    fn from(value: ExcessiveIndentWarning<'a, 'b>) -> Self {
        let mut quote = AnnotatedSourceSection::from_span(&value.block.span);
        quote.extend_up(3);
        quote.place_barrier_before(value.block.span.begin.line_pos, 0,
            value.expect_indent, "expected indent in these columns alone");
        quote.limit = Some(value.block.span.begin.line_pos + 3);
        Issue {
            quote,
            title: "Too many spaces before block lines",
            subtext: "Conventionally, a block's indent is exactly three greater than its parent's.",
            severity: Severity::Warning,
        }
    }
}

impl<'a, 'b> From<ExcessiveVerticalSpaceWarning<'a, 'b>> for Issue<'b> {
    fn from(value: ExcessiveVerticalSpaceWarning<'a, 'b>) -> Self {
        todo!()
    }
}
