extern crate hash_avl;
extern crate time;
extern crate rbtree;
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

fn run(n: usize) {
    println!("\navl tree");
    let v = default_make_avl_element(n);
    let mut t = Tree::new();
    let start = time::now();
    for d in &v {
        t.insert(*d, *d * 2);
    }
    let end = time::now();
    let duration = end - start;
    println!("size {}", t.size());
    println!("build time {} ", duration);
    assert!(t.check_valid());
    let mut count = 0;
    let start = time::now();
    for num in &v {
        count += if t.contain(num) {
            1
        } else {
            0
        };
    }
    let end = time::now();
    let duration = end - start;
    println!("contain count {}", count);
    println!("find time {} ", duration);
    let start = time::now();
    for num in &v {
        t.remove(num);
    }
    let end = time::now();
    let duration = end - start;
    println!("remove time {} ", duration);
    let start = time::now();
    for d in &v {
        t.insert(*d, *d * 2);
    }
    let end = time::now();
    let duration = end - start;
    println!("insert after remove time {} ", duration);
    let start = time::now();
    t.clear();
    let end = time::now();
    let duration = end - start;
    println!("clear time {} ", duration);

    drop(t);

    println!("\nrbtree");
    let mut t = rbtree::RBTree::new();
    let start = time::now();
    for d in &v {
        t.insert(*d, *d * 2);
    }
    let end = time::now();
    let duration = end - start;
    println!("size {}", t.len());
    println!("build time {} ", duration);
    let mut count = 0;
    let start = time::now();
    for num in &v {
        count += if t.contains_key(num) {
            1
        } else {
            0
        };
    }
    let end = time::now();
    let duration = end - start;
    println!("contain count {}", count);
    println!("find time {} ", duration);
    let start = time::now();
    for num in &v {
        t.remove(num);
    }
    let end = time::now();
    let duration = end - start;
    println!("remove time {} ", duration);
    let start = time::now();
    for d in &v {
        t.insert(*d, *d * 2);
    }
    let end = time::now();
    let duration = end - start;
    println!("insert after remove time {} ", duration);
    let start = time::now();
    t.clear();
    let end = time::now();
    let duration = end - start;
    println!("clear time {} ", duration);
    println!("--------------------------------");
}

fn main() {
    run(1000_000);
    run(10_000_000);
}