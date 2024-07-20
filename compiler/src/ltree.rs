//! This module provides a facility for constructing an *LTree* given
//! some source text. An *LTree* represents the hierarchical structure of the
//! source document logically in memory.
//!
//! Constructing an *LTree* is the first-step in parsing an md2 document.

pub mod ast {
    use crate::parse::SourceSpan;

    #[derive(Debug)]
    pub struct Root<'a> {
        pub children: Vec<Node<'a>>
    }

    #[derive(Debug)]
    pub enum Node<'a> {
        List(List<'a>),
        Line(Line<'a>),
        VerticalSpace(VerticalSpace)
    }

    #[derive(Debug)]
    pub struct List<'a> {
        pub children: Vec<ListElement<'a>>
    }

    #[derive(Debug)]
    pub struct ListElement<'a> {
        pub content: Root<'a>
    }

    #[derive(Debug)]
    pub struct Line<'a> { pub line_content: SourceSpan<'a> }

    #[derive(Debug)]
    pub struct VerticalSpace;
}


use crate::parse::Cursor;

pub struct ParseContext<'a> {
    cursor: Cursor<'a>
}

pub fn make_ltree(source: &String) -> ast::Root {
    let mut ctx = ParseContext { cursor: Cursor::new(source) };
    let (root, _) = parse_root(&mut ctx, 0);
    return root;
}

fn parse_root<'a, 'b>(ctx: &'a mut ParseContext<'b>, level: usize) -> (ast::Root<'b>, bool)
{
    let mut children: Vec<ast::Node<'b>> = Vec::new();
    let mut consec_blank_line_count: usize = 0;
    while !ctx.cursor.is_end() {
        if ctx.cursor.match_blank_line() {
            consec_blank_line_count += 1;
            if consec_blank_line_count == 1 {
                children.push(ast::Node::VerticalSpace(ast::VerticalSpace));
            }
            if (consec_blank_line_count == 2) & (level > 0) { 
                return (ast::Root { children }, true);
            }
            continue;
        }
        consec_blank_line_count = 0;
        // TODO: We need to handle the case where the indent
        //       is deeper than we expect it to be.
        // TODO: We should change level to be a measure of spaces,
        //       this way we can support arbitrary indent scheme.
        // Perhaps, when the indent is larger than we expect,
        // we create a `Nested` node which is a child of this node,
        // but represents an arbitrary nested block.

        // However **we must** preserve the behavior that if the
        // indent is shallower than we expect, we break! Because
        // the line is likely an ancestor's sibling.
        if !match_level(&mut ctx.cursor, level) { break; }
        if ctx.cursor.at_str("-- ") { 
            let (list, double_space) = parse_list(ctx, level);
            children.push(ast::Node::List(list));
            if double_space & (level > 0) { 
                return (ast::Root { children }, true);  
            }
            continue;
        }
        let line_content = ctx.cursor.pop_line();
        children.push(ast::Node::Line(ast::Line { line_content }));
    }
    if matches!(children.last(), Some(ast::Node::VerticalSpace(_))) {
        children.pop();
    }
    return (ast::Root { children }, false);
}

fn parse_list<'a, 'b>(ctx: &'a mut ParseContext<'b>, level: usize) -> (ast::List<'b>, bool) {
    let mut children: Vec<ast::ListElement<'b>> = Vec::new();
    loop {
        if !match_level(&mut ctx.cursor, level) { break; }
        if ctx.cursor.match_str("-- ") {
            let (content, double_space) = parse_root(ctx, level + 1);
            children.push(ast::ListElement { content });
            if double_space { return (ast::List { children }, true); }
            continue;
        }
        break;
    }
    return (ast::List { children }, false);
}
    
fn match_level<'a, 'b>(cursor: &'a mut Cursor<'b>, level: usize) -> bool {
    let indent = cursor.peek_while(|ch| ch == ' ').end_col();
    if indent != 3 * level { return false; }
    cursor.advance_n_chars(indent - cursor.pos().colu_pos);
    return true;
}
