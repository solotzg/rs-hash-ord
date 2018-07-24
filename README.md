# OrdMap & HashMap
[![Build Status](https://travis-ci.org/solotzg/rs-hash-ord.svg?branch=master)](https://travis-ci.org/solotzg/rs-hash-ord) [![Crates.io](https://img.shields.io/crates/v/hash_ord.svg)](https://crates.io/crates/hash_ord)
* AVL is not worse than RBTree and is also a feasible way to resolve Hash Collision Attack.
* This package exposes two public structs: `OrdMap` which implemented by optimized AVL, `HashMap` whose every index contains an AVL-Tree.
* To improve performance, raw pointer is used frequently. Because Rust uses a similar memory model to C/C++, two classic macros
`offset_of` and `container_of` are used to dereference member variables into main struct.
`Fastbin` is implemented to reduce the cost of memory allocation.
* `insert` and `remove` operations are optimized by selectively skipping `AVL Rebalance`, because under 95% of indexes, 
there are less than 3 nodes.
* Since `SipHash` is not good at performance, `FnvBuildHasher` is used as the default `BuildHasher`.
* The whole structure of HashMap is like:
```
 HashMap:
         ------------------------------------------
         |   HashIndex   |   HashIndex   |   ...
         ------------------------------------------
                                                 ----------------
 HashIndex:                                      |    ......    |
         ----------------    ----------------    |   AVL Node   |
         |   ........   |    |   AVL Root   |==> ----------------
 ... <==>|   ListHead   |<==>|   ListHead   |<==> ... <==> HEAD <==> ...
         ----------------    ----------------

 InternalHashEntry
             ----------------
             |   HashNode   |
             |     Value    |
             ----------------

 HashNode
          ----------------    -----------------
          |    ......    |    |   HashValue   |
          |    ......    |    |      Key      |
  ... <==>|   AVL Node   |<==>|    AVL Node   |<==> ...
          ----------------    -----------------
```
# Usage
Notice the Trait. Usage of most functions is same as STL HashMap, you can find examples in test case or 
[Documentation](https://docs.rs/hash_ord/). 
```
impl<K, V, S> HashMap<K, V, S> where K: Ord + Hash, S: BuildHasher
impl<K, V> OrdMap<K, V> where K: Ord
```
# Performance Test
## AVL Compare with RBTree
```
cargo run --release --example avl_cmp_rbtree
```
* Obviously, optimized AVL is not worse than RBTree, and with the advantage of smaller height and `Fastbin`, it performs better.
```
avl tree
size 1000000
build time PT0.086414625S 
contain count 1000000
find time PT0.079008516S 
remove time PT0.047414057S 
insert after remove time PT0.068621561S 
clear time PT0.020838042S 

rbtree
size 1000000
build time PT0.255747347S 
contain count 1000000
find time PT0.128373407S 
remove time PT0.129552620S 
insert after remove time PT0.256210620S 
clear time PT0.051735117S 
--------------------------------------------

avl tree
size 10000000
build time PT0.996452066S 
contain count 10000000
find time PT0.862577411S 
remove time PT0.641826155S 
insert after remove time PT0.779849807S 
clear time PT0.206050905S 

rbtree
size 10000000
build time PT3.348531156S 
contain count 10000000
find time PT1.482958388S 
remove time PT1.538498453S 
insert after remove time PT3.361221077S 
clear time PT0.505872813S 
--------------------------------------------
```
## HashMap Competition
* Because the operations of `String` themselves take too much time, we just show the competition with key `usize`.

Run command: 
```
cargo run --release --example hash_map_cmp_usize
```
Our implement performs much better, especially in the case of `searching`.
```
test hash avl map
insert time PT0.111996300S
max node num of single index: 1
find 1000000, time PT0.042600200S
remove time PT0.108599100S

test stl hash map
insert time PT0.141726400S
find 1000000, time PT0.117749600S
remove time PT0.139572800S
--------------------------------

test hash avl map
insert time PT0.661286300S
max node num of single index: 2
find 5000000, time PT0.251113S
remove time PT0.638360800S

test stl hash map
insert time PT0.746569600S
find 5000000, time PT0.647037200S
remove time PT0.828150S
--------------------------------

test hash avl map
insert time PT1.385713200S
max node num of single index: 2
find 10000000, time PT0.521490300S
remove time PT1.368300300S

test stl hash map
insert time PT1.632164300S
find 10000000, time PT1.426744300S
remove time PT1.703507200S
--------------------------------
```
* When facing Collision Attack, the runtime complexity of STL HashMap can be _O(n^2)_, but ours is _O(n log n)_.
```
cargo run --release --example hash_map_cmp_collision
```
```
test hash avl map
insert time PT0.000739762S
max node num of single index: 10000
find 10000, time PT0.000690444S
remove time PT0.000497103S

test stl hash map
insert time PT0.093169479S
find 10000, time PT0.079600027S
remove time PT0.089153558S
--------------------------------
```

# Change Logs
* version `0.1.9`
  - Because rust nightly change alloc api too frequently, use `libc::{malloc, free}` instead; 
  - Use stable `ops::RangeBounds`(stable since = "1.28.0") instead of `collections::range::RangeArgument`;

* version `0.1.10`
  - Function `avl_node_tear` doesn't work as expected; This bug did not cause any error;