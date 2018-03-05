# Performance Competition
AVL is not worse than RBTree 
## Compare with RBTree
```
Linux version 4.4.0-1049-aws (buildd@lcy01-amd64-001) (gcc version 5.4.0 20160609 (Ubuntu 5.4.0-6ubuntu1~16.04.5) )
Intel(R) Xeon(R) CPU E5-2676 v3 @ 2.40GHz
```
```
avl tree
size 100000
build avl time PT0.028503527S 
contain count 100000
find avl time PT0.020674714S 
clear avl time PT0.005049490S 
tol_time PT0.054227731S

rbtree
size 100000
build avl time PT0.028908981S 
contain count 100000
find avl time PT0.024804220S 
clear avl time PT0.005598498S 
tol_time PT0.059311699S
--------------------------------

avl tree
size 1000000
build avl time PT0.563494613S 
contain count 1000000
find avl time PT0.487795776S 
clear avl time PT0.123542618S 
tol_time PT1.174833007S

rbtree
size 1000000
build avl time PT0.637215084S 
contain count 1000000
find avl time PT0.586499857S 
clear avl time PT0.166939198S 
tol_time PT1.390654139S
--------------------------------

avl tree
size 10000000
build avl time PT11.550990767S 
contain count 10000000
find avl time PT9.896446463S 
clear avl time PT1.793976350S 
tol_time PT23.241413580S

rbtree
size 10000000
build avl time PT12.740772577S 
contain count 10000000
find avl time PT11.506824022S 
clear avl time PT2.621726976S 
tol_time PT26.869323575S
--------------------------------
```
## Compare with another implement of AVL
```
my avl tree
build avl time PT0.525162393S 
contain count 1000000
find avl time PT0.463548624S 
tol_time PT1.111170961S

avl_tree 0.2.0
build avl time PT0.858368285S 
contain count 1000000
find avl time PT0.472501388S 
tol_time PT1.483857972S
--------------------------------

my avl tree
build avl time PT12.050122632S 
contain count 10000000
find avl time PT10.213842381S 
tol_time PT24.446940781S

avl_tree 0.2.0
build avl time PT16.647625118S 
contain count 10000000
find avl time PT10.369277327S 
tol_time PT29.805170112S
--------------------------------
```