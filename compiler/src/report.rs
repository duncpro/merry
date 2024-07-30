//! There are a variety of syntax and semantic errors which can be present within
//! an md2 source file. This module provides facilities for displaying rich
//! and nicely formatted error messages for all these.

use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use crate::misc::pad;
use crate::misc::ansi::*;
use crate::scan::SourceSpan;

/// A reference to zero or more consecutive lines in a source file along with annotations.
pub struct AnnotatedSourceSection<'a> {
    source: &'a str,
    pub first_line_no: usize,
    last_line_no: usize,
    first_line_begin_bpos: usize,
    last_line_end_bpos: usize,
    barriers: HashMap<usize /* before line no */, Barrier>,
    highlights: BTreeMap<usize /* begin byte pos */, Highlight>,
    pub limit: Option<usize>
}

pub struct Highlight { byte_length: usize }

pub enum BarrierStyle {
    Ruler(/* col: */ usize, /* width: */ usize),
    Placeholder
}

pub struct Barrier { 
    note: &'static str,
    style: BarrierStyle,
}

impl<'a> AnnotatedSourceSection<'a> {
    /// Constructs the minimal [`AnnotatedSourceSection`] containing the given `span`.
    /// This likely encompasses more source text than the `span` itself since unlike
    /// [`SourceSpan`] an [`AnnotatedSourceSection`] contains only full width lines
    /// from the source text.
    pub fn from_span<'b>(span: &'b SourceSpan<'a>) -> Self {
        let first_line_begin_bpos = span.source[0..span.begin.byte_pos].bytes()
            .rposition(|b| b == '\n' as u8)
            .map(|bpos| bpos + 1)
            .unwrap_or(0);

        let last_line_end_bpos = span.source[span.end.byte_pos..].bytes()
                .position(|b| b == '\n' as u8)
                .map(|bpos| bpos + span.end.byte_pos)
                .unwrap_or(span.source.len());
        
        return AnnotatedSourceSection { 
            source: span.source, 
            first_line_no: span.begin.line_pos,
            last_line_no: span.end.line_pos, 
            first_line_begin_bpos, last_line_end_bpos,
            barriers: HashMap::new(),
            highlights: BTreeMap::new(),
            limit: Option::None
        }
    }

    /// Extends the quote to include the previous lines in the source text as well.
    /// If there are not previous lines in the source text this is a no-op.
    /// 
    /// Motivation: An extended quote provides more context for the programmer
    /// when they read an error report.     
    pub fn extend_up(&mut self, line_count: usize) { 
        for _ in 0..line_count {
            if self.first_line_no == 0 { return; }
            self.first_line_no -= 1;
            self.first_line_begin_bpos = 
                self.source[0..(self.first_line_begin_bpos - 1)].bytes()
                    .rposition(|b| b == '\n' as u8)
                    .map(|bpos| bpos + 1)
                    .unwrap_or(0);
        }
    }

    /// Extends the quote to include the next line in the source text as well.
    /// If there are no more lines in the source text this is a no-op.
    ///
    /// Motivation: An extended quote provides more context for the programmer
    /// when they read an error report.     
    pub fn extend_down(&mut self) { 
        if self.last_line_end_bpos >= self.source.len() { return; }
        self.last_line_no += 1;
        self.last_line_end_bpos =
            self.source[(self.last_line_end_bpos + 1)..].bytes()
                .position(|b| b == '\n' as u8)
                .map(|bpos| bpos + self.last_line_end_bpos + 1)
                .unwrap_or(self.source.len());
    }

    pub fn place_barrier_before(&mut self, line_no: usize, style: BarrierStyle, note: &'static str) 
    {
        self.barriers.insert(line_no, Barrier { note, style });
    }

    pub fn highlight(&mut self, begin_bpos: usize, end_bpos: usize) {
        assert!(begin_bpos >= self.first_line_begin_bpos);
        assert!(end_bpos <= self.last_line_end_bpos);
        assert!(self.highlights.range(begin_bpos..end_bpos).next().is_none());
        let byte_length = end_bpos - begin_bpos;
        self.highlights.insert(begin_bpos, Highlight { byte_length });
    }
}

fn print_quote(quote: &AnnotatedSourceSection, highlight_color: &'static str) {
    let quote_text = &quote.source[quote.first_line_begin_bpos..quote.last_line_end_bpos];
    let line_no_len = (quote.last_line_no + 1).to_string().len();
    
    let mut line_no = quote.first_line_no;
    let mut byte_pos: usize = quote.first_line_begin_bpos;
    let mut stop_highlight_at: Option<usize> = None;
    for line_text in quote_text.lines() {
        if let Some(barrier) = quote.barriers.get(&line_no) {
            for _ in 0..(line_no_len + 2) { print!(" "); }
            match barrier.style {
                BarrierStyle::Ruler(col, width) => {
                    for _ in 0..col { print!(" "); }
                    for _ in 0..width { print!("-"); }
                    println!(" {}", barrier.note);
                },
                BarrierStyle::Placeholder => println!("** {} **", barrier.note),
            }
        }
        // TODO: Replace pad() with non-allocating for loop
        print!("{}| ", pad(&(line_no + 1).to_string(), line_no_len));
        print!("{}", FG_GREY);
        let mut stdout_handle = std::io::stdout().lock();
        for byte in line_text.bytes() {
            if let Some(highlight) = quote.highlights.get(&byte_pos) {
                stdout_handle.write_all(highlight_color.as_bytes()).unwrap();
                stop_highlight_at = Some(byte_pos + highlight.byte_length);
            }
            if let Some(stop_pos) = stop_highlight_at {
                if stop_pos == byte_pos { 
                    stdout_handle.write_all(BG_DEFAULT.as_bytes()).unwrap();
                    stop_highlight_at = None;
                }
            }
            stdout_handle.write(&[byte]).unwrap();
            byte_pos += 1;
        }
        if let Some(stop_pos) = stop_highlight_at {
            if stop_pos == byte_pos {
                stdout_handle.write_all(BG_DEFAULT.as_bytes()).unwrap();
                stop_highlight_at = None;
            }
        }
        std::mem::drop(stdout_handle);
        print!("\n{}", FG_DEFAULT); // TODO: WindowsLinebreaks
        if quote.limit == Some(line_no) { break; }
        line_no += 1;
        byte_pos += 1; // TODO WindowsLinebreaks
    }
    println!("{}", BG_DEFAULT);

    if line_no < quote.last_line_no {
        println!("and {} more line(s)...", quote.last_line_no - line_no);
    }
}

pub enum Severity {
    /// A syntactic or semantic issue has `Error` severity if the source-text is
    /// likely to be misinterpreted by the compiler. 
    Error,

    /// A syntactic or semantic issue has `Warning` severity if the source-text is
    /// comprehended by the compiler but breaks stylistic convention.
    Warning
}

pub struct Issue<'a> {
    pub quote: AnnotatedSourceSection<'a>,
    pub title: &'static str,
    pub subtext: &'static str,
    pub severity: Severity,
}

pub fn print_issue<'a>(issue: &Issue<'a>, source_name: &str) {    
    print!("{}", BOLD);
    match issue.severity {
        Severity::Error => print!("{}Error{}: ", FG_RED, FG_DEFAULT),
        Severity::Warning => print!("{}Warning{}: ", FG_YELLOW, FG_DEFAULT),
    }
    print!("{}", issue.title);
    print!("{}", DEFAULT_TEXT_STYLE);
    println!();
    println!("{}", issue.subtext);

    println!();
    println!("at {}:{}", source_name, issue.quote.first_line_no + 1);
    let highlight_color = match issue.severity {
        Severity::Error => BG_RED,
        Severity::Warning => BG_YELLOW,
    };
    print_quote(&issue.quote, highlight_color);
    println!();
}
