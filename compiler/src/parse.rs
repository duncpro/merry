/// The denormalized position of a character in some source text.
#[derive(Clone, Copy, Default, Debug)]
pub struct SourceLocation {
    pub char_pos: usize,
    pub byte_pos: usize,
    pub line_pos: usize,
    /// The number of unicode characters elapsed since the last
    /// linebreak. 
    ///
    /// Important: This value is **not** suitable for use in error messages
    /// as it is a measure of unicode characters **not** grapheme clusters.
    pub colu_pos: usize
}

/// A substring in some source text.
pub struct SourceSpan<'a> {
    source: &'a str,
    begin: SourceLocation,
    end: SourceLocation
}

impl<'a> SourceSpan<'a> {
    /// Creates a new cursor through the entirety of the source text
    /// and places it at the end of this [`SourceSpan`].
    pub fn end<'b>(&'b self) -> ForwardCursor<'a> {
        ForwardCursor { pos: self.end, source: self.source }
    }
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
/// - Methods beginning with `at` are *peeking*, they do not ever
///   advance the cursor.
/// - Methods beginning with `match` are *popping*, they advance
///   the cursor past the predicate.
#[derive(Clone)]
pub struct ForwardCursor<'a> {
    pos: SourceLocation,
    source: &'a str
}

impl<'a> ForwardCursor<'a> {
    pub fn peek(&self) -> Option<char> { self.rem().chars().next() }

    pub fn pop(&mut self) -> Option<char> { 
        let next_char = self.peek()?;
        self.advance_char_assume_next(next_char);
        return Some(next_char);
    }

    pub fn pop_while<'b>(&'b mut self, pred: impl Fn(char) -> bool) 
    -> SourceSpan<'a> 
    {
        let begin = self.pos();
        loop {
            let Some(next_char) = self.peek() else { break };
            if !pred(next_char) { break; }
            self.advance_char_assume_next(next_char);
        }
        return SourceSpan { source: self.source, begin, end: self.pos() };
    }

    /// Checks to see if `pred` occurs subsequent to the cursor.
    pub fn at_str(&self, pred: &str) -> bool {
        let end = std::cmp::min(self.source.len(),
            self.pos.byte_pos + pred.len());
        let next_str = &self.source[self.pos.byte_pos..end];
        return next_str == pred;
    }

    pub fn match_str(&mut self, pred: &str) -> bool {
        if !self.at_str(pred) { return false; }
        for next_char in pred.chars() {
            self.advance_char_assume_next(next_char);
        }
        return true;
    }

    pub fn match_char(&mut self, pred: char) -> bool {
        let Some(next_char) = self.peek() else { return false };
        if next_char != pred { return false; }
        self.advance_char_assume_next(next_char);
        return true;
    }
    
    pub fn match_scan(&mut self, predicate: impl Scan) -> bool {
        let mut tmp_cursor = self.clone();
        if !predicate.scan(&mut tmp_cursor) { return false }
        *self = tmp_cursor;
        return true;        
    }    
    
    /// Advances the cursor past the string `s`. If `s` is not subsequent
    /// to the cursor, this procedure panics and does not advance the cursor.
    ///
    /// Use [`match_str`] over this procedure when `s` isn't necessarily next.
    pub fn advance_str(&mut self, s: &str) { assert!(self.match_str(s)) }

    /// Advances the cursor past the next linebreak and returns a [`SourceSpan`] containing 
    /// the intermediate text (excludes the terminating linebreak). EOF is considered
    /// a terminating linebreak.
    pub fn pop_line<'b>(&'b mut self) -> SourceSpan<'a> {
        let begin = self.pos();
        let mut end = self.pos();
        loop {
            let Some(next_char) = self.pop() else { break };
            if next_char == '\n' { break };
            end = self.pos();
        }
        return SourceSpan { source: self.source, begin, end };
    }

    /// Advances the cursor past the next `count` unicode characters.
    pub fn discard_n_chars(&mut self, count: usize) {
        for _ in 0..count { self.discard_char(); }
    }

    /// Advances the cursor past the next unicode character.
    /// Panics if the cursor is already at the end of the source text.
    pub fn discard_char(&mut self) {
        let next_char = self.rem().chars().next().unwrap();
        self.advance_char_assume_next(next_char);
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

    fn advance_char_assume_next(&mut self, next_char: char) {
        if next_char == '\n' {
            self.pos.line_pos += 1;
            self.pos.colu_pos = 0;
        } else {
            self.pos.colu_pos += 1;
        }
        self.pos.char_pos += 1;
        self.pos.byte_pos += next_char.len_utf8();
    }
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

