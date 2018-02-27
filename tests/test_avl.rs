extern crate hash_avl;

#[test]
fn test_avl() {
    let mut t = hash_avl::avl::Tree::new();
    assert!(t.avl_add_element(1, -1));
    assert!(t.avl_add_element(2, -2));
    assert!(t.avl_add_element(3, -3));
    assert!(!t.avl_add_element(1, 111));
}
