extern crate hash_ord;
extern crate rand;
extern crate time;

use hash_ord::hash_map;
use std::collections::HashMap as STLHashMap;
use std::hash::BuildHasher;
use std::hash::Hasher;

pub struct DefaultHasher;

impl DefaultHasher {
    #[allow(deprecated)]
    pub fn new() -> Self {
        DefaultHasher {}
    }
}

impl Default for DefaultHasher {
    fn default() -> DefaultHasher {
        DefaultHasher::new()
    }
}

impl Hasher for DefaultHasher {
    #[inline]
    fn finish(&self) -> u64 {
        0
    }

    #[inline]
    fn write(&mut self, _: &[u8]) {}
}

struct State {}

impl BuildHasher for State {
    type Hasher = DefaultHasher;
    #[inline]
    #[allow(deprecated)]
    fn build_hasher(&self) -> DefaultHasher {
        Default::default()
    }
}

pub fn default_make_avl_element(n: usize) -> Vec<usize> {
    let mut v = vec![0usize; n];
    for idx in 0..v.len() {
        v[idx] = n - idx;
    }
    v
}

fn main() {
    run(10_000);
}

fn run(max_num: usize) {
    let v = default_make_avl_element(max_num);
    test_hash_avl_map(max_num, &v);
    test_stl_hash_map(max_num, &v);
    println!("--------------------------------");
}

fn test_stl_hash_map(max_num: usize, v: &Vec<usize>) {
    println!("\ntest stl hash map");
    let mut map = STLHashMap::with_hasher(State {});
    map.reserve(max_num);
    let start = time::now();
    for i in v {
        map.insert(i, *i);
    }
    let duration = time::now() - start;
    println!("insert time {}", duration);

    let start = time::now();
    let mut cnt = 0;
    for i in v {
        cnt += if map.get(&i).is_none() { 0 } else { 1 };
    }
    let duration = time::now() - start;
    println!("find {}, time {}", cnt, duration);

    let start = time::now();
    for i in v {
        map.remove(&i);
    }
    let duration = time::now() - start;
    println!("remove time {}", duration);
}

fn test_hash_avl_map(max_num: usize, v: &Vec<usize>) {
    println!("\ntest hash avl map");
    let mut map = hash_map::HashMap::with_hasher(State {});
    map.reserve(max_num);
    let start = time::now();
    for i in v {
        map.insert(i, *i);
    }
    let duration = time::now() - start;
    println!("insert time {}", duration);

    println!(
        "max node num of single index: {}",
        map.get_max_node_of_single_index()
    );

    let start = time::now();
    let mut cnt = 0;
    for i in v {
        cnt += if map.get(&i).is_none() { 0 } else { 1 };
    }
    let duration = time::now() - start;
    println!("find {}, time {}", cnt, duration);

    let start = time::now();
    for i in v {
        map.remove(&i);
    }
    let duration = time::now() - start;
    println!("remove time {}", duration);
}
