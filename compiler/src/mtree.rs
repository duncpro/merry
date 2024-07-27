mod ast {
    use crate::ltree;
    use crate::ttree;

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
        pub line: ltree::ast::Line<'a>
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
    }

    #[derive(Debug)]
    pub struct List<'a> {
        pub elements: Vec<ListElement<'a>>
    }

    #[derive(Debug)]
    pub struct ListElement<'a> {
        pub block: Block<'a>
    }

    #[derive(Debug)]
    pub struct Root<'a> {
        pub block: Block<'a>
    }
}

use crate::{ltree, ttree, scan::SourceSpan};

pub fn make_mtree<'a, 'b>(ltree: &'a ltree::ast::Root<'b>) -> ast::Root<'b> {
    let block = make_block(&ltree.block);
    return ast::Root { block }
}

pub fn make_block<'a, 'b>(ltree_block: &'a ltree::ast::Block<'b>) -> ast::Block<'b> {
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
            paragraph_lines.push(line.line_content);
            continue;
        }
        push_paragraph!();
        if let ltree::ast::BlockChild::Block(ltree_child_block) = ltree_child {
            let mtree_child_block = make_block(ltree_child_block);
            mtree_children.push(ast::BlockChild::Block(mtree_child_block));
            continue;
        }
    }
    push_paragraph!();
    return ast::Block { children: mtree_children }
}
