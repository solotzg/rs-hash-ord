# Performance Competition
AVL is not worse than RBTree, and this's a feasible measure to resolve Hash Collision.
# Environment
```
Linux version 4.4.0-1049-aws (buildd@lcy01-amd64-001) (gcc version 5.4.0 20160609 (Ubuntu 5.4.0-6ubuntu1~16.04.5) )
Intel(R) Xeon(R) CPU E5-2676 v3 @ 2.40GHz
```
## AVL Compare with RBTree
```
avl tree
size 1000000
build avl time PT0.449384390S 
contain count 1000000
find avl time PT0.413850025S 
clear avl time PT0.030095744S 
tol_time PT0.893330159S

rbtree
size 1000000
build avl time PT0.653474497S 
contain count 1000000
find avl time PT0.586090195S 
clear avl time PT0.163268201S 
tol_time PT1.402832893S
--------------------------------

avl tree
size 10000000
build avl time PT10.983235023S 
contain count 10000000
find avl time PT9.879522083S 
clear avl time PT0.505790728S 
tol_time PT21.368547834S

rbtree
size 10000000
build avl time PT12.055410179S 
contain count 10000000
find avl time PT11.174089882S 
clear avl time PT2.267426381S 
tol_time PT25.496926442S
--------------------------------
```
## Compare with another implement of AVL
```
my avl tree
build avl time PT0.447466098S 
contain count 1000000
find avl time PT0.412760397S 
tol_time PT0.892230455S

avl_tree 0.2.0
build avl time PT0.789258763S 
contain count 1000000
find avl time PT0.452490925S 
tol_time PT1.390873758S
--------------------------------

my avl tree
build avl time PT10.902003554S 
contain count 10000000
find avl time PT9.930489747S 
tol_time PT21.289193191S

avl_tree 0.2.0
build avl time PT15.921550652S 
contain count 10000000
find avl time PT9.961234928S 
tol_time PT28.548414545S
--------------------------------
```
## Use AVL to resolve Hash Collision
```
test hash avl map
insert time PT1.532513279S
max node num of single index: 8
find 5000000, time PT1.357764873S
remove time PT1.655397891S

test stl hash map
insert time PT2.101090628S
find 5000000, time PT1.425694606S
remove time PT1.577394663S
--------------------------------
```