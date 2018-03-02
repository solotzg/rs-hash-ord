# rs_hash_avl

It's so fucking difficult to complete avl without using raw pointer. We try to use raw pointer as little as possible, and the performance of branch `0.1.0` is blow 

```
Windows 10
Intel(R) Core(TM) i5-4460  CPU @ 3.20GHz
```
```
test bench_avl_build          ... bench:  37,608,105 ns/iter (+/- 5,436,223)
test bench_avl_build_find_pop ... bench:  78,761,362 ns/iter (+/- 4,114,177)
test bench_avl_build_pop      ... bench:  57,745,992 ns/iter (+/- 5,958,457)
test bench_avl_find           ... bench:   8,622,224 ns/iter (+/- 2,181,619)
```
```
size 10000000
height 28
build avl time PT13.256197600S 
find avl time PT11.067222800S 
clear avl time PT3.498083200S
```
So, next step, we need to reduce abstraction.

```
avl tree
size 10000000
build avl time PT13.673228400S 
contain count 10000000
find avl time PT11.502455600S 
clear avl time PT3.465241200S 
tol_time PT28.640925200S

rbtree
size 10000000
build avl time PT13.451483800S 
contain count 10000000
find avl time PT12.081983800S 
clear avl time PT3.461229700S 
tol_time PT28.994697300S
```