use crate::mtree;
use crate::ttree;

pub fn codegen<'a, 'b, W>(mtree: &'a mtree::ast::Root<'b>, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out,"<!DOCTYPE html>")?;
    write!(out, "<html>")?;
    write!(out, "<body>")?;
    codegen_block_child(&mtree.child, out)?;
    write!(out, "</body>")?;
    write!(out, "</html>")?;
    return Ok(())
}

pub fn codegen_block<'a, 'b, W>(block: &'a mtree::ast::Block<'b>, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    for child in &block.children {
        codegen_block_child(child, out)?;
    }
    return Ok(())
}

pub fn codegen_block_child<'a, 'b, W>(child: &'a mtree::ast::BlockChild<'b>, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    match child {
        mtree::ast::BlockChild::Paragraph(node) => codegen_paragraph(&node, out)?,
        mtree::ast::BlockChild::Invoke(_) => todo!(),
        mtree::ast::BlockChild::Heading(node) => codegen_heading(&node, out)?,
        mtree::ast::BlockChild::Block(node) => codegen_block(node, out)?,
        mtree::ast::BlockChild::List(node) => codegen_list(&node, out)?,
        mtree::ast::BlockChild::Verbatim(node) => codegen_verbatim_block(node, out)?,
        mtree::ast::BlockChild::Section(node) => codegen_section(&node, out)?,
    }
    return Ok(())
}

pub fn codegen_section<'a, 'b, W>(section: &'a mtree::ast::Section<'b>, out: &mut W) 
-> std::io::Result<()> where W: std::io::Write 
{
    write!(out, "<section>")?;
    codegen_heading(&section.heading, out)?;
    for child in &section.children {
        codegen_block_child(child, out)?;
    }
    write!(out, "</section>")?;
    return Ok(());
}

pub fn codegen_paragraph<'a, 'b, W>(block: &'a mtree::ast::Paragraph<'b>, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<p>")?;
    codegen_ttree(&block.content, out)?;
    write!(out, "</p>")?;
    return Ok(())
}

pub fn codegen_heading<'a, 'b, W>(heading: &'a mtree::ast::Heading<'b>, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<h{}>", heading.hlevel)?;
    codegen_ttree(&heading.content, out)?;
    write!(out, "</h{}>", heading.hlevel)?;
    return Ok(())
}

pub fn codegen_list<'a, 'b, W>(list: &'a mtree::ast::List<'b>, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<ul>")?;
    for element in &list.elements {
        write!(out, "<li><div>")?;
        codegen_block(&element.content, out)?;
        write!(out, "</div></li>")?;
    }
    write!(out, "</ul>")?;
    return Ok(());
}

pub fn codegen_ttree<'a, 'b, W>(root: &'a ttree::ast::Root<'b>, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    for child in &root.children {
        match child {
            ttree::ast::AnyText::Plain(node) => codegen_plain_text(node, out)?,
            ttree::ast::AnyText::Delimited(node) => codegen_delim_span(node, out)?,
            ttree::ast::AnyText::InlineVerbatim(node) => codegen_verbatim_inline(node, out)?,
            ttree::ast::AnyText::ImplicitSpace(node) => codegen_implicit_space(node, out)?,
            ttree::ast::AnyText::Bracketed(node) => codegen_bracket_span(node, out)?,
            ttree::ast::AnyText::HTMLWrap(node) => codegen_ttree_wrap(node, out)?,
        }
    }
    return Ok(())
}

pub fn codegen_ttree_wrap<'a, 'b, W>(node: &'a ttree::ast::HTMLWrap<'b>, out: &mut W) 
-> std::io::Result<()> where W: std::io::Write 
{
    write!(out, "{}", node.prefix)?;
    codegen_ttree(&node.wrapped, out)?;
    write!(out, "{}", node.suffix)?;
    return Ok(());
}

pub fn codegen_implicit_space<'a, W>(_node: &'a ttree::ast::ImplicitSpace, out: &mut W) 
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, " ")    
}

pub fn codegen_verbatim_inline<'a, 'b, W>(node: &'a ttree::ast::InlineVerbatim<'b>, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<code>")?;
    for span in &node.inner_spans {
        write!(out, "{}", span.as_ref())?;
    }
    write!(out, "</code>")?;
    return Ok(())    
}

pub fn codegen_plain_text<'a, 'b, W>(node: &'a ttree::ast::PlainText, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "{}", node.span.as_ref())?;
    return Ok(());
}

pub fn codegen_delim_span<'a, 'b, W>(node: &'a ttree::ast::DelimitedText<'b>, out: &mut W) 
-> std::io::Result<()> where W: std::io::Write
{
    match node.delim_kind {
        ttree::ast::DelimiterKind::Asterisk => write!(out, "<b>")?,
        ttree::ast::DelimiterKind::Underscore => write!(out, "<u>")?,
        ttree::ast::DelimiterKind::Tilde => write!(out, "<i>")?,
    }   
    codegen_ttree(&node.child_root, out)?;
    match node.delim_kind {
        ttree::ast::DelimiterKind::Asterisk => write!(out, "</b>")?,
        ttree::ast::DelimiterKind::Underscore => write!(out, "</u>")?,
        ttree::ast::DelimiterKind::Tilde => write!(out, "</i>")?,
    }   
    return Ok(())    
}

pub fn codegen_bracket_span<'a, 'b, W>(node: &'a ttree::ast::BracketedText<'b>, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    codegen_ttree(&node.child_root, out)?;
    return Ok(())
}

pub fn codegen_verbatim_block<'a, 'b, W>(node: &'a mtree::ast::Verbatim<'b>, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<pre>")?;
    write!(out, "<code>")?;
    for line in &node.lines {
        writeln!(out, "{}", line.as_ref())?;
    }
    write!(out, "</code>")?;
    write!(out, "</pre>")?;
    return Ok(());
}
