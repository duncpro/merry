//! This module implements the builtin directive set. 
//!
//! - `href <tag> <url>` converts all spans marked with `<tag>` into hyperlinks to `<url>`.
//! - `transform <tag> <command>` 

use crate::assert_matches;
use crate::misc::remove_first;
use crate::mtree::ast::DirectiveInvocation;
use crate::ctree::{self, AnyInline, HyperlinkText, InlineRoot, Context, Container};
use crate::rewrite::{rewrite_subtrees, rewrite_inline_root};
use crate::scan::SourceSpan;

pub fn builtin_directives<'a, 'b, C>(invocation: DirectiveInvocation<'a>, scope: &mut C, 
    ctx: &mut Context<'a, 'b>) where C: Container<'a>
{
    if let Some(cmd) = invocation.cmd() {
        match cmd {
            "href" => apply_href(invocation.args(), scope, ctx),
            _ => ()
        }
    }
}

fn apply_href<'a, 'b, C>(args: &[SourceSpan<'a>], scope: &mut C, ctx: &mut Context<'a, 'b>) 
where C: Container<'a> 
{
    let Some(tag) = args.get(0) else { /* TODO: Issue */ return; };
    let Some(href) = args.get(1) else { /* TODO: Issue */ return; };
    
    rewrite_subtrees(scope, &mut |node| {
        let maybe_inline_content = match node {
            ctree::BlockChild::Paragraph(p) => Some(&mut p.content),
            ctree::BlockChild::Heading(h) => Some(&mut h.content),
            _ => None
        };
        if let Some(inline_content) = maybe_inline_content {
            rewrite_inline_root(inline_content, &mut |inline_node| {
                if let ctree::AnyInline::TaggedSpan(tagged_span) = inline_node {
                    if remove_first(&mut tagged_span.tags, |t| t.as_ref() == tag.as_ref()).is_some() {
                        let mut tmp = AnyInline::Hyperlink(HyperlinkText {
                            href: href.clone(),
                            child_root: InlineRoot::default()
                        });
                        std::mem::swap(&mut tmp, inline_node);
                        assert_matches!(inline_node, AnyInline::Hyperlink(hyperlink));
                        hyperlink.child_root.children.push(tmp);
                    }
                }
            });
        }
    });
}
