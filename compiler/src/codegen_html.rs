use crate::ctree;

pub fn codegen<W>(root: &ctree::Root, out: &mut W) -> std::io::Result<()>
where W: std::io::Write
{
    codegen_block(&root.block, out)
}

pub fn codegen_node<W>(any_node: &ctree::BlockChild, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    match any_node {
        ctree::BlockChild::Verbatim   (node) => codegen_verbatim_block(node, out),
        ctree::BlockChild::Section    (node) => codegen_section(node, out),
        ctree::BlockChild::List       (node) => codegen_list(node, out),
        ctree::BlockChild::Block      (node) => codegen_block(node, out),
        ctree::BlockChild::Paragraph  (node) => codegen_paragraph(node, out),
        ctree::BlockChild::HTML       (node) => codegen_html_block(node, out),
        ctree::BlockChild::Heading    (node) => codegen_heading(node, out),
        ctree::BlockChild::CodeSnippet(node) => codegen_code_snippet(node, out),
        ctree::BlockChild::None              => Ok(()),
    }
}

pub fn codegen_code_snippet<'a, W>(snippet: &ctree::CodeSnippet<'a>, out: &mut W)
-> std::io::Result<()> where W: std::io::Write 
{
    write!(out, "<pre>")?;
    for line in &snippet.lines {
        out.write_all(line.as_ref().as_bytes())?;
        write!(out, "\n")?;
    }
    write!(out, "</pre>")?;
    return Ok(())
}

pub fn codegen_verbatim_block<W>(verbatim: &ctree::VerbatimBlock, out: &mut W) 
-> std::io::Result<()> where W: std::io::Write 
{
    write!(out, "<div>")?;
    for line in &verbatim.lines {
        out.write_all(line.as_ref().as_bytes())?;
        write!(out, "\n")?;
    }
    write!(out, "</div>")?;
    return Ok(());
}

pub fn codegen_section<W>(section: &ctree::Section, out: &mut W) 
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<section>")?;
    codegen_heading(&section.heading, out)?;
    for child in &section.children { codegen_node(child, out)?; }
    write!(out, "</section>")?;
    return Ok(());
}

pub fn codegen_list<W>(list: &ctree::List, out: &mut W) 
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<ul>")?;
    for element in &list.elements {
        write!(out, "<li>")?;
        write!(out, "<div>")?;
        codegen_block(&element.content, out)?;
        write!(out, "</div>")?;
        write!(out, "</li>")?;
    }
    write!(out, "</ul>")?;
    return Ok(())
}

pub fn codegen_block<W>(block: &ctree::Block, out: &mut W) 
-> std::io::Result<()> where W: std::io::Write
{
    for child in &block.children {
        codegen_node(child, out)?;
    }
    return Ok(())
}

pub fn codegen_paragraph<W>(paragraph: &ctree::Paragraph, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<p>")?;
    codegen_inline_root(&paragraph.content, out)?;
    write!(out, "</p>")?;
    return Ok(());
}

pub fn codegen_html_block<W>(node: &ctree::HTML, out: &mut W) 
-> std::io::Result<()> where W: std::io::Write
{
    node.value.write(out)
}

pub fn codegen_heading<W>(heading: &ctree::Heading, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<h{}>", heading.hlevel)?;
    codegen_inline_root(&heading.content, out)?;
    write!(out, "</h{}>", heading.hlevel)?;
    return Ok(());
}

pub fn codegen_inline_root<W>(node: &ctree::InlineRoot, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    for child in &node.children {
        codegen_inline_node(child, out)?;
    }
    return Ok(());
}

pub fn codegen_inline_node<W>(node: &ctree::AnyInline, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    match node {
        ctree::AnyInline::Plain            (node) => codegen_plain_text(node, out),
        ctree::AnyInline::Hyperlink        (node) => codegen_hyperlink(node, out),
        ctree::AnyInline::Emboldened       (node) => codegen_bold_text(node, out),
        ctree::AnyInline::Italicized       (node) => codegen_italicized_text(node, out),
        ctree::AnyInline::Underlined       (node) => codegen_underlined_text(node, out),
        ctree::AnyInline::TaggedSpan       (node) => codegen_tagged_text(node, out),
        ctree::AnyInline::ImplicitSpace    (node) => codegen_implicit_space(node, out),
        ctree::AnyInline::Verbatim   (node) => codegen_inline_verbatim(node, out),
        ctree::AnyInline::InlineCodeSnippet(node) => codegen_inline_code_snippet(node, out),
        ctree::AnyInline::HTML       (node) => codegen_inline_html(node, out),
        ctree::AnyInline::None                    => Ok(()),
    }
}

pub fn codegen_inline_html<W>(inline_html: &ctree::InlineHTML, out: &mut W) 
-> std::io::Result<()> where W: std::io::Write
{
    inline_html.value.write(out)
}

pub fn codegen_inline_code_snippet<W>(node: &ctree::InlineCodeSnippet, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<code>")?;
    for span in &node.inner_spans {
        out.write_all(span.as_ref().as_bytes())?;
    }
    write!(out, "</code>")?;
    return Ok(());
}

pub fn codegen_italicized_text<W>(node: &ctree::ItalicizedText, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<i>")?;
    codegen_inline_root(&node.child_root, out)?;
    write!(out, "</i>")?;
    return Ok(());
}

pub fn codegen_bold_text<W>(node: &ctree::EmboldenedText, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<b>")?;
    codegen_inline_root(&node.child_root, out)?;
    write!(out, "</b>")?;
    return Ok(());
}

pub fn codegen_underlined_text<W>(node: &ctree::UnderlinedText, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<u>")?;
    codegen_inline_root(&node.child_root, out)?;
    write!(out, "</u>")?;
    return Ok(());
}

pub fn codegen_inline_verbatim<W>(node: &ctree::InlineVerbatim, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<span>")?;
    for span in &node.content {
        out.write_all(span.as_ref().as_bytes())?;
        write!(out, " ")?;
    }
    write!(out, "</span>")?;
    return Ok(())
}

pub fn codegen_hyperlink<W>(node: &ctree::HyperlinkText, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<a href={}>", node.href.as_ref())?;
    codegen_inline_root(&node.child_root, out)?;
    write!(out, "</a>")?;
    return Ok(());
}

pub fn codegen_plain_text<W>(node: &ctree::PlainText, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<span>")?;
    out.write_all(node.span.as_ref().as_bytes())?;
    write!(out, "</span>")?;
    return Ok(());
}

pub fn codegen_implicit_space<W>(_node: &ctree::ImplicitSpace, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, " ")
}

pub fn codegen_tagged_text<W>(node: &ctree::TaggedSpan, out: &mut W)
-> std::io::Result<()> where W: std::io::Write
{
    codegen_inline_root(&node.child_root, out)
}
