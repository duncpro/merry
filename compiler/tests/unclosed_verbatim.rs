use merry_compiler::assert_matches;
use merry_compiler::ltree::{AnyLTreeIssue, make_ltree, verify_ltree};
use merry_compiler::report::print_issue;

#[test]
pub fn test_verify_ltree() {
    let source = std::fs::read_to_string("tests/unclosed_verbatim.md2").unwrap();
    let ltree = make_ltree(&source);
    let warnings = verify_ltree(&ltree);
    for any_warning in &warnings {
        print_issue(&(*any_warning).into(), "tests/unclosed_verbatim.md2");
        assert_matches!(any_warning, AnyLTreeIssue::UnclosedVerbatim(_));
    }
    assert_eq!(warnings.len(), 1);
}
