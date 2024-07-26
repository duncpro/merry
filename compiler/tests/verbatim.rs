use merry_compiler::assert_matches;
use merry_compiler::ltree::{make_ltree, verify_ltree};

#[test]
pub fn test_make_ltree() {
    let source = std::fs::read_to_string("tests/verbatim.md2").unwrap();
    let ltree = make_ltree(&source);
}

#[test]
pub fn test_verify_ltree() {
    let source = std::fs::read_to_string("tests/verbatim.md2").unwrap();
    let ltree = make_ltree(&source);
    let warnings = verify_ltree(&ltree);
    assert_eq!(warnings.len(), 0);
}
