use crate::{assert_matches, ttree};
use crate::mtree::ast::BlockChild;
use crate::report::Issue;
use crate::rewrite::{rewrite_mtree, rewrite_ttree};
use crate::scan::SourceSpan;

/// Scans the given *MTree* for directive invocations matching the command `cmd`.
/// Then executes `rule` for each node that is *in scope* of the directive.
/// Finally, removes the directive invocation node from the tree.
///
/// A directive which acts on a limited scope is called a *scoped directive*.
/// Not all directives are *scoped directives* but many are, for instance `href` is.
///
/// The scope begins with the first child of the container (block or section) containing the
/// invocation and ends at the directive invocation itself.
fn apply_scoped_rewrite_directive<'a, D: Directive<'a>>(root: &mut BlockChild<'a>, 
    cmd: &str, issues: &mut Vec<Issue<'a>>)
{
    rewrite_mtree(root, &mut |node| {
        if let Some(children) = node.children_mut() {
            let mut i: usize = 0;
            while i < children.len() {
                if let BlockChild::Invoke(invoke) = &children[i] {
                    if invoke.cmd() == Some(cmd) {
                        assert_matches!(children.remove(i), BlockChild::Invoke(invoke));
                        if let Some(directive) = D::from_args(invoke.args(), issues) {
                            for sibling in &mut children[0..i] {
                                directive.do_rewrite(sibling);
                            }
                        }
                        continue;
                    }
                }
                i += 1;
            }
        }
    });
}

trait Directive<'a>: Sized {
    /// Constructs a directive from the arguments passed in the directive invocation.
    /// If the arguments are malformed, the error should be reproted by appending an
    /// an issue to `issues`. If the arguments are so malformed that the directive
    /// can not be constructed, then `None` may be returned. In this case, the directive invocation
    /// is still removed from the tree.
    fn from_args<'b>(args: &'b [SourceSpan<'a>], issues: &'b mut Vec<Issue<'a>>) -> Option<Self>;

    /// The rewrite rule for the directive. This procedure will be invoked for each node
    /// in the scope of the directive invocation.
    fn do_rewrite(&self, node: &mut BlockChild);
}

pub struct HrefDirective<'a> { tag: SourceSpan<'a>, href: SourceSpan<'a> }

impl<'a> Directive<'a> for HrefDirective<'a> {
    fn from_args<'b>(args: &'b [SourceSpan<'a>], issues: &'b mut Vec<Issue<'a>>) -> Option<Self> {
        let tag = args.get(0)?.clone(); // TODO: Report missing tag argument if None
        let href = args.get(1)?.clone(); // TODO: Report missing URL argument if None
        return Some(Self { tag, href });
    }

    fn do_rewrite(&self, node: &mut BlockChild) {
        let BlockChild::Paragraph(paragraph) = node else { return; };
        rewrite_ttree(&mut paragraph.content, &mut |text_node| {
            let ttree::ast::AnyText::Bracketed(bracketed) = text_node else { return };
            let Some(qualifier) = &mut bracketed.trailing_qualifier else { return };
            if !qualifier.contains_tag(self.tag.as_ref()) { return; }
            qualifier.remove_tag(self.tag.as_ref());
            let prefix = format!("<a href=\"{}\">", self.href.as_ref());
            let suffix = format!("</a>");
            let mut tmp = ttree::ast::AnyText::HTMLWrap(
                ttree::ast::HTMLWrap::new(prefix, suffix)); 
            std::mem::swap(text_node, &mut tmp);
            assert_matches!(text_node, ttree::ast::AnyText::HTMLWrap(wrapper));
            wrapper.wrapped.children.push(tmp);
        });
    }
}

pub fn apply_builtin_directives<'a>(root: &mut BlockChild<'a>, issues: &mut Vec<Issue<'a>>) {
    apply_scoped_rewrite_directive::<HrefDirective>(root, "href", issues);
}
