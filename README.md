# OrdMap & HashMap
* AVL is not worse than RBTree, and is also a feasible way to resolve Hash Collision Attack.
* This package expose two public structs: `OrdMap` which implemented by optimized AVL, `HashMap` whose every index contains an AVL-Tree.
* To improve performance, raw pointer is used frequently. Because Rust uses a similar memory model to C/C++, two classic macros
`offset_of` and `container_of` are used to dereference member variables into main struct.
`Fastbin` is implemented to reduce the cost of memory allocation.
* The whole structure os HashMap is like:
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
--------------------------------------------

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
--------------------------------------------
```
## HashMap Competition
Run commend
```
cargo run --release --example hash_map_cmp_string
```
* If the type of key is `String`, ours performs better in case of `insert` and `search`, Because Fastbin makes rehashing
run faster and comparable hash node helps to reduce search time.
```
test hash avl map
insert time PT0.264062182S
max node num of single index: 7
find 1000000, time PT0.232942470S
remove time PT0.303657020S

test stl hash map
insert time PT0.382943597S
find 1000000, time PT0.254504066S
remove time PT0.297953633S
--------------------------------

test hash avl map
insert time PT1.623697494S
max node num of single index: 8
find 5000000, time PT1.374587816S
remove time PT1.712209458S

test stl hash map
insert time PT2.146439362S
find 5000000, time PT1.494242541S
remove time PT1.613802131S
--------------------------------
```
* However, if type is usize|isize|f32... , which means the cost of key comparing and memory copying are low, then 
STL HashMap sometimes performs better in case of `insert` and `remove`.
```
cargo run --release --example hash_map_cmp_usize
```
```
test hash avl map
insert time PT0.134678145S
max node num of single index: 8
find 1000000, time PT0.118454962S
remove time PT0.126796283S

test stl hash map
insert time PT0.126203898S
find 1000000, time PT0.099042480S
remove time PT0.104399187S
--------------------------------

test hash avl map
insert time PT0.996003799S
max node num of single index: 8
find 5000000, time PT0.774285169S
remove time PT0.836908472S

test stl hash map
insert time PT0.982269496S
find 5000000, time PT0.868572380S
remove time PT0.835186910S
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
# Conclusion
* Rust is called `zero cost abstraction`, of course that's true. But comparing with C/C++, its compiler optimization is not good enough. 
In C/C++, syntax like `a = ( b < c ) ? d : e` may not generate branch, which means search operation on AVL could be better 
when the type of key is `1~8 Bytes`.