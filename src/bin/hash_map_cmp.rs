extern crate hash_avl;
extern crate time;

use hash_avl::hash_map;
use std::collections::HashMap as STLHashMap;

fn main() {
    let max_num = 1_000_000;
    test_hash_avl_map(max_num);
    test_stl_hash_map(max_num);
}

fn test_stl_hash_map(max_num: isize) {
    println!("\ntest stl hash map");
    let mut map = STLHashMap::new();
    map.reserve(max_num as usize);
    let start = time::now();
    for i in 0..max_num {
        map.insert(i, -i);
    }
    let duration = time::now() - start;
    println!("insert time {}", duration);

    let start = time::now();
    for i in 0..max_num {
        assert_eq!(*map.get(&i).unwrap(), -i);
    }
    let duration = time::now() - start;
    println!("find time {}", duration);
}

fn test_hash_avl_map(max_num: isize) {
    println!("\ntest hash avl map");
    let mut map = hash_map::HashMap::new();
    map.reserve(max_num as usize);
    let start = time::now();
    for i in 0..max_num {
        map.insert(i, -i);
    }
    let duration = time::now() - start;
    println!("insert time {}", duration);

    let start = time::now();
    for i in 0..max_num {
        assert_eq!(*map.get(&i).unwrap(), -i);
    }
    let duration = time::now() - start;
    println!("find time {}", duration);
}