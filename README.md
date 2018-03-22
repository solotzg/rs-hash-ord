# OrdMap & HashMap
* AVL is not worse than RBTree, and is also a feasible way to resolve Hash Collision Attack.
* This package expose two public structs: `OrdMap` which implemented by optimized AVL, `HashMap` whose every index contains an AVL-Tree.
* To improve performance, raw pointer is used frequently. Because Rust uses a similar memory model to C/C++, two classic macros
`offset_of` and `container_of` are used to dereference member variables into main struct.
`Fastbin` is implemented to reduce the cost of memory allocation.
# Usage
Notice the Trait. Usage of most functions is same as STL HashMap, you can find examples in test case. 
```
impl<K, V, S> HashMap<K, V, S> where K: Ord + Hash, S: BuildHasher
impl<K, V> OrdMap<K, V> where K: Ord
```
# Performance Test
## Environment
```
Linux version 4.4.0-1049-aws (buildd@lcy01-amd64-001) (gcc version 5.4.0 20160609 (Ubuntu 5.4.0-6ubuntu1~16.04.5) )
Intel(R) Xeon(R) CPU E5-2676 v3 @ 2.40GHz
```
## AVL Compare with RBTree
Somebody has already implemented RBTree, append `rbtree = "0.1.5"` to your `Cargo.toml [dependencies]`, then run commend
```
cargo run --release --example avl_cmp_rbtree
```
* Obviously, optimized AVL is not worse than RBTree, and with the advantage of smaller height, it works better in searching case.
It performs much better in clear case benefit from `Fastbin`
```
avl tree
size 1000000
build time PT0.523259712S 
contain count 1000000
find time PT0.462317294S 
remove time PT0.456915364S 
insert after remove time PT0.442013480S 
clear time PT0.029607052S 

rbtree
size 1000000
build time PT0.615952198S 
contain count 1000000
find time PT0.570363857S 
remove time PT0.641548505S 
insert after remove time PT0.613216248S 
clear time PT0.162732696S 
--------------------------------

avl tree
size 10000000
build time PT11.083079762S 
contain count 10000000
find time PT9.908204148S 
remove time PT11.365612250S 
insert after remove time PT11.503721610S 
clear time PT0.474218222S 

rbtree
size 10000000
build time PT12.478368807S 
contain count 10000000
find time PT11.286116786S 
remove time PT12.362542744S 
insert after remove time PT12.062266663S 
clear time PT2.479094958S 
--------------------------------
```
## HashMap Competition
Run commend
```
cargo run --release --example hash_map_cmp
```
* Our HashMap performs better in case of insert and search.
* When facing Collision Attack, the runtime complexity of STL HashMap can be _O(n)_, but ours is _O(log n)_.
```
test hash avl map
insert time PT0.271783330S
max node num of single index: 7
find 1000000, time PT0.244095811S
remove time PT0.301936260S

test stl hash map
insert time PT0.395503723S
find 1000000, time PT0.252295697S
remove time PT0.291297694S
--------------------------------

test hash avl map
insert time PT1.611267195S
max node num of single index: 8
find 5000000, time PT1.287719966S
remove time PT1.672060759S

test stl hash map
insert time PT2.149665549S
find 5000000, time PT1.454428664S
remove time PT1.600728169S
--------------------------------
```