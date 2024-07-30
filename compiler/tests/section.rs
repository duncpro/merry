use merry_compiler::assert_matches;
use merry_compiler::ltree::{make_ltree, verify_ltree};
use merry_compiler::mtree::{make_mtree, verify_mtree};
use merry_compiler::report::print_issue;

#[test]
pub fn test_make_ltree() {
    let source = std::fs::read_to_string("tests/section.md2").unwrap();
    let ltree = make_ltree(&source);
}

#[test]
pub fn test_verify_ltree() {
    let source = std::fs::read_to_string("tests/section.md2").unwrap();
    let ltree = make_ltree(&source);
    let issues = verify_ltree(&ltree);
    for issue in issues {
        print_issue(&issue.into(), "tests/section.md2");
    }
}

#[test]
pub fn test_make_mtree() {
    let source = std::fs::read_to_string("tests/section.md2").unwrap();
    let ltree = make_ltree(&source);
    let mtree = make_mtree(&ltree);
    let issues = verify_mtree(&mtree);
    for issue in issues {
        print_issue(&issue.into(), "tests/section.md2");
    }
    println!("{:#?}", mtree);
}

