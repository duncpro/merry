use merry_compiler::assert_matches;
use merry_compiler::builtins::apply_builtin_directives;
use merry_compiler::ltree::{make_ltree, verify_ltree};
use merry_compiler::mtree::{self, make_mtree, verify_mtree};
use merry_compiler::report::{print_issue, Issue};
use merry_compiler::rewrite::rewrite_mtree;

#[test]
pub fn test_make_ltree() {
    let source = std::fs::read_to_string("tests/invocation.md2").unwrap();
    let ltree = make_ltree(&source);
}

#[test]
pub fn test_verify_ltree() {
    let source = std::fs::read_to_string("tests/invocation.md2").unwrap();
    let ltree = make_ltree(&source);
    let issues = verify_ltree(&ltree);
    for issue in issues {
        print_issue(&issue.into(), "tests/invocation.md2");
    }
}

#[test]
pub fn test_make_mtree() {
    let source = std::fs::read_to_string("tests/invocation.md2").unwrap();
    let ltree = make_ltree(&source);
    let mut mtree = make_mtree(&ltree);
    let mut issues2: Vec<Issue> = Vec::new();
    apply_builtin_directives(&mut mtree.block, &mut issues2);
    let issues = verify_mtree(&mtree);
    for issue in issues {
        print_issue(&issue.into(), "tests/invocation.md2");
    }
    println!("{:#?}", mtree);
}

