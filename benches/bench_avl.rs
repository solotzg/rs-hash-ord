#![feature(plugin)]
#![feature(test)]

extern crate test;
extern crate rand;
extern crate hash_avl;

use hash_avl::avl::test::default_build_avl;

#[bench]
fn bench_avl_build(b: &mut test::Bencher) {
    let n = 100_000;
    let mut v = vec![0i32; n];
    for idx in 0..v.len() {
        v[idx] = idx as i32;
        let pos = rand::random::<usize>() % (idx + 1);
        assert!(pos <= idx);
        v.swap(idx, pos);
    }
    b.iter(|| {
        let mut t = hash_avl::avl::Tree::new();
        for num in &v {
            t.avl_add_element(*num, -(*num));
        }
    });
}

#[bench]
fn bench_avl_find(b: &mut test::Bencher) {
    let n = 100_000;
    let t = default_build_avl(n);
    b.iter(|| {
        for x in 0..n {
            let num = &(x as i32);
            t.avl_get(num);
        }
    });
}

#[bench]
fn bench_avl_build_pop(b: &mut test::Bencher) {
    b.iter(|| {
        let n = 100_000;
        let mut t = default_build_avl(n);
        for x in 0..n {
            let num = &(x as i32);
            t.avl_tree_pop(&num);
        }
    });
}