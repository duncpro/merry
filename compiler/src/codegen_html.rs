use crate::{ctree, report::Issue};

pub fn codegen<'a, W>(root: &ctree::Root<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>) -> std::io::Result<()>
where W: std::io::Write
{
    write!(out, "<!DOCTYPE html>")?;
    write!(out, "<html>")?;
    write!(out, "<head>")?;
    write!(out, "</head>")?;
    write!(out, "<body>")?;
    codegen_block(&root.block, out, issues)?;
    write!(out, "</body>")?;
    write!(out, "</html>")?;
    return Ok(());
}

pub fn codegen_node<'a, W>(any_node: &ctree::BlockChild<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>)
-> std::io::Result<()> where W: std::io::Write
{
    match any_node {
        ctree::BlockChild::Verbatim     (node) => codegen_verbatim_block(node, out),
        ctree::BlockChild::Section      (node) => codegen_section(node, out, issues),
        ctree::BlockChild::List         (node) => codegen_list(node, out, issues),
        ctree::BlockChild::Block        (node) => codegen_block(node, out, issues),
        ctree::BlockChild::Paragraph    (node) => codegen_paragraph(node, out, issues),
        ctree::BlockChild::HTML         (node) => codegen_html_block(node, out, issues),
        ctree::BlockChild::Heading      (node) => codegen_heading(node, out, issues),
        ctree::BlockChild::CodeSnippet  (node) => codegen_code_snippet(node, out),
        ctree::BlockChild::ThematicBreak(node) => codegen_thematic_break(node, out),
        ctree::BlockChild::None                => Ok(()),
    }
}

pub fn codegen_thematic_break<'a, W>(_node: &ctree::ThematicBreak, out: &mut W) 
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<hr/>")   
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

pub fn codegen_section<'a, W>(section: &ctree::Section<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>) 
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<section>")?;
    write!(out, "<h1>")?;
    codegen_inline_root(&section.heading.content, out, issues)?;
    write!(out, "</h1>")?;
    for child in &section.children { codegen_node(child, out, issues)?; }
    write!(out, "</section>")?;
    return Ok(());
}

pub fn codegen_list<'a, W>(list: &ctree::List<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>) 
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<ul>")?;
    for element in &list.elements {
        write!(out, "<li>")?;
        write!(out, "<div>")?;
        codegen_block(&element.content, out, issues)?;
        write!(out, "</div>")?;
        write!(out, "</li>")?;
    }
    write!(out, "</ul>")?;
    return Ok(())
}

pub fn codegen_block<'a, W>(block: &ctree::Block<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>) 
-> std::io::Result<()> where W: std::io::Write
{
    if block.indent { write!(out, "<div style=\"margin-left: 20px\">")?; }
    for child in &block.children {
        codegen_node(child, out, issues)?;
    }
    if block.indent { write!(out, "</div>")?; }
    return Ok(())
}

pub fn codegen_paragraph<'a, W>(paragraph: &ctree::Paragraph<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<p>")?;
    codegen_inline_root(&paragraph.content, out, issues)?;
    write!(out, "</p>")?;
    return Ok(());
}

pub fn codegen_html_block<'a, W>(node: &ctree::HTML<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>) 
-> std::io::Result<()> where W: std::io::Write
{
    node.value.write(out, issues);
    return Ok(());
}

pub fn codegen_heading<'a, W>(heading: &ctree::Heading<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<h{}>", heading.hlevel)?;
    codegen_inline_root(&heading.content, out, issues)?;
    write!(out, "</h{}>", heading.hlevel)?;
    return Ok(());
}

pub fn codegen_inline_root<'a, W>(node: &ctree::InlineRoot<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>)
-> std::io::Result<()> where W: std::io::Write
{
    for child in &node.children {
        codegen_inline_node(child, out, issues)?;
    }
    return Ok(());
}

pub fn codegen_inline_node<'a, W>(node: &ctree::AnyInline<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>)
-> std::io::Result<()> where W: std::io::Write
{
    match node {
        ctree::AnyInline::Plain            (node) => codegen_plain_text(node, out),
        ctree::AnyInline::Hyperlink        (node) => codegen_hyperlink(node, out, issues),
        ctree::AnyInline::Emboldened       (node) => codegen_bold_text(node, out, issues),
        ctree::AnyInline::Italicized       (node) => codegen_italicized_text(node, out, issues),
        ctree::AnyInline::Underlined       (node) => codegen_underlined_text(node, out, issues),
        ctree::AnyInline::TaggedSpan       (node) => codegen_tagged_text(node, out, issues),
        ctree::AnyInline::ImplicitSpace    (node) => codegen_implicit_space(node, out),
        ctree::AnyInline::Verbatim         (node) => codegen_inline_verbatim(node, out),
        ctree::AnyInline::InlineCodeSnippet(node) => codegen_inline_code_snippet(node, out),
        ctree::AnyInline::HTML             (node) => codegen_inline_html(node, out, issues),
        ctree::AnyInline::None                    => Ok(()),
    }
}

pub fn codegen_inline_html<'a, W>(inline_html: &ctree::InlineHTML<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>) 
-> std::io::Result<()> where W: std::io::Write
{
    inline_html.value.write(out, issues);
    return Ok(());
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

pub fn codegen_italicized_text<'a, W>(node: &ctree::ItalicizedText<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<i>")?;
    codegen_inline_root(&node.child_root, out, issues)?;
    write!(out, "</i>")?;
    return Ok(());
}

pub fn codegen_bold_text<'a, W>(node: &ctree::EmboldenedText<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<b>")?;
    codegen_inline_root(&node.child_root, out, issues)?;
    write!(out, "</b>")?;
    return Ok(());
}

pub fn codegen_underlined_text<'a, W>(node: &ctree::UnderlinedText<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<u>")?;
    codegen_inline_root(&node.child_root, out, issues)?;
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

pub fn codegen_hyperlink<'a, W>(node: &ctree::HyperlinkText<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>)
-> std::io::Result<()> where W: std::io::Write
{
    write!(out, "<a href={}>", node.href.as_ref())?;
    codegen_inline_root(&node.child_root, out, issues)?;
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

pub fn codegen_tagged_text<'a, W>(node: &ctree::TaggedSpan<'a>, out: &mut W, issues: &mut Vec<Issue<'a>>)
-> std::io::Result<()> where W: std::io::Write
{
    codegen_inline_root(&node.child_root, out, issues)
}
