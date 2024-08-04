//! This module defines the *CTree*. The *CTree* is the working in-memory representation of the
//! finished document. 
//!
//! Unlike the *LTree* *MTree*, and *TTree* which are ASTs for the **source** text, the *CTree* 
//! resembles the **finished** document. Therefore, the structure of the *CTree* does not
//! exactly align with the structure of the input text, however they are similar. For instance
//! `crate::mtree::DirectiveInvocation` has no analog in the *CTree* because directive invocations
//! are not content to be published themselves. Furthermore, the *CTree* contains some new node
//! types which dont appear in the aforementioned ASTs. For example, the `HTMLBlock` node. 
//!
//! Another important property of the *CTree* is that it is lighter than the aforementioned ASTs.
//! The *CTree* does not contain syntactic information like the positions of the opening and closing
//! delimiters of spans for instance. Note that the CTree does contain references into the source 
//! text as there are many cases where the source text can be copied verbatim into the finished document.

use crate::builtins::builtin_directives;
use crate::misc::remove_first;
use crate::report::Issue;
use crate::scan::SourceSpan;
use crate::ttree;
use crate::mtree;

// # Inline Elements

#[derive(Default, Debug)] 
pub struct InlineRoot<'a> { pub children: Vec<AnyInline<'a>> }

#[derive(Debug)]
pub struct InlineVerbatim<'a> { pub content: Vec<SourceSpan<'a>>, pub tags: Vec<SourceSpan<'a>> }

#[derive(Debug)]
pub struct ImplicitSpace;

#[derive(Debug)]
pub enum AnyInline<'a> {
    Plain(PlainText<'a>),
    Hyperlink(HyperlinkText<'a>),
    Emboldened(EmboldenedText<'a>),
    Italicized(ItalicizedText<'a>),
    Underlined(UnderlinedText<'a>),
    TaggedSpan(TaggedSpan<'a>),
    ImplicitSpace(ImplicitSpace),
    Verbatim(InlineVerbatim<'a>),
    InlineCodeSnippet(InlineCodeSnippet<'a>),
    HTML(InlineHTML<'a>),

    /// This node has no effect during code-generation. It is however useful whem
    /// manipulating the tree in memory. Specifically, this node can be used `std::mem::swap`,
    /// to remove a node from the CTree before replacing it with another node.
    None 
}

// ## Inline Text Elements
#[derive(Debug)]
pub struct PlainText<'a> { pub span: SourceSpan<'a> }

#[derive(Debug)]
pub struct HyperlinkText<'a> { pub href: SourceSpan<'a>, pub child_root: InlineRoot<'a> }

#[derive(Debug)]
pub struct EmboldenedText<'a> { pub child_root: InlineRoot<'a> }

#[derive(Debug)]
pub struct ItalicizedText<'a> { pub child_root: InlineRoot<'a> }

#[derive(Debug)]
pub struct UnderlinedText<'a> { pub child_root: InlineRoot<'a> }

#[derive(Debug)]
pub struct TaggedSpan<'a> { pub child_root: InlineRoot<'a>, pub tags: Vec<SourceSpan<'a>> }

#[derive(Debug)]
pub struct InlineCodeSnippet<'a> { pub inner_spans: Vec<SourceSpan<'a>> }

#[derive(Debug)]
pub struct InlineHTML<'a> { pub value: Box<dyn Writable<'a> + 'a> }

// # Block Elements

#[derive(Debug)]
pub struct Root<'a> { pub block: Block<'a> }

#[derive(Debug)]
pub struct VerbatimBlock<'a> { pub lines: Vec<SourceSpan<'a>>, pub tags: Vec<SourceSpan<'a>> }

#[derive(Debug)]
pub struct Section<'a> { pub heading: Heading<'a>, pub children: Vec<BlockChild<'a>> }

#[derive(Debug)]
pub struct List<'a> { pub elements: Vec<ListElement<'a>> }

#[derive(Debug)]
pub struct Heading<'a> { pub hlevel: usize, pub content: InlineRoot<'a> }

#[derive(Debug)]
pub struct Paragraph<'a> { pub content: InlineRoot<'a> }

#[derive(Debug)]
pub struct CodeSnippet<'a> { pub lines: Vec<SourceSpan<'a>> }

#[derive(Default, Debug)]
pub struct Block<'a> { pub children: Vec<BlockChild<'a>> }

/// An arbitrary piece of HTML which will be embedded into the finished document during the 
/// code-generation phase.
#[derive(Debug)]
pub struct HTML<'a> { pub value: Box<dyn Writable<'a> + 'a> }

#[derive(Debug)]
pub enum BlockChild<'a> {
    Verbatim(VerbatimBlock<'a>),
    Section(Section<'a>),
    List(List<'a>),
    Block(Block<'a>),
    Paragraph(Paragraph<'a>),
    HTML(HTML<'a>),
    Heading(Heading<'a>),
    CodeSnippet(CodeSnippet<'a>),
    None
}


pub trait Container<'a> {
    fn children_mut(&mut self) -> &mut Vec<BlockChild<'a>>;
}

impl<'a> Container<'a> for Section<'a> {
    fn children_mut(&mut self) -> &mut Vec<BlockChild<'a>> {
        &mut self.children 
    }
}

impl<'a> Container<'a> for Block<'a> {
    fn children_mut(&mut self) -> &mut Vec<BlockChild<'a>> { 
        &mut self.children 
    }
}

impl<'a> BlockChild<'a> {
    /// Returns an exclusive reference to the inline content of this node (if it has some
    /// inline content), otherwise returns `None`. 
    ///
    /// Currently, the following node types have inline content....
    /// - Heading
    /// - Section (its heading's inline content)
    /// - Paragraph
    pub fn inline_content_mut(&mut self) -> Option<&mut InlineRoot<'a>> {
        match self {
            BlockChild::Section(node) => Some(&mut node.heading.content),
            BlockChild::Paragraph(node) => Some(&mut node.content),
            BlockChild::Heading(node) => Some(&mut node.content),
            _ => None
        }
    }
}

// # List Elements

#[derive(Debug)]
pub struct ListElement<'a> { pub content: Block<'a> }

// # Interpret *MTree*

pub struct Context<'a, 'b> { issues: &'b mut Vec<Issue<'a>> }

pub fn make_ctree<'a, 'b>(mtree: mtree::ast::Root<'a>, issues: &'b mut Vec<Issue<'a>>) -> Root<'a> 
{
    let mut ctx = Context { issues };
    let block = interpret_mtree_block(mtree.block, &mut ctx);
    return Root { block }
}

fn interpret_mtree_node<'a, 'b>(ctree_parent: &mut impl Container<'a>, 
    mtree_node: mtree::ast::BlockChild<'a>, ctx: &mut Context<'a, 'b>)
{
    match mtree_node {
        mtree::ast::BlockChild::Paragraph(mtree_p) => {
            let ctree_p = interpret_mtree_paragraph(mtree_p);
            ctree_parent.children_mut().push(BlockChild::Paragraph(ctree_p));
        },
        mtree::ast::BlockChild::Heading(mtree_h) => {
            let ctree_h = interpret_mtree_heading(mtree_h);
            ctree_parent.children_mut().push(BlockChild::Heading(ctree_h));
        },
        mtree::ast::BlockChild::Block(mtree_b) => {
            let ctree_b = interpret_mtree_block(mtree_b, ctx);
            ctree_parent.children_mut().push(BlockChild::Block(ctree_b));
        },
        mtree::ast::BlockChild::List(mtree_l) => {
            let ctree_l = interpret_mtree_list(mtree_l, ctx);
            ctree_parent.children_mut().push(BlockChild::List(ctree_l));
        },
        mtree::ast::BlockChild::VerbatimBlock(mtree_v) => {
            let ctree_v = interpret_mtree_verbatim(mtree_v);
            ctree_parent.children_mut().push(ctree_v);
        },
        mtree::ast::BlockChild::Section(mtree_s) => {
            let ctree_s = interpret_mtree_section(mtree_s, ctx);
            ctree_parent.children_mut().push(BlockChild::Section(ctree_s));
        },
        mtree::ast::BlockChild::Invoke(mtree_i) => {
            interpret_invocation(ctree_parent, mtree_i, ctx);
        }
    }
}

fn interpret_mtree_paragraph<'a>(ast_p: mtree::ast::Paragraph<'a>) -> Paragraph<'a> {
    let content = interpret_ttree_root(ast_p.content);
    return Paragraph { content };
}

fn interpret_mtree_heading<'a>(ast_h: mtree::ast::Heading<'a>) -> Heading<'a> {
    let content = interpret_ttree_root(ast_h.content);
    let hlevel = ast_h.hlevel;
    return Heading { hlevel, content };
}

fn interpret_mtree_block<'a, 'b>(ast_block: mtree::ast::Block<'a>, ctx: &mut Context<'a, 'b>)
 -> Block<'a> 
{
    let mut ctree_block = Block::default();
    for ast_child in ast_block.children {
        interpret_mtree_node(&mut ctree_block, ast_child, ctx);
    }
    return ctree_block;
}

fn interpret_mtree_list<'a, 'b>(ast_l: mtree::ast::List<'a>, ctx: &mut Context<'a, 'b>) 
-> List<'a> 
{
    let mut ctree_elements: Vec<ListElement<'a>> = Vec::new();
    for ast_element in ast_l.elements {
        let element_block = interpret_mtree_block(ast_element.content, ctx);
        ctree_elements.push(ListElement { content: element_block });
    }
    return List { elements: ctree_elements };
}

fn interpret_mtree_verbatim<'a>(ast_v: mtree::ast::VerbatimBlock<'a>) -> BlockChild<'a> {
    let mut tags = make_tags(ast_v.trailing_qualifier);

    if remove_first(&mut tags, |t| t.as_ref() == "m").is_some() {
        // TODO: Report issue if there are other tags on this verbatim block.
        //       The m tag cannot coexist with other tags on a verbatim block.
        return BlockChild::CodeSnippet(CodeSnippet { lines: ast_v.lines });
    }
    
    return BlockChild::Verbatim(VerbatimBlock { lines: ast_v.lines, tags });
}

fn interpret_mtree_section<'a, 'b>(ast_s: mtree::ast::Section<'a>, ctx: &mut Context<'a, 'b>) 
-> Section<'a> 
{
    let mut ctree_section: Section<'a> = Section {
        heading: interpret_mtree_heading(ast_s.heading),
        children: Vec::new()
    };
    
    for ast_child in ast_s.children {
        interpret_mtree_node(&mut ctree_section, ast_child, ctx);
     }

    return ctree_section;
}

fn interpret_invocation<'a, 'b>(scope: &mut impl Container<'a>, 
    invocation: mtree::ast::DirectiveInvocation<'a>, ctx: &mut Context<'a, 'b>)
{
    builtin_directives(invocation, scope, ctx);
}

// # Interpret *TTree*

/// Interprets the given *TTree*, consuming it, and producing a *CTree*.
fn interpret_ttree_root<'a>(ttree_root: ttree::ast::Root<'a>) -> InlineRoot<'a> {
    let mut ctree_root = InlineRoot::default();
    for ttree_node in ttree_root.children {
        let ctree_child = interpret_ttree_node(ttree_node);
        ctree_root.children.push(ctree_child);
    }
    return ctree_root;
}

fn interpret_ttree_node<'a>(inline_ast_node: ttree::ast::AnyInline<'a>) -> AnyInline<'a> 
{
    match inline_ast_node {
        ttree::ast::AnyInline::Plain(ast_node) => 
            AnyInline::Plain(PlainText { span: ast_node.span }),
        ttree::ast::AnyInline::Delimited(ast_node) => 
            interpret_delimeted_text(ast_node),
        ttree::ast::AnyInline::InlineVerbatim(ast_node) => 
            interpret_inline_verbatim(ast_node),
        ttree::ast::AnyInline::ImplicitSpace(_) => 
            AnyInline::ImplicitSpace(ImplicitSpace),
        ttree::ast::AnyInline::Bracketed(ast_node) =>
            interpret_bracketed_text(ast_node),
    }
}

fn interpret_inline_verbatim<'a>(ast_node: ttree::ast::InlineVerbatim<'a>) -> AnyInline<'a> {
    let mut tags = make_tags(ast_node.trailing_qualifier);

    if remove_first(&mut tags, |t| t.as_ref() == "m").is_some() {
        let inner_spans = ast_node.inner_spans;
        let snippet = AnyInline::InlineCodeSnippet(InlineCodeSnippet { inner_spans });
        let mut wrapper = TaggedSpan { child_root: InlineRoot::default(), tags };
        wrapper.child_root.children.push(snippet);
        return AnyInline::TaggedSpan(wrapper);
    }

    let content = ast_node.inner_spans;
    return AnyInline::Verbatim(InlineVerbatim { content, tags });
}

fn interpret_delimeted_text<'a>(ast_node: ttree::ast::DelimitedText<'a>) -> AnyInline<'a> {
    match ast_node.delim_kind {
        ttree::ast::DelimiterKind::Asterisk => {
            AnyInline::Emboldened(EmboldenedText { 
                child_root: interpret_ttree_root(ast_node.child_root) 
            })
        },
        ttree::ast::DelimiterKind::Underscore => {
            AnyInline::Underlined(UnderlinedText { 
                child_root: interpret_ttree_root(ast_node.child_root)
            })
        },
        ttree::ast::DelimiterKind::Tilde => {
            AnyInline::Italicized(ItalicizedText { 
                child_root: interpret_ttree_root(ast_node.child_root)
            })
        },
    }
}

fn interpret_bracketed_text<'a>(ast_node: ttree::ast::BracketedText<'a>) -> AnyInline<'a> {
    AnyInline::TaggedSpan(TaggedSpan { 
        child_root: interpret_ttree_root(ast_node.child_root),
        tags: make_tags(ast_node.trailing_qualifier)
    })
}

fn make_tags<'a>(ast_node: Option<ttree::ast::TrailingQualifier<'a>>) 
-> Vec<SourceSpan<'a>>
{
    let mut tags: Vec<SourceSpan<'a>> = Vec::new();
    if let Some(trailing_qualifier) = ast_node {
        for tag_ast_node in  trailing_qualifier.tags {
            if let ttree::ast::Tag::Unsplit(unsplit_tag) = tag_ast_node {
                tags.push(unsplit_tag.span);
            }
        }
    }
    return tags;    
}

pub trait Writable<'a>: std::fmt::Debug {
    /// Writes this entire `Writable` to `out`. 
    ///
    /// As the implementor of this trait is likely the only object which knows exactly
    /// *why* this node appears in the tree, it is *its responsibility* to package any
    /// IO errors which occur into an `Issue`. For instance, the implementor likely
    /// knows which directive invocation produced this node, and also the node which
    /// was originally taggged/rewritten (if any).
    fn write(&self, out: &mut dyn std::io::Write, issues: &mut Vec<Issue<'a>>);
}
