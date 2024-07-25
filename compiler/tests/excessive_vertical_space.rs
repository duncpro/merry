use merry_compiler::assert_matches;
use merry_compiler::ltree::{AnyLTreeWarning, make_ltree, verify_ltree};
use merry_compiler::report::print_issue;

#[test]
pub fn test_verify_ltree() {
    let source = std::fs::read_to_string("tests/excessive_vertical_space.md2").unwrap();
    let ltree = make_ltree(&source);
    let warnings = verify_ltree(&ltree);
    for any_warning in &warnings {
        print_issue((*any_warning).into(), "tests/excessive_vertical_space.md2");
        assert_matches!(any_warning, AnyLTreeWarning::ExcessiveVerticalSpace(_));
    }
    assert_eq!(warnings.len(), 3);
}
