use merry_compiler::ltree::make_ltree;

#[test]
pub fn test_simple_list() {
    let source = std::fs::read_to_string("tests/simple_list.md2").unwrap();
    let ltree = make_ltree(&source);
    println!("{:#?}", ltree);
}
