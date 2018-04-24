#![feature(plugin)]
#![feature(test)]
#![feature(allocator_api)]
#![allow(dead_code)]

extern crate hash_ord;
extern crate rand;
extern crate test;

use hash_ord::ord_map::OrdMap;
type DefaultType = OrdMap<i32, Option<i32>>;

fn default_make_avl_element(n: usize) -> Vec<i32> {
    let mut v = vec![0i32; n];
    for idx in 0..v.len() {
        v[idx] = idx as i32;
        let pos = rand::random::<usize>() % (idx + 1);
        assert!(pos <= idx);
        v.swap(idx, pos);
    }
    v
}

fn default_build_avl(n: usize) -> DefaultType {
    let v = default_make_avl_element(n);
    let mut t = DefaultType::new();
    assert_eq!(t.len(), 0);
    for d in &v {
        t.insert(*d, Some(-*d));
    }
    t
}

#[bench]
fn bench_avl_build(b: &mut test::Bencher) {
    let n = 100_000;
    let v = default_make_avl_element(n);
    b.iter(|| {
        let mut t = OrdMap::new();
        for num in &v {
            t.insert(*num, -(*num));
        }
    });
}

#[bench]
fn bench_avl_find(b: &mut test::Bencher) {
    let n = 10_000_000;
    let t = default_build_avl(n);
    b.iter(|| t.get(&-1).is_some());
}

#[bench]
fn bench_avl_build_pop(b: &mut test::Bencher) {
    let n = 100_000;
    let v = default_make_avl_element(n);
    b.iter(|| {
        let mut t = OrdMap::new();
        for num in &v {
            t.insert(*num, -(*num));
        }
        for num in &v {
            t.remove(num);
        }
    });
}

#[bench]
fn bench_avl_build_find_pop(b: &mut test::Bencher) {
    let n = 100_000;
    let v = default_make_avl_element(n);
    b.iter(|| {
        let mut t = OrdMap::new();
        for num in &v {
            t.insert(*num, -(*num));
        }
        for num in &v {
            t.get(num).is_some();
        }
        for num in &v {
            t.remove(num);
        }
    });
}

#[bench]
fn bench_test_box(b: &mut test::Bencher) {
    struct Node {
        data: Option<i32>,
    }
    b.iter(|| {
        for _ in 0..1000 {
            Box::new(Node { data: Some(123) });
        }
    })
}
