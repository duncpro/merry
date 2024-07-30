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
//! - Lines with the same indentation, not seperated by more than one blank line, constitute a
//!   *block*.
//! - A non-blank line, not seperated by more than one blank line from the preceeding
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
    use crate::scan::SourceSpan;

    #[derive(Debug)]
    pub struct Root<'a> { pub block: Block<'a> }

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
        Block(Block<'a>),
        Verbatim(Verbatim<'a>)
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

    #[derive(Debug)]
    pub struct Verbatim<'a> {
        pub lines: Vec<SourceSpan<'a>>,
        pub close: Option<SourceSpan<'a>>,
        pub span: SourceSpan<'a>,
        pub indent: usize,
        /// The tail of a Verbatim is the remainder left of the last line after
        /// the closing declarator is consumed.
        pub tail: Option<SourceSpan<'a>>,
        pub open: SourceSpan<'a>
    }
}

use crate::scan::{ForwardCursor, SourceSpan};
use crate::scanner;

/// Constructs an *LTree* from the entirety of the given source text. The module-level 
/// documentation contains an explanation of *LTree*.
pub fn make_ltree<'a>(source: &'a String) -> ast::Root<'a> {
    let mut ctx = ParseContext { cursor: ForwardCursor::new(source) };
    let block = parse_block(&mut ctx, 0, 0).node;
    return ast::Root { block };
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
fn parse_block<'a, 'b>(ctx: &'a mut ParseContext<'b>, indent: usize, depth: usize) 
-> ParseResult<ast::Block<'b>>
{
    let mut children: Vec<ast::BlockChild<'b>> = Vec::new();
    let begin = ctx.cursor.pos();
    let destin = loop {
        let mut consec_blank_line_count: usize = 0;
        while let Some(line_span) = ctx.cursor.match_scan(blank_line()) {
            consec_blank_line_count += 1;
            let vspace = ast::VerticalSpace { span: line_span };
            children.push(ast::BlockChild::VerticalSpace(vspace));
        }
        if depth > 0 {
            if consec_blank_line_count > 2 { break TreeDestin::Root };
            if consec_blank_line_count == 2 { break TreeDestin::Parent; }
        }
        if ctx.cursor.is_end() { break TreeDestin::Root; }
        if let Some(decl) = ctx.cursor.at_scan(nested_block_decl(indent)) {
            use_result!(parse_block(ctx, decl.end.colu_pos, depth + 1),
                 |child| children.push(ast::BlockChild::Block(child)));
        }
        if ctx.cursor.at_scan(list_decl(indent)).is_some() {
            use_result!(parse_list(ctx, indent, depth), 
                |list| children.push(ast::BlockChild::List(list)));
        }
        if ctx.cursor.at_scan(verbatim_decl(indent)).is_some() {
            children.push(ast::BlockChild::Verbatim(parse_verbatim(ctx, indent)));
            continue;
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
fn parse_list<'a, 'b>(ctx: &'a mut ParseContext<'b>, indent: usize, depth: usize) 
-> ParseResult<ast::List<'b>>
{
    let mut node: ast::List<'b> = <ast::List as Default>::default();
    let destin: TreeDestin = loop {
        if !ctx.cursor.match_scan(list_decl(indent)).is_some() { break TreeDestin::Parent; }
        use_result!(parse_block(ctx, indent + 3, depth + 1), 
            |content| node.children.push(ast::ListElement { content }));
    };
    return ParseResult {destin, node };
}

fn parse_verbatim<'a, 'b>(ctx: &'a mut ParseContext<'b>, indent: usize) -> ast::Verbatim<'b> {
    assert_eq!(ctx.cursor.pop_spaces().end.colu_pos, indent);
    let begin = ctx.cursor.pos();
    let open = ctx.cursor.match_scan(verbatim_open()).unwrap();
    let start_backtick_count: usize = open.as_ref().len();
    assert!(ctx.cursor.match_linebreak());
    let mut lines: Vec<SourceSpan<'b>> = Vec::new();
    let mut close: Option<SourceSpan<'b>> = None;
    let mut tail: Option<SourceSpan<'b>> = None;
    loop {
        let mut actual_indent: usize = 0;
        loop {
            // Whitespace beyond the expected indent is interpreted verbatim.
            if actual_indent == indent { break; }
            // The the indent ends prematurely, we'll begin the verbatim early,
            // however this will be reported as an issue during the verification step.
            if ctx.cursor.match_symbol(" ").is_none() { break; }
            actual_indent += 1;
        }
        if ctx.cursor.is_end() { break; }
        if let Some(span) = ctx.cursor.match_scan(verbatim_end(start_backtick_count)) {
            close = Some(span);
            tail = Some(ctx.cursor.pop_line());
            break;
        }
        lines.push(ctx.cursor.pop_line());
    }
    let end = ctx.cursor.pos();
    let span = SourceSpan { source: ctx.cursor.source, begin, end };
    return ast::Verbatim { span, lines, close, indent, tail, open };
}

// Token Scanners

scanner! { 
    list_decl (indent: usize) |cursor| {
        cursor.pop_spaces();
        if cursor.pos().colu_pos != indent { return false }
        cursor.match_symbol("-- ").is_some()
    }
}

scanner! {
    blank_line () |cursor| {
        cursor.pop_spaces();
        cursor.match_linebreak()
    }
}

scanner! {
    nested_block_decl (indent: usize) |cursor| {
        cursor.pop_spaces();
        cursor.pos().colu_pos > indent
    }
}

scanner! {
    block_continuation (indent: usize) |cursor| {
        cursor.pop_spaces();
        cursor.pos().colu_pos == indent
    }
}

scanner! {
    verbatim_decl (indent: usize) |cursor| {
        cursor.pop_spaces();
        if cursor.pos().colu_pos != indent { return false }
        if cursor.repeat_match_symbol("`") < 1 { return false; }
        cursor.match_linebreak()
    }
}

scanner! {
    verbatim_end (expect_backtick_count: usize) |cursor| {
        let actual_backtick_count = cursor.repeat_match_symbol("`");
        return actual_backtick_count >= expect_backtick_count;
    }
}

scanner! {
    verbatim_open() |cursor| {
        let backtick_count = cursor.repeat_match_symbol("`");
        return backtick_count > 0;
    }
}




// Verification

#[derive(Clone, Copy, Debug)]
pub enum AnyLTreeIssue<'a, 'b> {
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

    /// This warning is raised for every sequence of redundant *vertical spaces*.
    /// That is, a sequence of vertical spaces which would have identical interpretation
    /// if it were shorter.
    ExcessiveVerticalSpace(ExcessiveVerticalSpaceWarning<'b>),

    /// This warning is raised for every child block not seperated from its previous
    /// sibling by a vertical space. 
    AbruptChildBlock(AbruptChildBlockWarning<'a, 'b>),

    UnclosedVerbatim(UnclosedVerbatimError<'a, 'b>),

    VerbatimUnderindented(VerbatimUnderindentedWarning<'a, 'b>),

    LongVerbatimCloseWarning(LongVerbatimCloseWarning<'a, 'b>)
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
pub struct ExcessiveVerticalSpaceWarning<'a> {
    pub span: SourceSpan<'a>,
    pub limit: usize
}

#[derive(Clone, Copy, Debug)]
pub struct AbruptChildBlockWarning<'a, 'b> {
    pub child_block: &'a ast::Block<'b>
}

#[derive(Clone, Copy, Debug)]
pub struct UnclosedVerbatimError<'a, 'b> {
    verbatim: &'a ast::Verbatim<'b>
}

#[derive(Clone, Copy, Debug)]
pub struct VerbatimUnderindentedWarning<'a, 'b> {
    verbatim: &'a ast::Verbatim<'b>
}

#[derive(Clone, Copy, Debug)]
pub struct LongVerbatimCloseWarning<'a, 'b> { 
    node: &'a ast::Verbatim<'b>,
    close: SourceSpan<'b>
}

pub fn verify_ltree<'a, 'b>(root: &'a ast::Root<'b>) -> Vec<AnyLTreeIssue<'a, 'b>> {
    let mut report: Vec<AnyLTreeIssue<'a, 'b>> = Vec::new();
    verify_seperation(&root.block, &mut report, false);
    verify_block(&root.block, &mut report, 0);  
    return report;
}

fn verify_block<'a, 'b>(block: &'a ast::Block<'b>, report: &mut Vec<AnyLTreeIssue<'a, 'b>>,
    expect_indent: usize) 
{
    verify_block_indent(block, report, expect_indent);

    let mut prev_child: Option<&'a ast::BlockChild<'b>> = None;
    for child in &block.children {
        if let ast::BlockChild::Block(child_block) = child {
            verify_block(child_block, report, block.indent + 3);
            let mut allow_double_break = false;
            if let Some(ast::BlockChild::Block(prev_block)) = prev_child {
                allow_double_break = 
                    tail_block(prev_block).indent == child_block.indent;
            }
            verify_seperation(child_block, report, allow_double_break);
        }
        if let ast::BlockChild::List(list) = child {
            for element in &list.children {
                verify_block(&element.content, report, block.indent + 3);
            }
        }
        if let ast::BlockChild::Verbatim(verbatim) = child {
            verify_verbatim(verbatim, report);
        }
        prev_child = Some(child);
    }
    
    for (i, next) in block.children.iter().skip(1).enumerate() {
        let ast::BlockChild::Block(ref next_block) = next else { continue };
        if matches!(tail(&block.children[i]), ast::BlockChild::VerticalSpace(_)) { continue; }
        let acb_warning = AbruptChildBlockWarning { child_block: next_block };
        report.push(AnyLTreeIssue::AbruptChildBlock(acb_warning));
    }
}

fn verify_seperation<'a, 'b>(block: &'a ast::Block<'b>, report: &mut Vec<AnyLTreeIssue<'a, 'b>>,
    allow_double_break: bool) 
{
    let mut vspace_bounds: Option<(&ast::VerticalSpace, &ast::VerticalSpace)> = None;
    macro_rules! push_vspace_error { ($tail_call:expr) => {
        if let Some((first, last)) = vspace_bounds.take() {
            let limit = if $tail_call && allow_double_break { 2 } else { 1 };
            if last.span.end.line_pos - first.span.begin.line_pos > limit {
                let span = SourceSpan { source: first.span.source,
                    begin: first.span.begin, end: last.span.begin };
                let evs_warning = ExcessiveVerticalSpaceWarning { span, limit };
                report.push(AnyLTreeIssue::ExcessiveVerticalSpace(evs_warning));
            }
         }
    }}
    for child in &block.children {
        if let ast::BlockChild::VerticalSpace(vspace) = child {
            match vspace_bounds {
                Some((_, ref mut end)) => *end = vspace,
                None => vspace_bounds = Some((vspace, vspace)),
            }
            continue;
        } 
        push_vspace_error!(/* tail_call = */ false);
     } 
     push_vspace_error!(/* tail_call = */ true);
 } 

fn verify_block_indent<'a, 'b>(block: &'a ast::Block<'b>, report: &mut Vec<AnyLTreeIssue<'a, 'b>>,
    expect_indent: usize) 
{
    let actual_indent = block.indent;
    if actual_indent > expect_indent {
        let ei_warning = ExcessiveIndentWarning { expect_indent, block };
        report.push(AnyLTreeIssue::ExcessiveIndent(ei_warning));
    }
    if actual_indent < expect_indent {
        let ii_warning = InsufficientIndentWarning { expect_indent, block };
        report.push(AnyLTreeIssue::InsufficientIndent(ii_warning));
    }
}

fn verify_verbatim<'a, 'b>(verbatim: &'a ast::Verbatim<'b>, 
    report: &mut Vec<AnyLTreeIssue<'a, 'b>>) 
{
    let mut inconsistent_indent = false;
    for line in &verbatim.lines {
        if line.begin.colu_pos == verbatim.indent { continue };
        inconsistent_indent = true;
        break;
    }
    if let Some(end_span) = verbatim.close {
        if end_span.begin.colu_pos != verbatim.indent {
            inconsistent_indent = true;
        }
    }
    if inconsistent_indent {
        let warning = VerbatimUnderindentedWarning { verbatim };
        report.push(AnyLTreeIssue::VerbatimUnderindented(warning));
    }
    if verbatim.close.is_none() {
        let warning = UnclosedVerbatimError { verbatim };
        report.push(AnyLTreeIssue::UnclosedVerbatim(warning));
    }
    if let Some(close) = verbatim.close {
        if close.as_ref().len() > verbatim.open.as_ref().len() {
            let warning = LongVerbatimCloseWarning { close, node: verbatim };
            report.push(AnyLTreeIssue::LongVerbatimCloseWarning(warning));
        }
    }
}

use crate::report::{Issue, AnnotatedSourceSection, Severity, BarrierStyle};

impl<'a, 'b> From<AnyLTreeIssue<'a, 'b>> for Issue<'b> {
    fn from(any: AnyLTreeIssue<'a, 'b>) -> Self {
        match any {
            AnyLTreeIssue::InsufficientIndent(spec) => spec.into(),
            AnyLTreeIssue::ExcessiveIndent(spec) => spec.into(),
            AnyLTreeIssue::ExcessiveVerticalSpace(spec) => spec.into(),
            AnyLTreeIssue::AbruptChildBlock(spec) => spec.into(),
            AnyLTreeIssue::UnclosedVerbatim(spec) => spec.into(),
            AnyLTreeIssue::VerbatimUnderindented(spec) => spec.into(),
            AnyLTreeIssue::LongVerbatimCloseWarning(spec) => spec.into(),
        }
    }
}

impl<'a, 'b> From<InsufficientIndentWarning<'a, 'b>> for Issue<'b> {
    fn from(value: InsufficientIndentWarning<'a, 'b>) -> Self {
        let mut quote = AnnotatedSourceSection::from_span(&value.block.span);
        quote.extend_up(3);
        quote.place_barrier_before(value.block.span.begin.line_pos, 
            BarrierStyle::Ruler(0, value.expect_indent),
            "expected indent in all these columns");
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
        quote.place_barrier_before(value.block.span.begin.line_pos, 
            BarrierStyle::Ruler(0, value.expect_indent), 
            "expected indent in these columns alone");
        quote.limit = Some(value.block.span.begin.line_pos + 3);
        Issue {
            quote,
            title: "Too many spaces before block lines",
            subtext: "Conventionally, a block's indent is exactly three greater than its parent's.",
            severity: Severity::Warning,
        }
    }
}

impl<'a> From<ExcessiveVerticalSpaceWarning<'a>> for Issue<'a> {
    fn from(value: ExcessiveVerticalSpaceWarning<'a>) -> Self {
        let mut quote = AnnotatedSourceSection::from_span(&value.span);
        quote.extend_up(1);
        quote.extend_down();
        quote.place_barrier_before(value.span.begin.line_pos + value.limit, 
            BarrierStyle::Placeholder, "blank lines could've ended here");
        Issue {
            quote,
            title: "Too many blank separator lines",
            subtext: "A lesser number of linebreaks has equivalent interpretation.",
            severity: Severity::Warning
        }
    }
}

impl<'a, 'b> From<AbruptChildBlockWarning<'a, 'b>> for Issue<'b> {
    fn from(value: AbruptChildBlockWarning<'a, 'b>) -> Self {
        let first_line = value.child_block.span.begin.line_pos;
        let mut quote = AnnotatedSourceSection::from_span(&value.child_block.span);
        quote.extend_up(1);
        quote.limit = Some(first_line);
        quote.place_barrier_before(first_line, BarrierStyle::Placeholder,
            "expected blank separator line here");
        Issue {
            quote,
            title: "Missing blank separator line",
            subtext: "Conventionally, a child block is seperated from its parent by a blank line.",
            severity: Severity::Warning,
        }
    }
}

impl<'a, 'b> From<VerbatimUnderindentedWarning<'a, 'b>> for Issue<'b> {
    fn from(value: VerbatimUnderindentedWarning<'a, 'b>) -> Self {
        let mut quote = AnnotatedSourceSection::from_span(&value.verbatim.span);
        quote.place_barrier_before(value.verbatim.span.begin.line_pos, 
            BarrierStyle::Ruler(0, value.verbatim.indent), 
            "expected indent in these columns");
        Issue {
            quote,
            title: "Backtick block contains under-indented lines",
            subtext: "The lines in a backtick block should begin at the same level as the declarator.",
            severity: Severity::Warning,
        }
    }
}

impl<'a, 'b> From<UnclosedVerbatimError<'a, 'b>> for Issue<'b> {
    fn from(value: UnclosedVerbatimError<'a, 'b>) -> Self {
        let mut quote = AnnotatedSourceSection::from_span(&value.verbatim.span);
        quote.limit = Some(value.verbatim.span.begin.line_pos + 1);
         Issue {
            quote,
            title: "Backtick block is never closed",
            subtext: "This backtick block should end with a closing declarator matching the \
                      opening declarator.",
            severity: Severity::Error,
        }
    }
}

impl<'a, 'b> From<LongVerbatimCloseWarning<'a, 'b>> for Issue<'b> {
    fn from(value: LongVerbatimCloseWarning<'a, 'b>) -> Self {
        let mut quote = AnnotatedSourceSection::from_span(&value.node.span);
        quote.highlight(value.node.open.begin.byte_pos, value.node.open.end.byte_pos);
        quote.highlight(value.close.begin.byte_pos, value.close.end.byte_pos);
        Issue { 
            quote,
            title: "Too many closing backticks",
            subtext: "The number of closing backticks should equal the number of opening backticks.",
            severity: Severity::Warning
        }
    }
}


// Utilities

fn tail<'a, 'b>(root: &'a ast::BlockChild<'b>) -> &'a ast::BlockChild<'b> {
    if let ast::BlockChild::Block(block) = root {
        if let Some(child) = block.children.last() {
            return tail(child);
        }
    } 
    if let ast::BlockChild::List(list) = root {
        if let Some(child) = list.children.last() {
            if let Some(grandchild) = child.content.children.last() {
               return tail(grandchild);
            } 
        }
    } 
    return root;
}

fn tail_block<'a, 'b>(root: &'a ast::Block<'b>) -> &'a ast::Block<'b> {
    let Some(last) = root.children.last() else { return root; };
    if let ast::BlockChild::Block(child_block) = last {
        return tail_block(child_block);
    }
    if let ast::BlockChild::List(list) = last {
        if let Some(child_element) = list.children.last() {
            return tail_block(&child_element.content);
        }
    } 
    return root;
}
