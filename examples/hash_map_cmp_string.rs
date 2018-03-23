extern crate hash_ord;
extern crate time;
extern crate rand;

use hash_ord::hash_map;
use std::collections::HashMap as STLHashMap;

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
    run(1_000_000);
    run(5_000_000);
//    run(10_000_000);
}

fn run(max_num: usize) {
    let v = default_make_avl_element(max_num);
    test_hash_avl_map(max_num, &v);
    test_stl_hash_map(max_num, &v);
    println!("--------------------------------");
}

fn test_stl_hash_map(max_num: usize, v: &Vec<i32>) {
    println!("\ntest stl hash map");
    let mut map = STLHashMap::new();
    map.reserve(max_num);
    let start = time::now();
    for i in v {
        map.insert(i.to_string(), -*i);
    }
    let duration = time::now() - start;
    println!("insert time {}", duration);

    let start = time::now();
    let mut cnt = 0;
    for i in v {
        cnt += if map.get(&i.to_string()).is_none() { 0 } else { 1 };
    }
    let duration = time::now() - start;
    println!("find {}, time {}", cnt, duration);

    let start = time::now();
    for i in v {
        map.remove(&i.to_string());
    }
    let duration = time::now() - start;
    println!("remove time {}", duration);
}

fn test_hash_avl_map(max_num: usize, v: &Vec<i32>) {
    println!("\ntest hash avl map");
    let mut map = hash_map::HashMap::new();
    map.reserve(max_num);
    let start = time::now();
    for i in v {
        map.insert(i.to_string(), -*i);
    }
    let duration = time::now() - start;
    println!("insert time {}", duration);

    println!("max node num of single index: {}", map.get_max_node_of_single_index());

    let start = time::now();
    let mut cnt = 0;
    for i in v {
        cnt += if map.get(&i.to_string()).is_none() { 0 } else { 1 };
    }
    let duration = time::now() - start;
    println!("find {}, time {}", cnt, duration);

    let start = time::now();
    for i in v {
        map.remove(&i.to_string());
    }
    let duration = time::now() - start;
    println!("remove time {}", duration);
}