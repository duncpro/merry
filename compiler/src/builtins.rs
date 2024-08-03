//! This module implements the builtin directive set. 
//!
//! - `href <tag> <url>` converts all spans marked with `<tag>` into hyperlinks to `<url>`.
//! - `rewrite <tag> <command>` 

use std::process::{Command, Stdio};
use std::io::Write;
use crate::assert_matches;
use crate::misc::{remove_first, Writable};
use crate::mtree::ast::DirectiveInvocation;
use crate::ctree::{AnyInline, HyperlinkText, InlineRoot, Context, Container, BlockChild, HTML, InlineHTML};
use crate::rewrite::{rewrite_subtrees, rewrite_inline_root};
use crate::scan::SourceSpan;

pub fn builtin_directives<'a, 'b, C>(invocation: DirectiveInvocation<'a>, scope: &mut C, 
    ctx: &mut Context<'a, 'b>) where C: Container<'a>
{
    if let Some(cmd) = invocation.cmd() {
        match cmd {
            "href" => apply_href(invocation.take_args(), scope, ctx),
            "rewrite" => apply_rewrite(invocation.take_args(), scope, ctx), 
            _ => ()
        }
    }
}

fn apply_href<'a, 'b, C>(args: Vec<SourceSpan<'a>>, scope: &mut C, ctx: &mut Context<'a, 'b>) 
where C: Container<'a> 
{
    let Some(tag) = args.get(0) else { /* TODO: Issue */ return; };
    let Some(href) = args.get(1) else { /* TODO: Issue */ return; };
    
    rewrite_subtrees(scope, &mut |node| {
        let Some(inline_content) = node.inline_content_mut() else { return; };
        rewrite_inline_root(inline_content, &mut |inline_node| {
            let AnyInline::TaggedSpan(tagged_span) = inline_node else { return; };
            if remove_first(&mut tagged_span.tags, |t| t.as_ref() == tag.as_ref()).is_none() { return; };
            let mut tmp = AnyInline::Hyperlink(HyperlinkText {
                href: href.clone(),
                child_root: InlineRoot::default()
            });
            std::mem::swap(&mut tmp, inline_node);
            assert_matches!(inline_node, AnyInline::Hyperlink(hyperlink));
            hyperlink.child_root.children.push(tmp);
        });
    });
}

fn apply_rewrite<'a, 'b, C>(args: Vec<SourceSpan<'a>>, scope: &mut C, ctx: &mut Context<'a, 'b>) 
where C: Container<'a>
{
    let Some(tag) = args.get(0) else { /* TODO: Issue */ return; };
    let Some(external_cmd) = args.get(1) else { return; };

    rewrite_subtrees(scope, &mut |node| {
        let BlockChild::Verbatim(verbatim) = node else { return; };
        if remove_first(&mut verbatim.tags, |t| t.as_ref() == tag.as_ref()).is_none() { return; }
        let mut tmp: BlockChild<'a> = BlockChild::None;
        std::mem::swap(&mut tmp, node);
        assert_matches!(tmp, BlockChild::Verbatim(verbatim));
        let src: Vec<SourceSpan<'a>> = verbatim.lines;
        let external_args: Vec<SourceSpan<'a>> = Vec::from(&args[2..]);
        let external_cmd = external_cmd.clone();
        let rewriter: Box<dyn Writable + 'a> = Box::new(ExternalRewriter { src, external_cmd,
            external_args });
        *node = BlockChild::HTML(HTML { value: rewriter });
    });

    rewrite_subtrees(scope, &mut |node| {
        let Some(inline_content) = node.inline_content_mut() else { return; };
        rewrite_inline_root(inline_content, &mut |inline_node| {
            let AnyInline::Verbatim(verbatim) = inline_node else { return; };
            if remove_first(&mut verbatim.tags, |t| t.as_ref() == tag.as_ref()).is_none() { return; };
            let mut tmp: AnyInline<'a> = AnyInline::None;
            std::mem::swap(&mut tmp, inline_node);
            assert_matches!(tmp, AnyInline::Verbatim(verbatim));
            let src: Vec<SourceSpan<'a>> = verbatim.content;
            let external_args: Vec<SourceSpan<'a>> = Vec::from(&args[2..]);
            let external_cmd = external_cmd.clone();
            let rewriter: Box<dyn Writable + 'a> = Box::new(ExternalRewriter { src, external_cmd, 
                external_args });
            *inline_node = AnyInline::HTML(InlineHTML { value: rewriter });
        });
    });
}

#[derive(Debug)]
pub struct ExternalRewriter<'a> { 
    pub src: Vec<SourceSpan<'a>>,
    pub external_args: Vec<SourceSpan<'a>>,
    pub external_cmd: SourceSpan<'a>
}

impl<'a> Writable for ExternalRewriter<'a> {
    fn write(&self, out: &mut dyn std::io::Write) -> std::io::Result<()> {
        let mut command = Command::new(self.external_cmd.as_ref()); 
        for external_arg in &self.external_args {
            command.arg(external_arg.as_ref());
        }  
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());

        let mut process = command.spawn()?;
        if let Some(mut stdin) = process.stdin.take() {
            for span in &self.src {
                stdin.write_all(span.as_ref().as_bytes())?;
            }
        }
        if let Some(mut stdout) = process.stdout.take() {
            std::io::copy(&mut stdout, out)?;
        }
        process.wait()?;
        
        return Ok(())
    }
}
