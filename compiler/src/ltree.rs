//! This module provides the facilities needed to construct an *LTree* given
//! some source text. An *LTree* represents the hierarchical structure of the
//! source document logically in memory.
//!
//! Constructing an *LTree* is the first-step in parsing an md2 document.

pub mod ast {
    use crate::parse::SourceSpan;

    #[derive(Debug, Default)]
    pub struct Root<'a> { pub children: Vec<Block<'a>> }

    #[derive(Debug, Default)]
    pub struct Block<'a> { pub children: Vec<Node<'a>> }

    #[derive(Debug)]
    pub enum Node<'a> {
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

    #[derive(Debug)]
    pub struct VerticalSpace;
}

use crate::{parse::ForwardCursor, scanner};

/// Constructs an *LTree* from the entirety of the given source text.
/// The module-level documentation explains *LTrees*.
pub fn make_ltree<'a>(source: &'a String) -> ast::Root<'a> {
    let mut ctx = ParseContext { cursor: ForwardCursor::new(source) };
    let mut root: ast::Root<'a> = <ast::Root as Default>::default();
    while !ctx.cursor.is_end() {
        let block = parse_block(&mut ctx, 0).node;
        root.children.push(block);
    }
    return root;
}


// *LTree* Praser

pub struct ParseContext<'a> { cursor: ForwardCursor<'a> }

#[derive(Debug)]
enum TreeDestin { Root, Parent }

#[derive(Debug)]
struct ParseResult<N> { destin: TreeDestin, node: N }

/// Calls `$use` with the [`ParseResult`]'s `node` and then short-circuits the
/// enclosing loop either through `break` or `continue`.
///
/// Practically speaking, the loop is "broken" when a lineage terminating
/// sequence is encountered. That is, two blank lines or EOF.
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

/// Advances the [`Cursor`] past the next *Block* and assembles an [`ast::Block`]
/// reprsenting the content.
/// 
/// This procedure will *never* return in the middle of a line.
/// In other words, the caller can assume that the [`Cursor`] is placed
/// at the beginning of a subsequent line (or EOF) after `parse` returns.
fn parse_block<'a, 'b>(ctx: &'a mut ParseContext<'b>, indent: usize) 
-> ParseResult<ast::Block<'b>>
{
    let mut node: ast::Block<'b> = <ast::Block as Default>::default();
    let mut consec_blank_line_count: usize = 0;
    let destin: TreeDestin = loop {
        if ctx.cursor.match_scan(blank_line()) {
            consec_blank_line_count += 1;
            if consec_blank_line_count == 2 { break TreeDestin::Root; }
            node.children.push(ast::Node::VerticalSpace(ast::VerticalSpace));
            continue;
        }
        consec_blank_line_count = 0;
        if ctx.cursor.match_scan(nested_block_decl(indent)) {
            if matches!(node.children.last(), Some(ast::Node::VerticalSpace(_))) {
                node.children.pop();
            }
            use_result!(parse_block(ctx, ctx.cursor.pos().colu_pos),
                 |child| node.children.push(ast::Node::Block(child)));
        }
        if !ctx.cursor.match_scan(block_continuation(indent)) {
            break TreeDestin::Parent;    
        }
        if ctx.cursor.at_str("-- ") { 
            use_result!(parse_list(ctx, indent), 
                |child| node.children.push(ast::Node::List(child)));
        }
        let line_content = ctx.cursor.pop_line();
        node.children.push(ast::Node::Line(ast::Line { line_content }));
    };
    if matches!(node.children.last(), Some(ast::Node::VerticalSpace(_))) {
        node.children.pop();
    }
    return ParseResult { destin, node };
}

/// Advances the [`Cursor`] past the next *List* and assembles an [`ast::List`]
/// to represent the content.
/// 
/// This procedure will *never* returns in the middle of line.
/// In other words, the caller can assume that the [`Cursor`] is placed
/// at the beginning of a subsequent line (or EOF) after `parse` returns.
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
        if cursor.is_end() { return true; }
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


