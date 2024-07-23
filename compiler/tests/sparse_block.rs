use merry_compiler::assert_matches;
use merry_compiler::ltree::{make_ltree, verify_ltree};
use merry_compiler::ltree;

/// This test verifies that [`make_ltree`] constructs a correct *LTree* given
/// a document containing a sparse *Block*. That is, a block containing
/// at least one blank but non-terminal line.
///
/// The following properties must be satisfied...
/// - Every *Line* in the document appears in the tree.
/// - There are no errenous nodes in the tree, for instance a `VerticalSpace`
///   where none actually exists.
#[test]
pub fn test_make_ltree() {
    let source = std::fs::read_to_string("tests/sparse_block.md2").unwrap();
    let ltree = make_ltree(&source);
    
    assert_eq!(ltree.children.len(), 1);

    assert_matches!(ltree.children[0], ltree::ast::RootChild::Block(ref block0));
    assert_eq!(block0.children.len(), 10);
    assert_matches!(block0.children[0], ltree::ast::BlockChild::Line(_));
    assert_matches!(block0.children[1], ltree::ast::BlockChild::Line(_));
    assert_matches!(block0.children[2], ltree::ast::BlockChild::VerticalSpace(_));
    assert_matches!(block0.children[3], ltree::ast::BlockChild::Line(_));
    assert_matches!(block0.children[4], ltree::ast::BlockChild::Line(_));
    assert_matches!(block0.children[5], ltree::ast::BlockChild::VerticalSpace(_));
    assert_matches!(block0.children[6], ltree::ast::BlockChild::Line(_));
    assert_matches!(block0.children[7], ltree::ast::BlockChild::Line(_));
    assert_matches!(block0.children[8], ltree::ast::BlockChild::Line(_));
    assert_matches!(block0.children[9], ltree::ast::BlockChild::Line(_));
}

#[test]
pub fn test_verify_ltree() {
    let source = std::fs::read_to_string("tests/sparse_block.md2").unwrap();
    let ltree = make_ltree(&source);
    let warnings = verify_ltree(&ltree);
     assert!(warnings.is_empty());
}
