extern crate hash_avl;
extern crate time;
extern crate avl_tree;
extern crate rand;

use hash_avl::avl::AVLTree as Tree;

pub fn default_make_avl_element(n: usize) -> Vec<i32> {
    let mut v = vec![0i32; n];
    for idx in 0..v.len() {
        v[idx] = idx as i32;
        let pos = rand::random::<usize>() % (idx + 1);
        assert!(pos <= idx);
        v.swap(idx, pos);
    }
    v
}

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