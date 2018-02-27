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

So, next step, we need to reduce abstraction.