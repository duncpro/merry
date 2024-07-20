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
    source: &'a String,
    begin: SourceLocation,
    /// The length of the [`SourceSpan`] measured in bytes.
    length: usize
}

impl<'a> SourceSpan<'a> {
    /// Computes the column index of the *cell* immediately after the *last*
    /// character within this [`SourceSpan`]. If the [`SourceSpan`] is empty then this is
    /// simply the column index of the start position.
    ///
    /// Note that this is the column index of the subsequent cell, **not** the
    /// subsequence character. It **will not** correspond to an actual position in
    /// the document if the next character is a linebreak.
    ///
    /// A cell's column index is the number of unicode characters preceding the cell.
    ///
    /// Important: This value is not suitable for use in error messages
    /// as it is a measure of unicode characters and **not** grapheme
    /// clusters.
    pub fn end_col(self) -> usize {
        let mut col: usize = self.begin.colu_pos;
        for ch in self.as_slice().chars() {
            if ch == '\n' { col = 0; continue; }
            col += 1;
        }
        return col;
    }

    pub fn as_slice(&self) -> &str {
        let begin = self.begin.byte_pos;
        let end = begin + self.length;
        return &self.source[begin..end];
    }
}

impl<'a> std::fmt::Debug for SourceSpan<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.as_slice())
    }
}

/// A cursor in some source text. 
/// - Methods beginning with `at` are *peeking*, they do not ever
///   advance the cursor.
/// - Methods beginning with `match` are *popping*, they advance
///   the cursor past the predicate.
pub struct Cursor<'a> {
    pos: SourceLocation,
    source: &'a String
}

impl<'a> Cursor<'a> {
    pub fn peek_while<'b>(&'b self, pred: impl Fn(char) -> bool)
    -> SourceSpan<'a>
    {
        let begin = self.pos();
        let mut length: usize = 0;
        for ch in self.source[begin.byte_pos..].chars() {
            if !(pred)(ch) { break; }
            length += ch.len_utf8();
        }
        return SourceSpan { source: self.source, begin, length }
    }

    /// Checks to see if `pred` occurs subsequent to the cursor.
    pub fn at_str(&self, pred: &str) -> bool {
        let end = std::cmp::min(self.source.len(),
            self.pos.byte_pos + pred.len());
        let next_str = &self.source[self.pos.byte_pos..end];
        return next_str == pred;
    }

    /// Determines whether the only characters remaining before the
    /// next linebreak or EOF are spaces. If true, advances the cursor 
    /// past the linebreak.
    ///
    /// If the cursor has exhausted the source text then true is returned.
    pub fn match_blank_line(&mut self) -> bool {
        let mut space_count: usize = 0;
        let mut lb_terminated = false;
        for ch in self.source[self.pos.byte_pos..].chars() {
            if ch == '\n' { lb_terminated = true; break; }
            if ch == ' ' { space_count += 1; continue; }
            return false;
        }
        for _ in 0..space_count { self.advance_char_assume_next(' '); }
        if lb_terminated { self.advance_char_assume_next('\n') }
        return true;
    }

    pub fn match_str(&mut self, pred: &str) -> bool {
        if !self.at_str(pred) { return false; }
        for next_char in pred.chars() {
            self.advance_char_assume_next(next_char);
        }
        return true;
    }

    /// Advances the cursor past the next linebreak and returns a [`SourceSpan`] 
    /// containing the intermediate text (excludes the terminating linebreak).
    /// In this case EOF is treated as a terminating linebreak.
    pub fn pop_line<'b>(&'b mut self) -> SourceSpan<'a> {
        let begin = self.pos();
        let mut length: usize = 0;
        for ch in self.source[begin.byte_pos..].chars() {
            self.advance_char_assume_next(ch);
            if ch == '\n' { break; }
            length += ch.len_utf8();
        }
        return SourceSpan { source: self.source, begin, length };
    }

    /// Advances the cursor past the next `count` unicode characters.
    pub fn advance_n_chars(&mut self, count: usize) {
        for _ in 0..count { self.advance_char(); }
    }

    /// Advances the cursor past the next unicode character.
    /// Panics if the cursor is already at the end of the source text.
    pub fn advance_char(&mut self) {
        let next_char = self.source[self.pos.byte_pos..]
            .chars().next().unwrap();
        self.advance_char_assume_next(next_char);
    }

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

    pub fn pos(&self) -> SourceLocation { self.pos }

    pub fn is_end(&self) -> bool {
        return self.pos.byte_pos == self.source.len()
    }

    pub fn new(source: &'a String) -> Self {
        Self { source, pos: SourceLocation::default() }
    }
}
