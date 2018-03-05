extern crate hash_avl;
extern crate time;
extern crate avl_tree;

use hash_avl::avl::AVLTree as Tree;
use hash_avl::avl::test::default_make_avl_element;

fn main() {
    run(1000_000);
    run(10_000_000);
}

fn test_a(v: &Vec<i32>) {
    println!("\nmy avl tree");
    let mut t = Tree::new();
    let start = time::now();
    for d in v {
        t.insert(*d, *d * 2);
    }
    let end = time::now();
    let duration = end - start;
    println!("build avl time {} ", duration);
    let mut count = 0;
    let start = time::now();
    for num in v {
        count += if t.contain(num) {
            1
        } else {
            0
        };
    }
    let end = time::now();
    let duration = end - start;
    println!("contain count {}", count);
    println!("find avl time {} ", duration);
}

fn test_b(v: &Vec<i32>) {
    println!("\navl_tree 0.2.0");
    let mut t = avl_tree::AVLTree::new();
    let start = time::now();
    for d in v {
        t.insert(*d, *d * 2);
    }
    let end = time::now();
    let duration = end - start;
    println!("build avl time {} ", duration);
    let mut count = 0;
    let start = time::now();
    for num in v {
        count += if t.contains(*num) {
            1
        } else {
            0
        };
    }
    let end = time::now();
    let duration = end - start;
    println!("contain count {}", count);
    println!("find avl time {} ", duration);
}

fn run(n: usize) {
    let v = default_make_avl_element(n);
    let start = time::now();
    test_a(&v);
    let end = time::now();
    println!("tol_time {}", end - start);
    let start = time::now();
    test_b(&v);
    let end = time::now();
    println!("tol_time {}", end - start);
    println!("--------------------------------");
}