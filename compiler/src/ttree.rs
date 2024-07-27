use std::iter::Peekable;

use crate::scan::{SourceSpan, SourceLocation, ForwardCursor};

pub mod ast {
    use crate::scan::SourceSpan;

    #[derive(Debug)]
    pub struct TrailingQualifier<'a> {
        pub tags: Vec<Tag<'a>>,
        pub close: Option<SourceSpan<'a>>
    }

    #[derive(Debug)]
    pub enum Tag<'a> {
        Split(/* first part */ SourceSpan<'a>),
        Unsplit(SourceSpan<'a>)
    }

    #[derive(Debug)]
    #[derive(Clone, Copy)]
    pub enum DelimiterKind {
        Asterisk,
        Underscore,
        Tilde
    }

    #[derive(Debug)]
    pub struct DelimitedText<'a> {
        pub delim_kind: DelimiterKind,
        pub open: SourceSpan<'a>,
        pub close: Option<SourceSpan<'a>>,
        pub child_root: Root<'a>,
        pub trailing_qualifier: Option<TrailingQualifier<'a>>
     }

    #[derive(Debug)]
    pub struct BracketedText<'a> {
        pub open: SourceSpan<'a>,
        pub close: Option<SourceSpan<'a>>,
        pub child_root: Root<'a>,
        pub trailing_qualifier: Option<TrailingQualifier<'a>>
    }

    #[derive(Debug)]
    pub struct PlainText<'a> { pub span: SourceSpan<'a> }

    #[derive(Debug)]
    pub enum AnyText<'a> {
        Plain(PlainText<'a>),
        Delimited(DelimitedText<'a>),
        InlineVerbatim(InlineVerbatim<'a>),
        ImplicitSpace(ImplicitSpace),
        Bracketed(BracketedText<'a>)
    }

    #[derive(Debug)]
    pub struct InlineVerbatim<'a> { 
        span: SourceSpan<'a>,
    }

    #[derive(Debug)]
    pub struct ImplicitSpace;

    #[derive(Debug)]
    pub struct Root<'a> {
        pub children: Vec<AnyText<'a>>
    }
}

pub fn make_ttree<'a, 'b>(lines: &'a [SourceSpan<'b>]) -> ast::Root<'b> {
    let line_cursor = lines[0].begin();
    let mut ctx: ParseContext<'a, 'b> = ParseContext { lines, next_line_i: 1, line_cursor };
    return parse_root(&mut ctx, None);
}


struct ParseContext<'a, 'b> {
    lines: &'a [SourceSpan<'b>],
    next_line_i: usize,
    line_cursor: ForwardCursor<'b>
}

#[allow(unused_assignments)]
fn parse_root<'a, 'b>(ctx: &mut ParseContext<'a, 'b>, stop_at: Option<&str>) -> ast::Root<'b>
{
    let mut children: Vec<ast::AnyText<'b>> = Vec::new();

    let mut pt_begin: Option<SourceLocation> = None;
    macro_rules! push_pt { () => {
        if let Some(begin) = pt_begin {
            children.push(ast::AnyText::Plain(ast::PlainText { span: SourceSpan { 
                begin, end: ctx.line_cursor.pos(), source: ctx.line_cursor.source } }));
            pt_begin = None;
        }
    }}
    
    loop {
        if is_totally_exhausted(ctx) { break; }
        if is_line_exhausted(ctx) {
            push_pt!();
            children.push(ast::AnyText::ImplicitSpace(ast::ImplicitSpace)); 
            advance_line(ctx);
            continue;
        }
        if let Some(symbol) = stop_at { 
            if ctx.line_cursor.at_symbol(symbol) {
                break; 
            }
        }
        if let Some(delim_kind) = at_delim(&ctx.line_cursor) {
            push_pt!();
            let open = ctx.line_cursor.match_symbol(spell_delim(delim_kind)).unwrap();
            let child_root = parse_root(ctx, Some(spell_delim(delim_kind)));
            let close = ctx.line_cursor.match_symbol(spell_delim(delim_kind));
            let trailing_qualifier = parse_trailing_qualifier(ctx);
            let delim_text = ast::DelimitedText { delim_kind, open,
                child_root, close, trailing_qualifier };
            children.push(ast::AnyText::Delimited(delim_text));
            continue;
        }
        if ctx.line_cursor.at_symbol("[") {
            push_pt!();
            let open = ctx.line_cursor.match_symbol("[").unwrap();
            let child_root = parse_root(ctx, Some("]"));
            let close = ctx.line_cursor.match_symbol("]");
            let trailing_qualifier = parse_trailing_qualifier(ctx);
            let brack_text = ast::BracketedText { open, close, child_root,
                trailing_qualifier };
            children.push(ast::AnyText::Bracketed(brack_text));
            continue;
        }
        // TODO: Match inline verbatim start
        // Otherwise this must be plain-text.
        if pt_begin.is_none() { pt_begin = Some(ctx.line_cursor.pos()) }
        ctx.line_cursor.pop_grapheme().unwrap();
    }
    push_pt!();
    return ast::Root { children }
}

#[allow(unused_assignments)]
fn parse_trailing_qualifier<'a, 'b>(ctx: &mut ParseContext<'a, 'b>)
-> Option<ast::TrailingQualifier<'b>>
{
    if ctx.line_cursor.match_symbol("{").is_none() { return None; }
    let mut tags: Vec<ast::Tag<'b>> = Vec::new();
    let mut current_tag_begin: Option<SourceLocation> = None;
    macro_rules! push_tag { ($is_split:expr) => {
        if let Some(begin) = current_tag_begin {
            if begin.byte_pos < ctx.line_cursor.pos().byte_pos {
                let span = SourceSpan { source: ctx.line_cursor.source, begin,
                    end: ctx.line_cursor.pos() };
                let tag = match $is_split {
                    true => ast::Tag::Split(span),
                    false => ast::Tag::Unsplit(span)
                };
                tags.push(tag);
            }
            current_tag_begin = None;
        }
    }}
    let mut close: Option<SourceSpan<'b>> = None;
    loop {
        if is_totally_exhausted(ctx) { break; };
        if is_line_exhausted(ctx) { 
            push_tag!(true);
            advance_line(ctx); 
            continue; 
        }
        if ctx.line_cursor.match_symbol(" ").is_some() { push_tag!(true); continue; }
        if ctx.line_cursor.match_symbol(",").is_some() { push_tag!(false); continue; }
        if ctx.line_cursor.at_symbol("}") {
            push_tag!(false);
            close = Some(ctx.line_cursor.match_symbol("}").unwrap());
            break;
        }
        if current_tag_begin.is_none() { current_tag_begin = Some(ctx.line_cursor.pos()); }
        // Otherwise, this is a grapheme within a tag.
        ctx.line_cursor.pop_grapheme().unwrap();
    }

    return Some(ast::TrailingQualifier { tags, close });
}

fn at_delim<'b>(cursor: &ForwardCursor<'b>) -> Option<ast::DelimiterKind> {
    let kind = match cursor.peek_char()? {
        '*' => ast::DelimiterKind::Asterisk,
        '_' => ast::DelimiterKind::Underscore,
        '~' => ast::DelimiterKind::Tilde,
         _  => return None
    };
    return Some(kind);
}


fn is_line_exhausted<'a, 'b>(ctx: &ParseContext<'a, 'b>) -> bool 
{
    return ctx.line_cursor.is_end();
}

fn is_totally_exhausted<'a, 'b>(ctx: &ParseContext<'a, 'b>) -> bool {
    return is_line_exhausted(ctx) && ctx.next_line_i == ctx.lines.len();
}

fn advance_line<'a, 'b>(ctx: &mut ParseContext<'a, 'b>) 
{
    ctx.line_cursor = ctx.lines[ctx.next_line_i].begin();
    ctx.next_line_i += 1;
}

fn spell_delim(delim: ast::DelimiterKind) -> &'static str {
    match delim {
        ast::DelimiterKind::Asterisk => "*",
        ast::DelimiterKind::Underscore => "_",
        ast::DelimiterKind::Tilde => "~",
    }
}

