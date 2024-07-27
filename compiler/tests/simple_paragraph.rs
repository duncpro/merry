use merry_compiler::assert_matches;
use merry_compiler::ltree::{make_ltree, verify_ltree};
use merry_compiler::mtree::make_mtree;

#[test]
pub fn test_make_ltree() {
    let source = std::fs::read_to_string("tests/simple_paragraph.md2").unwrap();
    let ltree = make_ltree(&source);
    println!("{:#?}", ltree);
}

#[test]
pub fn test_verify_ltree() {
    let source = std::fs::read_to_string("tests/simple_paragraph.md2").unwrap();
    let ltree = make_ltree(&source);
    let warnings = verify_ltree(&ltree);
    assert_eq!(warnings.len(), 0);
}

#[test]
pub fn test_make_mtree() {
    let source = std::fs::read_to_string("tests/simple_paragraph.md2").unwrap();
    let ltree = make_ltree(&source);
    let mtree = make_mtree(&ltree);
    println!("{:#?}", mtree);
}
