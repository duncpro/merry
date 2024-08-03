//! This module provides a facility for performing simple *CTree* rewrites. These kind
//! of rewrites are usually conducted as a consequence of a directive invocation.
//! 
//! However, for more complex rewrite, it may be easier and more effecient to implement 
//! a specialized suite of traversal routines instead of using the general-purpose facilities 
//! provided here.

use crate::ctree;

pub fn rewrite_subtree<'a>(node: &mut ctree::BlockChild<'a>, 
    rewrite_node: &mut impl FnMut(&mut ctree::BlockChild<'a>))
{
    (rewrite_node)(node);

    match node {
        ctree::BlockChild::Block(block) => {
            for child in &mut block.children { 
                rewrite_subtree(child, rewrite_node);
            }
        },
        ctree::BlockChild::Section(section) => {
            for child in &mut section.children { 
                rewrite_subtree(child, rewrite_node);
            }
        },
        ctree::BlockChild::List(list) => {
            for element in &mut list.elements {
                for child in &mut element.content.children {
                    rewrite_subtree(child, rewrite_node);
                }
            }
        },
        _ => ()
    }
}

pub fn rewrite_subtrees<'a>(container: &mut impl ctree::Container<'a>, 
    rewrite_node: &mut impl FnMut(&mut ctree::BlockChild<'a>))
{
    for child in container.children_mut() {
        rewrite_subtree(child, rewrite_node);
    }
}

pub fn rewrite_subtree_inline<'a>(node: &mut ctree::AnyInline<'a>, 
    rewrite_node: &mut impl FnMut(&mut ctree::AnyInline<'a>)) 
{
    (rewrite_node)(node);
    let maybe_root = match node {
        ctree::AnyInline::Hyperlink (child_node) => Some(&mut child_node.child_root),
        ctree::AnyInline::Emboldened(child_node) => Some(&mut child_node.child_root),
        ctree::AnyInline::Italicized(child_node) => Some(&mut child_node.child_root),
        ctree::AnyInline::Underlined(child_node) => Some(&mut child_node.child_root),
        ctree::AnyInline::TaggedSpan(child_node) => Some(&mut child_node.child_root),
        ctree::AnyInline::Plain(_) => None,
        ctree::AnyInline::ImplicitSpace(_) => None,
        ctree::AnyInline::InlineVerbatim(_) => None,
    };
    if let Some(root) = maybe_root {
        for child_node in &mut root.children {
            rewrite_subtree_inline(child_node, rewrite_node);
        }
    }
}

pub fn rewrite_inline_root<'a>(root: &mut ctree::InlineRoot<'a>,
    rewrite_node: &mut impl FnMut(&mut ctree::AnyInline<'a>))
{
    for child_node in &mut root.children {
        rewrite_subtree_inline(child_node, rewrite_node);
    }
}
