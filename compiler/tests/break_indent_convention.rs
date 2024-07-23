use merry_compiler::assert_matches;
use merry_compiler::ltree::{AnyLTreeWarning, make_ltree, verify_ltree};
use merry_compiler::report::print_issue;

#[test]
pub fn test_verify_ltree() {
    let source = std::fs::read_to_string("tests/break_indent_convention.md2").unwrap();
    let ltree = make_ltree(&source);
    let warnings = verify_ltree(&ltree);
    assert_eq!(warnings.len(), 2);
    assert_matches!(warnings[0], AnyLTreeWarning::InsufficientIndent(warning0));
    assert_matches!(warnings[1], AnyLTreeWarning::ExcessiveIndent(warning1));
    print_issue(warning0.into(), "tests/break_indent_convention.md2");
    print_issue(warning1.into(), "tests/break_indent_convention.md2");
}
