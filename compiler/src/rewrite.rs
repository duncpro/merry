//! This module provides a facility for performing simple AST rewrites. These kind
//! of rewrites are usually conducted as a consequence of a directive invocation.
//! 
//! However, for more complex rewrite, it may be easier and more effecient to implement 
//! a specialized suite of traversal routines instead of using the general-purpose facilities 
//! provided here.

use crate::ttree;

pub fn rewrite_ttree(root: &mut ttree::ast::Root, rewrite_node: &impl Fn(&mut ttree::ast::AnyText)) 
{
    for child in &mut root.children {
        (rewrite_node)(child);
        match child {
            ttree::ast::AnyText::Delimited(node) => 
                rewrite_ttree(&mut node.child_root, rewrite_node),
            ttree::ast::AnyText::Bracketed(node) => 
                rewrite_ttree(&mut node.child_root, rewrite_node),
            ttree::ast::AnyText::Plain(_) => {},
            ttree::ast::AnyText::InlineVerbatim(_) => {},
            ttree::ast::AnyText::ImplicitSpace(_) => {},
        }
    }
}

use crate::mtree;

