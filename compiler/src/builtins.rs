//! This module implements the builtin directive set. 
//!
//! - `href <tag> <url>` converts all spans marked with `<tag>` into hyperlinks to `<url>`.
//! - `rewrite <tag> <command>` 

use std::process::{Command, Stdio};
use std::io::{Write, read_to_string, Read};
use crate::{assert_matches, req};
use crate::misc::remove_first;
use crate::mtree::ast::DirectiveInvocation;
use crate::ctree::{AnyInline, BlockChild, Container, Context, HTML, HyperlinkText, InlineHTML, InlineRoot, Writable};
use crate::report::{Issue, AnnotatedSourceSection, Severity, Elaboration, QuoteElaboration, SourceQuoteElaboration};
use crate::rewrite::{rewrite_inline_root, rewrite_subtrees};
use crate::scan::SourceSpan;

pub fn builtin_directives<'a, 'b, C>(invocation: DirectiveInvocation<'a>, scope: &mut C, 
    ctx: &mut Context<'a, 'b>) where C: Container<'a>
{
    if let Some(cmd) = invocation.cmd() {
        match cmd {
            "href" => apply_href(invocation.take_args(), scope, ctx),
            "rewrite" => apply_rewrite(invocation.take_args(), scope, ctx), 
            "embed" => apply_embed(invocation.take_args(), scope, ctx),
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
    let Some(external_cmd) = args.get(1) else { /* TODO: Issue */ return; };

    rewrite_subtrees(scope, &mut |node| {
        let BlockChild::Verbatim(verbatim) = node else { return; };
        req!(Some(verbatim_tag), 
            remove_first(&mut verbatim.tags, |t| t.as_ref() == tag.as_ref()));
        let mut tmp: BlockChild<'a> = BlockChild::None;
        std::mem::swap(&mut tmp, node);
        assert_matches!(tmp, BlockChild::Verbatim(verbatim));
        let src: Vec<SourceSpan<'a>> = verbatim.lines;
        let external_args: Vec<SourceSpan<'a>> = Vec::from(&args[2..]);
        let external_cmd = external_cmd.clone();
        let rewriter: Box<dyn Writable<'a> + 'a> = Box::new(ExternalRewriter { 
            src, external_cmd, external_args, verbatim_tag, cwd: ctx.cwd.clone() });
        *node = BlockChild::HTML(HTML { value: rewriter });
    });

    rewrite_subtrees(scope, &mut |node| {
        let Some(inline_content) = node.inline_content_mut() else { return; };
        rewrite_inline_root(inline_content, &mut |inline_node| {
            let AnyInline::Verbatim(verbatim) = inline_node else { return; };
            req!(Some(verbatim_tag), 
                remove_first(&mut verbatim.tags, |t| t.as_ref() == tag.as_ref()));            
            let mut tmp: AnyInline<'a> = AnyInline::None;
            std::mem::swap(&mut tmp, inline_node);
            assert_matches!(tmp, AnyInline::Verbatim(verbatim));
            let src: Vec<SourceSpan<'a>> = verbatim.content;
            let external_args: Vec<SourceSpan<'a>> = Vec::from(&args[2..]);
            let external_cmd = external_cmd.clone();
            let rewriter: Box<dyn Writable<'a> + 'a> = Box::new(ExternalRewriter { 
                src, external_cmd, external_args, verbatim_tag, cwd: ctx.cwd.clone() });
            *inline_node = AnyInline::HTML(InlineHTML { value: rewriter });
        });
    });
}


#[derive(Debug)]
struct ExternalRewriter<'a> {
    pub src: Vec<SourceSpan<'a>>,
    pub external_args: Vec<SourceSpan<'a>>,
    pub external_cmd: SourceSpan<'a>,
    pub verbatim_tag: SourceSpan<'a>,
    pub cwd: std::path::PathBuf
}

impl<'a> Writable<'a> for ExternalRewriter<'a> {
    
    fn write(&self, out: &mut dyn std::io::Write, issues: &mut Vec<Issue<'a>>) {
        let mut command = Command::new(self.external_cmd.as_ref()); 
        for external_arg in &self.external_args {
            command.arg(external_arg.as_ref());
        }  
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.current_dir(&self.cwd);

        let mut process = req!(command.spawn(), |err| {
            let mut quote = AnnotatedSourceSection::from_span(&self.verbatim_tag);
            quote.highlight(self.verbatim_tag.begin.byte_pos, self.verbatim_tag.end.byte_pos);
            issues.push(Issue { 
                quote, 
                title: "Failed to start external rewriter process", 
                subtext: "The verbatim tagged here will not be represented in the finished document\n\
                          because the external command given in the rewrite directive invocation could\n\
                          not be executed.", 
                severity: Severity::Error, 
                elaborations: vec![
                    Elaboration::SourceQuote(SourceQuoteElaboration { 
                        caption: "This directive invocation was applied to the verbatim.",
                        content: AnnotatedSourceSection::from_span(&self.external_cmd)
                    }),
                    Elaboration::Quote(QuoteElaboration {
                        caption: "The following error occurred after invoking the external command",
                        content: err.to_string()
                    }),
                ] 
            });
        });
        
        if let Some(mut stdin) = process.stdin.take() {
            for span in &self.src {
                req!(stdin.write_all(span.as_ref().as_bytes()), |err| {
                    let mut quote = AnnotatedSourceSection::from_span(&self.verbatim_tag);
                    quote.highlight(self.verbatim_tag.begin.byte_pos, self.verbatim_tag.end.byte_pos);
                    issues.push(Issue { 
                        quote,
                        title: "External rewriter is not accepting input",
                        subtext: "The verbatim tagged here will not be represented in the finished document\n\
                                  because the external command given in the rewrite directive invocation is not\n\
                                  reading from stdin.", 
                        severity: Severity::Error,
                        elaborations: vec![
                            Elaboration::SourceQuote(SourceQuoteElaboration { 
                                caption: "The following directive invocation acted on this verbatim...",
                                content: AnnotatedSourceSection::from_span(&self.external_cmd)
                            }),
                            Elaboration::Quote(QuoteElaboration {
                                caption: "The following error occurred while writing to the external process...",
                                content: err.to_string()
                            }),
                        ]
                    });
                });
            }
        }
        
        if let Some(mut stdout) = process.stdout.take() {
            req!(std::io::copy(&mut stdout, out), |err| { 
                let mut quote = AnnotatedSourceSection::from_span(&self.verbatim_tag);
                quote.highlight(self.verbatim_tag.begin.byte_pos, self.verbatim_tag.end.byte_pos);
                issues.push(Issue { 
                    quote,
                    title: "Failed to pipe external process output into finished document.",
                    subtext: "The verbatim tagged here could not be rewritten because an error\n\
                              ocurred while piping the process' stdout into the finished document.\n\
                              The document is now likely malformed, or at least incomplete.", 
                    severity: Severity::Error,
                    elaborations: vec![
                        Elaboration::SourceQuote(SourceQuoteElaboration { 
                            caption: "This directive invocation was applied to the verbatim.",
                            content: AnnotatedSourceSection::from_span(&self.external_cmd)
                        }),
                        Elaboration::Quote(QuoteElaboration {
                            caption: "The following error occurred while piping from the external process...",
                            content: err.to_string()
                        }),
                    ]
                });
            });
        }
        
        if let Some(mut stderr) = process.stderr.take() {
            let mut error_text = String::new();
            stderr.read_to_string(&mut error_text); // TODO
            if error_text.len() > 0 {
                let mut quote = AnnotatedSourceSection::from_span(&self.verbatim_tag);
                quote.highlight(self.verbatim_tag.begin.byte_pos, self.verbatim_tag.end.byte_pos);
                issues.push(Issue { 
                    quote,
                    title: "An external error ocurred while rewriting a verbatim.",
                    subtext: "An external process logged an error while rewriting the verbatim tagged here.",
                    severity: Severity::Warning,
                    elaborations: vec![
                        Elaboration::SourceQuote(SourceQuoteElaboration { 
                            caption: "The following directive invocation acted on this verbatim...",
                            content: AnnotatedSourceSection::from_span(&self.external_cmd)
                        }),
                        Elaboration::Quote(QuoteElaboration {
                            caption: "The following error occurred while piping from the external process...",
                            content: error_text
                        }),
                    ]
                });
            }
        }
        
        req!(process.wait(), |err| {
            let mut quote = AnnotatedSourceSection::from_span(&self.verbatim_tag);
            quote.highlight(self.verbatim_tag.begin.byte_pos, self.verbatim_tag.end.byte_pos);
            issues.push(Issue { 
                quote,
                title: "Cannot confirm completion of external process.",
                subtext: "The verbatim tagged here could not be rewritten because an error\n\
                          ocurred while piping the process' stdout into the finished document.\n\
                          The document is now likely malformed, or at least incomplete.", 
                severity: Severity::Error,
                elaborations: vec![
                    Elaboration::SourceQuote(SourceQuoteElaboration { 
                        caption: "The following directive invocation acted on this verbatim...",
                        content: AnnotatedSourceSection::from_span(&self.external_cmd)
                    }),
                    Elaboration::Quote(QuoteElaboration {
                        caption: "The following error occurred while waiting for the process...",
                        content: err.to_string()
                    }),
                ]
            });
        });
    } 
}

fn apply_embed<'a, 'b, C>(args: Vec<SourceSpan<'a>>, scope: &mut C, ctx: &mut Context<'a, 'b>) 
where C: Container<'a>
{
    let Some(external_cmd) = args.get(0).cloned() else { /* TODO: Issue */ return; };
    let external_args = Vec::from(&args[1..]);

    let value: Box<dyn Writable<'a> + 'a> = Box::new(Synthesizer { 
        external_cmd, external_args, cwd: ctx.cwd.clone() });
    scope.children_mut().push(BlockChild::HTML(HTML { value }));
}

#[derive(Debug)]
struct Synthesizer<'a> {
    pub external_args: Vec<SourceSpan<'a>>,
    pub external_cmd: SourceSpan<'a>,
    pub cwd: std::path::PathBuf
}

impl<'a> Writable<'a> for Synthesizer<'a> {
    fn write(&self, out: &mut dyn std::io::Write, issues: &mut Vec<Issue<'a>>) {
        let mut command = Command::new(self.external_cmd.as_ref()); 
        for external_arg in &self.external_args {
            command.arg(external_arg.as_ref());
        }  
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.current_dir(&self.cwd);
        
        let mut process = req!(command.spawn(), |err| {
            let quote = AnnotatedSourceSection::from_span(&self.external_cmd);
            issues.push(Issue { 
                quote, 
                title: "Failed to start external process", 
                subtext: "The content represented by this directive invocation will not be included\n\
                          in the finished document becuase the external process could not be executed.", 
                severity: Severity::Error, 
                elaborations: vec![
                    Elaboration::Quote(QuoteElaboration {
                        caption: "The following error occurred after invoking the external command",
                        content: err.to_string()
                    }),
                ] 
            });
        });

        if let Some(mut stdout) = process.stdout.take() {
            req!(std::io::copy(&mut stdout, out), |err| { 
                let quote = AnnotatedSourceSection::from_span(&self.external_cmd);
                 issues.push(Issue { 
                    quote,
                    title: "Failed to pipe external process output into finished document.",
                    subtext: "The finished document will likely be malformed, or at least incomplete.", 
                    severity: Severity::Error,
                    elaborations: vec![
                        Elaboration::Quote(QuoteElaboration {
                            caption: "The following error occurred while piping from the external process...",
                            content: err.to_string()
                        }),
                    ]
                });
            });
        }

        if let Some(mut stderr) = process.stderr.take() {
            let mut error_text = String::new();
            stderr.read_to_string(&mut error_text); // TODO
            if error_text.len() > 0 {
                let quote = AnnotatedSourceSection::from_span(&self.external_cmd);
                issues.push(Issue { 
                    quote,
                    title: "An external error occurred during synthesis",
                    subtext: "The external process logged an error while synthesizing the content\n\
                              represented by this \"embed\". The finished document will likely by\n\
                              incomplete or malformed.",
                    severity: Severity::Warning,
                    elaborations: vec![
                        Elaboration::Quote(QuoteElaboration {
                            caption: "The process logged the following error...",
                            content: error_text
                        }),
                    ]
                });
            }
        }

        req!(process.wait(), |err| {
            let quote = AnnotatedSourceSection::from_span(&self.external_cmd);
            issues.push(Issue { 
                quote,
                title: "Cannot confirm successful completion of external process.",
                subtext: "This \"make\" invocation might not have been expanded correctly.\n\
                          The finished document might be incomplete or malformed.", 
                severity: Severity::Error,
                elaborations: vec![
                    Elaboration::Quote(QuoteElaboration {
                        caption: "The following error occurred while waiting for the process...",
                        content: err.to_string()
                    }),
                ]
            });
        });
    }
}
