use merry_compiler::ltree::make_ltree;
use merry_compiler::ltree;
use merry_compiler::assert_matches;
use merry_compiler::ltree::verify_ltree;

/// Verify that [`make_ltree`] constructs an *LTree* correctly
/// given a document containing only a simple nested list.
/// 
/// The following properties must be satisfied...
/// - The hierarchy of the tree matches the indentation of the document.
/// - Every *Line* in the document appears in the tree.
/// - There are no errenous nodes in the tree, for instance a `VerticalSpace`
///   where none actually exists.
#[test]
pub fn test_make_ltree() {
    let source = std::fs::read_to_string("tests/simple_list.md2").unwrap();
    let ltree = make_ltree(&source);

    let block0 = ltree.block;
    assert_matches!(block0.children[0], ltree::ast::BlockChild::List(ref family_list));
    {
        let feline_root = &family_list.children[0].content;
        {
            assert_matches!(feline_root.children[0], ltree::ast::BlockChild::Line(ref line));
            assert_eq!(line.line_content.as_ref(), "Feline");
        }
        {
            assert_matches!(feline_root.children[1], ltree::ast::BlockChild::List(ref species_list));
            {
                let housecat_root = &species_list.children[0].content;
                assert_matches!(housecat_root.children[0], ltree::ast::BlockChild::Line(ref line));
                assert_eq!(line.line_content.as_ref(), "House Cat");
                {
                    assert_matches!(housecat_root.children[1], ltree::ast::BlockChild::List(ref individuals_list));
                    {
                        let jessie_root = &individuals_list.children[0].content;
                        assert_matches!(jessie_root.children[0], ltree::ast::BlockChild::Line(ref line));
                        assert_eq!(line.line_content.as_ref(), "Jessie");
                        assert_eq!(jessie_root.children.len(), 1);
                    }
                    {
                        let stewie_root = &individuals_list.children[1].content;
                        assert_matches!(stewie_root.children[0], ltree::ast::BlockChild::Line(ref line));
                        assert_eq!(line.line_content.as_ref(), "Stewie");
                        assert_eq!(stewie_root.children.len(), 1);
                    }
                    {
                        let phoebe_root = &individuals_list.children[2].content;
                        assert_matches!(phoebe_root.children[0], ltree::ast::BlockChild::Line(ref line));
                        assert_eq!(line.line_content.as_ref(), "Phoebe");
                        assert_eq!(phoebe_root.children.len(), 1);
                    }
                    {
                        let titan_root = &individuals_list.children[3].content;
                        assert_matches!(titan_root.children[0], ltree::ast::BlockChild::Line(ref line));
                        assert_eq!(line.line_content.as_ref(), "Titan");
                        assert_eq!(titan_root.children.len(), 1);
                    }
                    assert_eq!(individuals_list.children.len(), 4);
                }
                assert_eq!(housecat_root.children.len(), 2);
            }
        }
        assert_eq!(feline_root.children.len(), 2);
    }

    {
        let canine_root = &family_list.children[1].content;
        {
            assert_matches!(canine_root.children[0], ltree::ast::BlockChild::Line(ref line));
            assert_eq!(line.line_content.as_ref(), "Canine");
        }
        {
            assert_matches!(canine_root.children[1], ltree::ast::BlockChild::List(ref species_list));
            {
                let labrador_root = &species_list.children[0].content;
                assert_matches!(labrador_root.children[0], ltree::ast::BlockChild::Line(ref line));
                assert_eq!(line.line_content.as_ref(), "Labrador");
                {
                    assert_matches!(labrador_root.children[1], ltree::ast::BlockChild::List(ref individuals_list));
                    {
                        let cocoa_root = &individuals_list.children[0].content;
                        assert_matches!(cocoa_root.children[0], ltree::ast::BlockChild::Line(ref line));
                        assert_eq!(line.line_content.as_ref(), "Cocoa");
                        assert_eq!(cocoa_root.children.len(), 1);
                    }
                    {
                        let maggie_root = &individuals_list.children[1].content;
                        assert_matches!(maggie_root.children[0], ltree::ast::BlockChild::Line(ref line));
                        assert_eq!(line.line_content.as_ref(), "Maggie");
                        assert_eq!(maggie_root.children.len(), 1);
                    }
                    assert_eq!(individuals_list.children.len(), 2);
                }
                assert_eq!(labrador_root.children.len(), 2);
            }
            {
                let pit_root = &species_list.children[1].content;
                assert_matches!(pit_root.children[0], ltree::ast::BlockChild::Line(ref line));
                assert_eq!(line.line_content.as_ref(), "Pit");
                {
                    assert_matches!(pit_root.children[1], ltree::ast::BlockChild::List(ref individuals_list));
                    {
                        let layla_root = &individuals_list.children[0].content;
                        assert_matches!(layla_root.children[0], ltree::ast::BlockChild::Line(ref line));
                        assert_eq!(line.line_content.as_ref(), "Layla");
                        assert_eq!(layla_root.children.len(), 1);
                    }
                    assert_eq!(individuals_list.children.len(), 1);
                }
                assert_eq!(pit_root.children.len(), 2);
            }
        }
        assert_eq!(canine_root.children.len(), 2);
    }
    assert_eq!(family_list.children.len(), 2);
    assert_eq!(block0.children.len(), 1);
}


#[test]
pub fn test_verify_ltree() {
    let source = std::fs::read_to_string("tests/simple_list.md2").unwrap();
    let ltree = make_ltree(&source);
    let warnings = verify_ltree(&ltree);
     assert!(warnings.is_empty());
}
