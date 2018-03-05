#![feature(plugin)]
#![feature(test)]

extern crate test;
extern crate rand;
extern crate hash_avl;

use hash_avl::avl::AVLTree as Tree;
use hash_avl::avl::test::{default_build_avl, default_make_avl_element};

#[bench]
fn bench_avl_build(b: &mut test::Bencher) {
    let n = 100_000;
    let v = default_make_avl_element(n);
    b.iter(|| {
        let mut t = Tree::new();
        for num in &v {
            t.insert(*num, -(*num));
        }
    });
}

#[bench]
fn bench_avl_find(b: &mut test::Bencher) {
    let n = 10_000_000;
    let t = default_build_avl(n);
    b.iter(|| {
        t.get_ref(&-1).is_some()
    });
}

#[bench]
fn bench_avl_build_pop(b: &mut test::Bencher) {
    let n = 100_000;
    let v = default_make_avl_element(n);
    b.iter(|| {
        let mut t = Tree::new();
        for num in &v {
            t.insert(*num, -(*num));
        }
        for num in &v {
            t.pop(num);
        }
    });
}

#[bench]
fn bench_avl_build_find_pop(b: &mut test::Bencher) {
    let n = 100_000;
    let v = default_make_avl_element(n);
    b.iter(|| {
        let mut t = Tree::new();
        for num in &v {
            t.insert(*num, -(*num));
        }
        for num in &v {
            t.get_ref(num).is_some();
        }
        for num in &v {
            t.pop(num);
        }
    });
}