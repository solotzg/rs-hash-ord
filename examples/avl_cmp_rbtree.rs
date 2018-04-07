extern crate hash_ord;
extern crate time;

mod rbtree_by_tickdream125;

use hash_ord::ord_map::OrdMap;
use rbtree_by_tickdream125::RBTree;

fn run(n: usize) {
    println!("\navl tree");
    let mut t = OrdMap::new();
    let start = time::now();
    for d in 0..n {
        t.insert(d, d * 2);
    }
    let end = time::now();
    let duration = end - start;
    println!("size {}", t.len());
    println!("build time {} ", duration);
    assert!(t.check_balanced());
    let mut count = 0;
    let start = time::now();
    for d in 0..n {
        count += if t.contains_key(&d) { 1 } else { 0 };
    }
    let end = time::now();
    let duration = end - start;
    println!("contain count {}", count);
    println!("find time {} ", duration);
    let start = time::now();
    for d in 0..n {
        t.remove(&d);
    }
    let end = time::now();
    let duration = end - start;
    println!("remove time {} ", duration);
    let start = time::now();
    for d in 0..n {
        t.insert(d, d * 2);
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
    let mut t = RBTree::new();
    let start = time::now();
    for d in 0..n {
        t.insert(d, d * 2);
    }
    let end = time::now();
    let duration = end - start;
    println!("size {}", t.len());
    println!("build time {} ", duration);
    let mut count = 0;
    let start = time::now();
    for d in 0..n {
        count += if t.contains_key(&d) { 1 } else { 0 };
    }
    let end = time::now();
    let duration = end - start;
    println!("contain count {}", count);
    println!("find time {} ", duration);
    let start = time::now();
    for d in 0..n {
        t.remove(&d);
    }
    let end = time::now();
    let duration = end - start;
    println!("remove time {} ", duration);
    let start = time::now();
    for d in 0..n {
        t.insert(d, d * 2);
    }
    let end = time::now();
    let duration = end - start;
    println!("insert after remove time {} ", duration);
    let start = time::now();
    t.clear();
    let end = time::now();
    let duration = end - start;
    println!("clear time {} ", duration);
    println!("--------------------------------------------");
}

fn main() {
    run(1000_000);
    run(10_000_000);
}
