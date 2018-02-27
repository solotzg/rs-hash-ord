extern crate hash_avl;
extern crate time;

use hash_avl::avl::test::{default_make_avl_element};

fn main() {
    let n = 10_000_000;
    let v = default_make_avl_element(n);
    let mut t = hash_avl::avl::Tree::new();
    let start = time::now();
    for d in &v {
        t.avl_add_element(*d, -*d);
    }
    println!("size {}", t.size());
    println!("height {}", t.height());
    let end = time::now();
    let duration = end - start;
    println!("build avl time {} ", duration);
    let start = time::now();
    for num in &v {
        t.avl_get(num);
    }
    let end = time::now();
    let duration = end - start;
    println!("find avl time {} ", duration);

    let start = time::now();
    t.avl_tree_clear();
    let end = time::now();
    let duration = end - start;
    println!("clear avl time {} ", duration);
}