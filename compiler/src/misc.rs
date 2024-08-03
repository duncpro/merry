#[macro_export]
macro_rules! assert_matches {
    ($e:expr, $p:pat) => {
        let $p = $e 
        else { panic!("{:?} did not match pattern {}", $e, core::stringify!($p)); };
    }
}

pub mod ansi {
    pub const FG_DEFAULT: &str = "\x1b[39m";
    pub const FG_RED: &str = "\x1b[31m";
    pub const FG_YELLOW: &str = "\x1b[33m";
    pub const FG_GREY: &str = "\x1b[90m";
    pub const BG_DEFAULT: &str = "\x1b[49m";
    pub const BG_RED: &str = "\x1b[41m";
    pub const BG_YELLOW: &str = "\x1b[43m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DEFAULT_TEXT_STYLE: &str = "\x1b[22m";
}

// TODO: Deprecate and remove. Needless allocation. See use sites.
pub fn pad(text: &str, max_len: usize) -> String {
    assert!(text.len() <= max_len);
    let mut padded = text.to_string();
    while padded.len() < max_len {
        padded.push(' ');
    }
    return padded;
}

