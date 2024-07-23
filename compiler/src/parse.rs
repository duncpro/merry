/// The denormalized position of a character in some source text.
#[derive(Clone, Copy, Default, Debug)]
pub struct SourceLocation {
    pub byte_pos: usize,
    pub line_pos: usize,
    pub colu_pos: usize,
}

/// A substring in some source text. 
#[derive(Clone, Copy)]
pub struct SourceSpan<'a> {
    pub source: &'a str,
    pub begin: SourceLocation,
    pub end: SourceLocation
}

impl<'a> AsRef<str> for SourceSpan<'a> {
    fn as_ref(&self) -> &str {
        let begin = self.begin.byte_pos;
        let end = self.end.byte_pos;
        return &self.source[begin..end];
    }
}

impl<'a> std::fmt::Debug for SourceSpan<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.as_ref())
    }
}

/// A cursor in some source text. 
///
/// - Methods beginning with `at` are *peeking*, they do not advance the cursor.
/// - Methods beginning with `match` are *popping*, they advance the cursor past
///   the predicate.
///
/// Unlike the string iterators found in the standard library, this cursor tracks
/// the unicode column index in addition to the line number and byte index.
#[derive(Clone)]
pub struct ForwardCursor<'a> {
    pos: SourceLocation,
    pub source: &'a str
}

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

impl<'a> ForwardCursor<'a> {
    /// Advances the cursor until a non-space character is encountered.
    pub fn pop_spaces<'b>(&'b mut self) -> usize {
        let mut count: usize = 0;
        loop {
            if self.rem().chars().next() != Some(' ') { break; }
            self.pos.byte_pos += 1;
            self.pos.colu_pos += 1;
            count += 1;
        }
        return count;
    }

    pub fn peek_spaces<'b>(&'b self) -> usize { self.clone().pop_spaces() }

    /// Returns true if and only if `pred` is subsequent to the cursor.
    pub fn at(&self, pred: &str) -> bool { self.rem().starts_with(pred) }

    /// Advances the cursor past `pred` and returns true. Otherwise, if `pred` is not subsequent
    /// to the cursor, returns false without advancing.
    ///
    /// This procedure will panic if `pred` contains a linebreak. To match a linebreak,
    /// use the explicit [`match_linebreak`] procedure instead.
    pub fn match_symbol(&mut self, pred: &str) -> bool {
        assert!(!pred.contains('\n'));
        if !self.at(pred) { return false; }
        for grapheme in pred.graphemes(true) {
            self.pos.byte_pos += grapheme.len();
            self.pos.colu_pos += grapheme.width();
        }
        return true;
    }

    pub fn match_linebreak(&mut self) -> bool {
        if !self.rem().starts_with('\n') { return false; }
        self.pos.byte_pos += 1;
        self.pos.line_pos += 1;
        self.pos.colu_pos = 0;
        return true;
    }
    
    pub fn match_scan<'b>(&'b mut self, predicate: impl Scan) -> Option<SourceSpan<'a>> {
        let mut tmp_cursor = self.clone();
        if !predicate.scan(&mut tmp_cursor) { return None }
        let span = SourceSpan { source: self.source, begin: self.pos(), end: tmp_cursor.pos() };
        *self = tmp_cursor;
        return Some(span);        
    }

    pub fn at_scan(&mut self, predicate: impl Scan) -> Option<SourceSpan<'a>> {
        let mut tmp_cursor = self.clone();
        if !predicate.scan(&mut tmp_cursor) { return None; }
        let span = SourceSpan { source: self.source, begin: self.pos(), end: tmp_cursor.pos() };
        return Some(span);
    }

    /// Advances the cursor past the next linebreak and returns a [`SourceSpan`] containing 
    /// the intermediate text (excludes the terminating linebreak). EOF is considered
    /// a terminating linebreak.
    pub fn pop_line<'b>(&'b mut self) -> SourceSpan<'a> {
        let mut span_end = self.pos();
        let mut self_end = self.pos();
        for grapheme in self.rem().graphemes(true) {
            self_end.byte_pos += grapheme.len();
            if grapheme == "\n" {
                self_end.line_pos += 1;
                self_end.colu_pos = 0;
                break;
            }
            self_end.colu_pos += grapheme.width();
            span_end = self_end;
        }
        let begin = self.pos();
        self.pos = self_end;
        return SourceSpan { source: self.source, begin, end: span_end };
    }

    pub fn pos(&self) -> SourceLocation { self.pos }

    pub fn is_end(&self) -> bool {
        return self.pos.byte_pos == self.source.len()
    }

    pub fn new(source: &'a String) -> Self {
        Self { source, pos: SourceLocation::default() }
    }

    // Internal
    
    fn rem(&self) -> &str { &self.source[self.pos.byte_pos..] }
}

pub trait Scan {
    fn scan<'a>(&self, cursor: &mut ForwardCursor<'a>) -> bool;
}

pub struct Scanner<C> where C: Fn(&mut ForwardCursor) -> bool {
     closure: C 
}

impl<C> Scan for Scanner<C> where C: Fn(&mut ForwardCursor) -> bool {
    fn scan<'a>(&self, cursor: &mut ForwardCursor<'a>) -> bool {
        return (self.closure)(cursor);
    }
}

impl<C> Scanner<C> where C: Fn(&mut ForwardCursor) -> bool {
    pub fn new(closure: C) -> Self { Self { closure } }
}

#[macro_export]
macro_rules! scanner {
    ($(#[ $attr:meta ])* $func_name:ident ($($param:ident : $type:ty),*) |$cursor:ident| $block:block) => {
        $(#[ $attr ])*
        pub fn $func_name($($param: $type),*) -> impl crate::parse::Scan {
            crate::parse::Scanner::new(move |$cursor| $block)
        }
    };
}

