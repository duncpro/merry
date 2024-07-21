use merry_compiler::assert_matches;
use merry_compiler::ltree::make_ltree;
use merry_compiler::ltree;

/// This test verifies that [`make_ltree`] constructs a correct *LTree* given
/// a document containing nested *Blocks*.
///
/// The following properties must be satisfied...
/// - The hierarchy of the tree matches the indentation of the document.
/// - Every *Line* in the document appears in the tree.
/// - There are no errenous nodes in the tree, for instance a `VerticalSpace`
///   where none actually exists.

#[test]
pub fn test_make_ltree() {
    let source = std::fs::read_to_string("tests/simple_blocks.md2").unwrap();
    let ltree = make_ltree(&source);

    assert_eq!(ltree.children.len(), 1);

    assert_matches!(ltree.children[0], ltree::ast::RootChild::Block(ref block0));    
    assert_eq!(block0.children.len(), 7);
    assert_matches!(block0.children[0], ltree::ast::BlockChild::Line(_));
    assert_matches!(block0.children[1], ltree::ast::BlockChild::Line(_));
    assert_matches!(block0.children[2], ltree::ast::BlockChild::Line(_));
    assert_matches!(block0.children[3], ltree::ast::BlockChild::Line(_));
    assert_matches!(block0.children[4], ltree::ast::BlockChild::Line(_));
    assert_matches!(block0.children[5], ltree::ast::BlockChild::VerticalSpace(_));

    assert_matches!(block0.children[6], ltree::ast::BlockChild::Block(ref block1));
    assert_eq!(block1.children.len(), 6);
    assert_matches!(block1.children[0], ltree::ast::BlockChild::Line(_));
    assert_matches!(block1.children[1], ltree::ast::BlockChild::Line(_));
    assert_matches!(block1.children[2], ltree::ast::BlockChild::Line(_));
    assert_matches!(block1.children[3], ltree::ast::BlockChild::Line(_));
    assert_matches!(block1.children[4], ltree::ast::BlockChild::VerticalSpace(_));

    assert_matches!(block1.children[5], ltree::ast::BlockChild::Block(ref block2));
    assert_eq!(block2.children.len(), 4);
    assert_matches!(block2.children[0], ltree::ast::BlockChild::Line(_));
    assert_matches!(block2.children[1], ltree::ast::BlockChild::Line(_));
    assert_matches!(block2.children[2], ltree::ast::BlockChild::Line(_));
    assert_matches!(block2.children[3], ltree::ast::BlockChild::Line(_));
    
}

