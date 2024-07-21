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
        VerticalSpace(VerticalSpace)
    }

    #[derive(Debug, Default)]
    pub struct Block<'a> { pub children: Vec<BlockChild<'a>> }    

    #[derive(Debug)]
    pub struct VerticalSpace;

    #[derive(Debug)]
    pub enum BlockChild<'a> {
        List(List<'a>),
        Line(Line<'a>),
        VerticalSpace(VerticalSpace),
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
    pub struct Line<'a> { pub line_content: SourceSpan<'a> }
}

use crate::{parse::ForwardCursor, scanner};

/// Constructs an *LTree* from the entirety of the given source text. The module-level 
/// documentation contains an explanation of *LTree*.
pub fn make_ltree<'a>(source: &'a String) -> ast::Root<'a> {
    let mut ctx = ParseContext { cursor: ForwardCursor::new(source) };
    let mut root: ast::Root<'a> = <ast::Root as Default>::default();
    loop {
        if ctx.cursor.is_end() { break; }
        if ctx.cursor.match_scan(blank_line()) { continue; }
        let block = parse_block(&mut ctx, 0).node;
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

/// Advances the cursor past the next *block* and assembles an [`ast::Block`] reprsenting the
/// content.
/// 
/// This procedure will *never* return in the middle of a line. In other words, the caller can
/// assume that the cursor is placed at the beginning of a subsequent line (or EOF) after `parse`
/// returns.
fn parse_block<'a, 'b>(ctx: &'a mut ParseContext<'b>, indent: usize) 
-> ParseResult<ast::Block<'b>>
{
    let mut node: ast::Block<'b> = <ast::Block as Default>::default();
    let mut consec_blank_line_count: usize = 0;
    let destin: TreeDestin = loop {
        if ctx.cursor.is_end() { break TreeDestin::Root; }
        if ctx.cursor.match_scan(blank_line()) {
            consec_blank_line_count += 1;
            node.children.push(ast::BlockChild::VerticalSpace(ast::VerticalSpace));
            if consec_blank_line_count == 2 { break TreeDestin::Root; }
            continue;
        }
        consec_blank_line_count = 0;
        if ctx.cursor.match_scan(nested_block_decl(indent)) {
            use_result!(parse_block(ctx, ctx.cursor.pos().colu_pos),
                 |child| node.children.push(ast::BlockChild::Block(child)));
        }
        if !ctx.cursor.match_scan(block_continuation(indent)) {
            break TreeDestin::Parent;    
        }
        if ctx.cursor.at_str("-- ") { 
            use_result!(parse_list(ctx, indent), 
                |list| node.children.push(ast::BlockChild::List(list)));
        }
        let line_content = ctx.cursor.pop_line();
        node.children.push(ast::BlockChild::Line(ast::Line { line_content }));
    };
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
        if !ctx.cursor.match_scan(list_decl(level)) { break TreeDestin::Parent; }
        use_result!(parse_block(ctx, level + 3), 
            |content| node.children.push(ast::ListElement { content }));
    };
    return ParseResult {destin, node };
}


// Token Scanners

scanner! { 
    list_decl (indent: usize) |cursor| {
        cursor.pop_while(|ch| ch == ' ');
        if cursor.pos().colu_pos != indent { return false }
        if !cursor.match_str("-- ") { return false }
        return true;
    }
}

scanner! {
    blank_line () |cursor| {
        cursor.pop_while(|next_char| next_char == ' ');
        return cursor.match_char('\n')
    }
}

scanner! {
    nested_block_decl (indent: usize) |cursor| {
        cursor.pop_while(|ch| ch == ' ');
        let end = cursor.pos().colu_pos;
        return end > indent;
    }
}

scanner! {
    block_continuation (indent: usize) |cursor| {
        cursor.pop_while(|ch| ch == ' ');
        let end = cursor.pos().colu_pos;
        return end == indent;
    }
}

// Verification

pub enum LTreeWarning {
    /// This warning is raised for any *block* with an indent less than three times
    /// its nested depth.
    ///
    /// Well-formatted source text uses exactly three spaces per level.
    BreaksIndentConvention,

    /// This warning is raised for any *block* with an indent greater than three times
    /// its nested depth.
    ///
    /// Well-formatted source text uses exactly three spaces per level.
    ExcessiveIndent,

    /// This warning is raised for every *vertical space* which appears in the root.
    ///
    /// Well-formatted source text contains exactly two vertical spaces between adjacent
    /// *blocks*. This is the minimum number of vertical spaces required to terminate
    /// the preceeding block. 
    ///
    /// In an *LTree*, the lineage terminating sequence is associated with the 
    /// most deeply-nested *block*. Therefore, a [`ast::VerticalSpace`] only appears in the
    /// *root* when a sequence of blank lines longer than the terminating sequence is encountered.
    ExcessiveVerticalSpace,
}

pub fn verify_ltree(root: ast::Root) {
}
