use std::cmp::Ordering;
use std::marker;
use std::mem;
use std::ops::Index;
use std::iter::FromIterator;
use avl_node::{AVLNodePtr, AVLNode, AVLNodePtrBase, AVLRoot, AVLRootPtr};
use avl_node;
use std::ptr;

pub struct DataNode<K, V> {
    node_ptr: AVLNode,
    key: K,
    value: V,
}

impl<K, V> DataNode<K, V> {
    #[inline]
    fn get_pair(self) -> (K, V) {
        (self.key, self.value)
    }
}

trait AVLDataNodeOperation {
    fn deep_clone<K, V>(node: AVLNodePtr, parent: AVLNodePtr) -> AVLNodePtr where K: Clone, V: Clone;
    fn key_ref<'a, K, V>(self) -> &'a K;
    fn value_ref<'a, K, V>(self) -> &'a V;
    fn value_mut<'a, K, V>(self) -> &'a mut V;
    fn get_pair<K, V>(self) -> (K, V);
    fn destroy<K, V>(self);
    fn new<K, V>(k: K, v: V) -> AVLNodePtr;
    fn set_value<K, V>(self, value: V);
    fn avl_data_node_deref_mut<K, V>(self) -> *mut DataNode<K, V>;
}

impl AVLDataNodeOperation for *mut AVLNode {
    fn deep_clone<K, V>(node: AVLNodePtr, parent: AVLNodePtr) -> AVLNodePtr where K: Clone, V: Clone {
        if node.is_null() {
            return node;
        }
        let res = AVLNodePtr::new(node.key_ref::<K, V>().clone(), node.value_ref::<K, V>().clone());
        res.set_parent(parent);
        res.set_left(AVLNodePtr::deep_clone::<K, V>(node.left(), res));
        res.set_right(AVLNodePtr::deep_clone::<K, V>(node.right(), res));
        res.set_height(node.height());
        res
    }

    #[inline]
    fn key_ref<'a, K, V>(self) -> &'a K {
        unsafe { &(*self.avl_data_node_deref_mut::<K, V>()).key }
    }

    #[inline]
    fn value_ref<'a, K, V>(self) -> &'a V {
        unsafe { &(*self.avl_data_node_deref_mut::<K, V>()).value }
    }

    #[inline]
    fn value_mut<'a, K, V>(self) -> &'a mut V {
        unsafe { &mut (*self.avl_data_node_deref_mut::<K, V>()).value }
    }

    #[inline]
    fn get_pair<K, V>(self) -> (K, V) {
        unsafe {
            let data_ptr = self.avl_data_node_deref_mut::<K, V>();
            Box::from_raw(data_ptr).get_pair()
        }
    }

    #[inline]
    fn destroy<K, V>(self) {
        unsafe {
            let data_ptr = self.avl_data_node_deref_mut::<K, V>();
            Box::from_raw(data_ptr);
        }
    }

    #[inline]
    fn new<K, V>(k: K, v: V) -> AVLNodePtr {
        let ptr = Box::into_raw(Box::new(DataNode::<K, V> {
            key: k,
            value: v,
            node_ptr: AVLNode::default(),
        }));
        unsafe { &mut (*ptr).node_ptr as AVLNodePtr }
    }

    #[inline]
    fn set_value<K, V>(self, value: V) {
        unsafe { (*self.avl_data_node_deref_mut::<K, V>()).value = value; }
    }

    #[inline]
    fn avl_data_node_deref_mut<K, V>(self) -> *mut DataNode<K, V> {
        container_of!(self, DataNode<K, V>, node_ptr)
    }
}

pub struct Cursors<'a, K, V> where K: Ord + 'a, V: 'a {
    tree_mut: &'a mut AVLTree<K, V>,
    pos: AVLNodePtr,
}

enum CursorsOperation {
    NEXT,
    PREV,
}

impl<'a, K, V> Cursors<'a, K, V> where K: Ord {
    pub fn next(&mut self) {
        self.pos = self.pos.next();
    }

    pub fn prev(&mut self) {
        self.pos = self.pos.prev();
    }

    pub fn get_ref(&self) -> Option<(&K, &V)> {
        if self.pos.not_null() {
            Some((self.pos.key_ref::<K, V>(), self.pos.value_ref::<K, V>()))
        } else {
            None
        }
    }

    pub fn get_mut(&mut self) -> Option<(&K, &mut V)> {
        if self.pos.not_null() {
            Some((self.pos.key_ref::<K, V>(), self.pos.value_mut::<K, V>()))
        } else {
            None
        }
    }

    fn erase<F>(&mut self, f: F, op: CursorsOperation) where F: Fn(Option<(K, V)>) {
        if self.pos.is_null() {
            f(None);
            return;
        }
        let node = self.pos;
        match op {
            CursorsOperation::NEXT => { self.next() }
            CursorsOperation::PREV => { self.prev() }
        }
        unsafe {
            self.tree_mut.remove_node(node);
            f(Some(node.get_pair()));
        }
    }

    pub fn erase_then_next<F>(&mut self, f: F) where F: Fn(Option<(K, V)>) {
        self.erase(f, CursorsOperation::NEXT);
    }

    pub fn erase_then_prev<F>(&mut self, f: F) where F: Fn(Option<(K, V)>) {
        self.erase(f, CursorsOperation::PREV);
    }
}

pub struct AVLTree<K, V> where K: Ord {
    root: AVLRoot,
    count: usize,
    _marker: marker::PhantomData<(K, V)>,
}

impl<K, V> AVLTree<K, V> where K: Ord {
    #[inline]
    pub fn find_cursors<'a>(tree: &'a mut AVLTree<K, V>, what: &K) -> Cursors<'a, K, V> {
        unsafe {
            let node = tree.find_node(&what);
            Cursors { tree_mut: tree, pos: node }
        }
    }

    #[inline]
    pub fn max_height(&self) -> i32 {
        self.root.node.height()
    }

    #[inline]
    pub fn empty(&self) -> bool {
        self.size() == 0
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.count
    }

    #[inline]
    fn first_node(&self) -> AVLNodePtr {
        self.root.node.first_node()
    }

    #[inline]
    fn last_node(&self) -> AVLNodePtr {
        self.root.node.last_node()
    }

    #[inline]
    pub fn new() -> Self {
        AVLTree { root: AVLRoot { node: ptr::null_mut() }, count: 0, _marker: marker::PhantomData }
    }

    #[inline]
    pub fn clone_from(&mut self, t: &AVLTree<K, V>) where K: Clone, V: Clone {
        self.root = AVLRoot { node: AVLNodePtr::deep_clone::<K, V>(t.root.node, ptr::null_mut()) };
        self.count = t.count;
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        let (duplicate, parent, cmp_node_ref) = self.find_duplicate(&key);
        if duplicate.is_null() {
            self.link_post_insert(key, value, parent, cmp_node_ref);
        } else {
            duplicate.set_value::<K, V>(value);
        }
    }

    #[inline]
    fn find_duplicate(&mut self, key: &K) -> (AVLNodePtr, AVLNodePtr, *mut AVLNodePtr) {
        unsafe {
            let mut duplicate = ptr::null_mut();
            let mut cmp_node_ref = &mut self.root.node as *mut AVLNodePtr;
            let mut parent = ptr::null_mut();
            while (*cmp_node_ref).not_null() {
                parent = *cmp_node_ref;
                match key.cmp(parent.key_ref::<K, V>()) {
                    Ordering::Less => {
                        cmp_node_ref = parent.left_mut();
                    }
                    Ordering::Equal => {
                        duplicate = parent;
                        break;
                    }
                    Ordering::Greater => {
                        cmp_node_ref = parent.right_mut();
                    }
                }
            }
            (duplicate, parent, cmp_node_ref)
        }
    }

    #[inline]
    unsafe fn find_node(&self, what: &K) -> AVLNodePtr {
        let mut node = self.root.node;
        while node.not_null() {
            match what.cmp(node.key_ref::<K, V>()) {
                Ordering::Equal => {
                    return node;
                }
                Ordering::Less => {
                    node = node.left();
                }
                Ordering::Greater => {
                    node = node.right();
                }
            }
        }
        ptr::null_mut()
    }


    #[inline]
    fn isomorphic(&self, t: &AVLTree<K, V>) -> bool {
        if self.size() != t.size() {
            return false;
        }
        self.root.node.isomorphic(t.root.node)
    }

    fn bst_check(&self) -> bool {
        let mut iter = self.iter();
        let first = iter.next();
        if first.is_none() {
            return iter.size_hint().0 == self.size() && self.root.node.is_null();
        }
        let mut prev = first;
        let mut cnt = 1usize;
        loop {
            match iter.next() {
                None => { break; }
                Some(x) => {
                    cnt += 1;
                    if *prev.unwrap().0 >= *x.0 {
                        return false;
                    }
                    prev = Some(x);
                }
            }
        }
        cnt == self.size()
    }

    fn bst_check_reverse(&self) -> bool {
        let mut iter = self.iter();
        let first = iter.next_back();
        if first.is_none() {
            return iter.size_hint().0 == self.size() && self.root.node.is_null();
        }
        let mut prev = first;
        let mut cnt = 1usize;
        loop {
            match iter.next_back() {
                None => { break; }
                Some(x) => {
                    cnt += 1;
                    if *prev.unwrap().0 <= *x.0 {
                        return false;
                    }
                    prev = Some(x);
                }
            }
        }
        cnt == self.size()
    }

    #[inline]
    unsafe fn remove_node(&mut self, node: AVLNodePtr) {
        if node.is_null() {
            return;
        }
        if !node.empty() {
            avl_node::erase_node(node, self.get_root_ptr());
            node.set_parent(node);
            self.count -= 1;
        }
    }

    #[inline]
    pub fn pop(&mut self, what: &K) -> Option<(K, V)> {
        unsafe {
            let node = self.find_node(what);
            if node.is_null() {
                None
            } else {
                self.remove_node(node);
                Some(node.get_pair())
            }
        }
    }

    #[inline]
    pub fn contain(&self, what: &K) -> bool {
        unsafe { self.find_node(what).not_null() }
    }

    #[inline]
    pub fn get_ref<'a, 'b>(&'a self, what: &K) -> Option<&'b V> where 'b: 'a {
        unsafe {
            let node = self.find_node(what);
            if node.is_null() {
                None
            } else {
                Some(node.value_ref::<K, V>())
            }
        }
    }

    #[inline]
    pub fn get_mut<'a, 'b>(&'a mut self, what: &K) -> Option<&'b mut V> where 'b: 'a {
        unsafe {
            let node = self.find_node(what);
            if node.is_null() {
                None
            } else {
                Some(node.value_mut::<K, V>())
            }
        }
    }

    #[inline]
    fn drop_node(node: AVLNodePtr) {
        if node.not_null() {
            AVLTree::<K, V>::drop_node(node.left());
            AVLTree::<K, V>::drop_node(node.right());
            node.destroy::<K, V>();
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        AVLTree::<K, V>::drop_node(self.root.node);
        self.root.node = ptr::null_mut();
        self.count = 0;
    }

    #[inline]
    pub fn traversal_clear(&mut self) {
        let mut next = ptr::null_mut();
        while self.root.node.not_null() {
            unsafe { avl_node::avl_node_tear(&mut self.root as avl_node::AVLRootPtr, &mut next as *mut AVLNodePtr).destroy::<K, V>() };
        }
        self.count = 0;
    }

    #[inline]
    pub fn link_post_insert(&mut self, key: K, value: V, parent: AVLNodePtr, cmp_node_ref: *mut AVLNodePtr) {
        let new_node = AVLNodePtr::new(key, value);
        unsafe { avl_node::link_node(new_node, parent, cmp_node_ref); }
        unsafe { avl_node::node_post_insert(new_node, self.get_root_ptr()); }
        self.count += 1;
    }

    #[inline]
    fn get_root_ptr(&mut self) -> AVLRootPtr {
        &mut self.root as AVLRootPtr
    }

    #[inline]
    pub fn insert_or_replace(&mut self, key: K, mut value: V) -> Option<V> {
        let (duplicate, parent, cmp_node_ref) = self.find_duplicate(&key);
        if duplicate.is_null() {
            self.link_post_insert(key, value, parent, cmp_node_ref);
            None
        } else {
            mem::swap(&mut value, duplicate.value_mut::<K, V>());
            Some(value)
        }
    }

    #[inline]
    pub fn keys(&self) -> Keys<K, V> {
        Keys { inner: self.iter(), _marker: marker::PhantomData }
    }

    #[inline]
    pub fn values(&self) -> Values<K, V> {
        Values { inner: self.iter(), _marker: marker::PhantomData }
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        ValuesMut { inner: self.iter_mut(), _marker: marker::PhantomData }
    }

    #[inline]
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            head: self.first_node(),
            tail: self.last_node(),
            len: self.size(),
            _marker: marker::PhantomData,
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            head: self.first_node(),
            tail: self.last_node(),
            len: self.size(),
            _marker: marker::PhantomData,
        }
    }

    #[inline]
    unsafe fn set_empty(&mut self) {
        self.root.node = ptr::null_mut();
        self.count = 0;
    }
}

impl<K, V> Drop for AVLTree<K, V> where K: Ord {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<K, V> Clone for AVLTree<K, V> where K: Ord + Clone, V: Clone {
    fn clone(&self) -> Self {
        let mut tree = AVLTree::new();
        tree.clone_from(&self);
        tree
    }
}

impl<'a, K, V> Index<&'a K> for AVLTree<K, V> where K: Ord {
    type Output = V;

    #[inline]
    fn index(&self, key: &K) -> &V {
        self.get_ref(key).expect("no entry found for key")
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for AVLTree<K, V> {
    fn from_iter<T: IntoIterator<Item=(K, V)>>(iter: T) -> AVLTree<K, V> {
        let mut tree = AVLTree::new();
        tree.extend(iter);
        tree
    }
}

impl<K: Ord, V> Extend<(K, V)> for AVLTree<K, V> {
    fn extend<T: IntoIterator<Item=(K, V)>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

pub struct Keys<'a, K: Ord + 'a, V: 'a> {
    inner: Iter<'a, K, V>,
    _marker: marker::PhantomData<&'a (K, V)>,
}

impl<'a, K: Ord, V> Clone for Keys<'a, K, V> {
    fn clone(&self) -> Keys<'a, K, V> {
        Keys { inner: self.inner.clone(), _marker: marker::PhantomData }
    }
}

impl<'a, K: Ord, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<(&'a K)> {
        self.inner.next().map(|(k, _)| k)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

pub struct Values<'a, K: 'a + Ord, V: 'a> {
    inner: Iter<'a, K, V>,
    _marker: marker::PhantomData<&'a (K, V)>,
}

impl<'a, K: Ord, V> Clone for Values<'a, K, V> {
    fn clone(&self) -> Values<'a, K, V> {
        Values { inner: self.inner.clone(), _marker: marker::PhantomData }
    }
}

impl<'a, K: Ord, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<(&'a V)> {
        self.inner.next().map(|(_, v)| v)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

pub struct ValuesMut<'a, K: 'a + Ord, V: 'a> {
    inner: IterMut<'a, K, V>,
    _marker: marker::PhantomData<(K, V)>,
}

impl<'a, K: Ord, V> Clone for ValuesMut<'a, K, V> {
    fn clone(&self) -> ValuesMut<'a, K, V> {
        ValuesMut { inner: self.inner.clone(), _marker: marker::PhantomData }
    }
}

impl<'a, K: Ord, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    #[inline]
    fn next(&mut self) -> Option<(&'a mut V)> {
        self.inner.next().map(|(_, v)| v)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

pub struct IntoIter<K: Ord, V> {
    head: AVLNodePtr,
    tail: AVLNodePtr,
    len: usize,
    _marker: marker::PhantomData<(K, V)>,
}

impl<K: Ord, V> Drop for IntoIter<K, V> {
    #[inline]
    fn drop(&mut self) {
        for (_, _) in self {}
    }
}

impl<K: Ord, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        if self.len == 0 {
            return None;
        }
        if self.head.is_null() {
            return None;
        }
        let head = self.head;
        self.head = self.head.next();
        let (k, v) = head.get_pair();
        self.len -= 1;
        Some((k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<K: Ord, V> DoubleEndedIterator for IntoIter<K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<(K, V)> {
        if self.len == 0 {
            return None;
        }
        if self.tail.is_null() {
            return None;
        }
        let tail = self.tail;
        self.tail = self.tail.prev();
        let (k, v) = tail.get_pair();
        self.len -= 1;
        Some((k, v))
    }
}

pub struct Iter<'a, K: Ord + 'a, V: 'a> {
    head: AVLNodePtr,
    tail: AVLNodePtr,
    len: usize,
    _marker: marker::PhantomData<&'a (K, V)>,
}

impl<'a, K: Ord + 'a, V: 'a> Clone for Iter<'a, K, V> {
    fn clone(&self) -> Iter<'a, K, V> {
        Iter {
            head: self.head,
            tail: self.tail,
            len: self.len,
            _marker: self._marker,
        }
    }
}

impl<'a, K: Ord + 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        if self.len == 0 {
            return None;
        }

        if self.head.is_null() {
            return None;
        }
        let head = self.head;
        let (k, v) = (head.key_ref::<K, V>(), head.value_ref::<K, V>());
        self.head = self.head.next();
        self.len -= 1;
        Some((k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, K: Ord + 'a, V: 'a> DoubleEndedIterator for Iter<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> {
        // println!("len = {:?}", self.len);
        if self.len == 0 {
            return None;
        }
        let tail = self.tail;
        let (k, v) = (tail.key_ref::<K, V>(), tail.value_ref::<K, V>());
        self.tail = self.tail.prev();
        self.len -= 1;
        Some((k, v))
    }
}

pub struct IterMut<'a, K: Ord + 'a, V: 'a> {
    head: AVLNodePtr,
    tail: AVLNodePtr,
    len: usize,
    _marker: marker::PhantomData<&'a (K, V)>,
}

impl<'a, K: Ord + 'a, V: 'a> Clone for IterMut<'a, K, V> {
    fn clone(&self) -> IterMut<'a, K, V> {
        IterMut {
            head: self.head,
            tail: self.tail,
            len: self.len,
            _marker: self._marker,
        }
    }
}

impl<'a, K: Ord + 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        if self.len == 0 {
            return None;
        }

        if self.head.is_null() {
            return None;
        }
        let head = self.head;
        let (k, v) = (head.key_ref::<K, V>(), head.value_mut::<K, V>());
        self.head = self.head.next();
        self.len -= 1;
        Some((k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, K: Ord + 'a, V: 'a> DoubleEndedIterator for IterMut<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<(&'a K, &'a mut V)> {
        if self.len == 0 {
            return None;
        }
        let tail = self.tail;
        let (k, v) = (tail.key_ref::<K, V>(), tail.value_mut::<K, V>());
        self.tail = self.tail.prev();
        self.len -= 1;
        Some((k, v))
    }
}

impl<K: Ord, V> IntoIterator for AVLTree<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(mut self) -> IntoIter<K, V> {
        let iter = if self.root.node.is_null() {
            IntoIter {
                head: ptr::null_mut(),
                tail: ptr::null_mut(),
                len: 0,
                _marker: marker::PhantomData,
            }
        } else {
            IntoIter {
                head: self.first_node(),
                tail: self.last_node(),
                len: self.size(),
                _marker: marker::PhantomData,
            }
        };
        unsafe { self.set_empty(); }
        iter
    }
}

#[cfg(test)]
pub mod test {
    extern crate rand;

    use avl::AVLTree;
    use std::cmp::Ordering;
    use avl::AVLDataNodeOperation;
    use avl_node::AVLNodePtrBase;

    type DefaultType = AVLTree<i32, Option<i32>>;

    #[test]
    fn test_avl_basic() {
        let mut t = DefaultType::new();
        {
            assert!(t.root.node.is_null());
            t.insert(3, None);
            assert_eq!(*t.root.node.key_ref::<i32, Option<i32>>(), 3);
            assert_eq!(t.root.node.height(), 1);
            assert!(t.root.node.left().is_null());
            assert!(t.root.node.right().is_null());

            t.insert(2, None);
            assert_eq!(*t.root.node.key_ref::<i32, Option<i32>>(), 3);
            assert_eq!(t.root.node.height(), 2);
            assert_eq!(*t.root.node.left().key_ref::<i32, Option<i32>>(), 2);

            t.insert(1, None);
            assert_eq!(*t.root.node.key_ref::<i32, Option<i32>>(), 2);
            assert_eq!(t.root.node.height(), 2);
            assert_eq!(*t.root.node.left().key_ref::<i32, Option<i32>>(), 1);
        }
    }

    #[test]
    fn test_avl_erase() {
        let test_num = 100usize;
        let mut t = default_build_avl(test_num);
        assert!(t.bst_check());
        assert!(t.bst_check_reverse());
        for _ in 0..60 {
            let x = (rand::random::<usize>() % test_num) as i32;
            unsafe {
                match t.pop(&x) {
                    None => {}
                    Some((k, v)) => {
                        assert_eq!(v.unwrap(), -x);
                        assert_eq!(k, x);
                    }
                }
                assert!(t.find_node(&x).is_null());
            }
        }
        assert!(t.bst_check());
        assert!(t.bst_check_reverse());
    }

    #[test]
    fn test_avl_rotate_right() {
        let mut t = DefaultType::new();
        {
            t.insert(3, None);
            assert_eq!(*t.root.node.key_ref::<i32, Option<i32>>(), 3);
            assert_eq!(t.root.node.height(), 1);
            t.insert(2, None);
            assert_eq!(*t.root.node.key_ref::<i32, Option<i32>>(), 3);
            assert_eq!(t.root.node.height(), 2);
            t.insert(1, None);
            assert_eq!(*t.root.node.key_ref::<i32, Option<i32>>(), 2);
            assert_eq!(t.root.node.height(), 2);
        }
    }

    #[test]
    fn test_avl_rotate_left() {
        let mut t = DefaultType::new();
        {
            t.insert(1, None);
            assert_eq!(*t.root.node.key_ref::<i32, Option<i32>>(), 1);
            assert_eq!(t.root.node.height(), 1);
            t.insert(2, None);
            assert_eq!(*t.root.node.key_ref::<i32, Option<i32>>(), 1);
            assert_eq!(t.root.node.height(), 2);
            t.insert(3, None);
            assert_eq!(*t.root.node.key_ref::<i32, Option<i32>>(), 2);
            assert_eq!(t.root.node.height(), 2);
        }
    }

    #[test]
    fn test_avl_element_cmp() {
        #[derive(Eq, Debug)]
        struct MyData {
            a: i32,
        }

        impl Ord for MyData {
            fn cmp(&self, other: &Self) -> Ordering {
                self.a.cmp(&other.a)
            }
        }

        impl PartialOrd for MyData {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl PartialEq for MyData {
            fn eq(&self, other: &Self) -> bool {
                self.a == other.a
            }
        }

        impl Clone for MyData {
            fn clone(&self) -> Self {
                MyData { a: self.a }
            }
        }

        let mut t = AVLTree::<MyData, Option<i32>>::new();
        {
            t.insert(MyData { a: 1 }, None);
            assert_eq!(*t.root.node.key_ref::<MyData, Option<i32>>(), MyData { a: 1 });
            assert_eq!(t.root.node.height(), 1);
            t.insert(MyData { a: 2 }, None);
            assert_eq!(*t.root.node.key_ref::<MyData, Option<i32>>(), MyData { a: 1 });
            assert_eq!(t.root.node.height(), 2);

            *t.get_mut(&MyData { a: 1 }).unwrap() = Some(23333);
            assert_eq!((*t.get_ref(&MyData { a: 1 }).unwrap()).unwrap(), 23333);
        }
    }

    #[test]
    fn test_avl_find() {
        let t = default_build_avl(1000);
        for num in 0..t.size() {
            let x = num as i32;
            assert_eq!(*t.get_ref(&x).unwrap(), Some(-x));
        }
    }

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

    pub fn default_build_avl(n: usize) -> DefaultType {
        let v = default_make_avl_element(n);
        let mut t = DefaultType::new();
        assert_eq!(t.size(), 0);
        for d in &v {
            t.insert(*d, Some(-*d));
        }
        t
    }

    #[test]
    fn test_avl_validate() {
        let test_num = 1000usize;
        let mut t = AVLTree::new();
        for i in 0..test_num {
            t.insert(i, i);
        }
        assert_eq!(t.size(), test_num);
        assert_eq!(t.root.node.height(), 10);
        let left = t.root.node.left();
        assert_eq!(left.height(), 9);
        let right = t.root.node.right();
        assert_eq!(right.height(), 9);

        assert!(t.bst_check());
        assert!(t.bst_check_reverse());
    }

    #[test]
    fn test_avl_clear() {
        let test_num = 200usize;
        let mut t = default_build_avl(test_num);
        t.clear();
        assert!(t.empty());
        assert!(t.root.node.is_null());
    }

    #[test]
    fn test_avl_clone() {
        let test_num = 500usize;
        let ta = default_build_avl(test_num);
        let tb = ta.clone();
        assert!(ta.isomorphic(&tb));
    }

    #[test]
    fn test_avl_iteration() {
        let v = default_make_avl_element(100);
        let mut t = AVLTree::new();
        for x in &v {
            t.insert(*x, -*x);
        }
        let mut u = 0;
        for (k, v) in t.iter_mut() {
            assert_eq!(*k, u);
            assert_eq!(*v, -u);
            u += 1;
        }
    }

    #[test]
    fn test_avl_extend_iter() {
        let mut a = AVLTree::new();
        a.insert(2, 2);
        let mut b = AVLTree::new();
        b.insert(1, 1);
        b.insert(3, 3);
        a.extend(b.into_iter());
        assert_eq!(a.size(), 3);
        assert_eq!(a[&1], 1);
        assert_eq!(a[&2], 2);
        assert_eq!(a[&3], 3);
    }

    #[test]
    fn test_avl_keys() {
        let mut v = default_make_avl_element(100);
        let mut t = AVLTree::new();
        for x in &v {
            t.insert(*x, -*x);
        }
        let keys: Vec<_> = t.keys().collect();
        v.sort();
        assert_eq!(v.len(), keys.len());
        for i in 0..v.len() {
            assert_eq!(v[i], *keys[i]);
        }
    }

    #[test]
    fn test_avl_values() {
        let mut v = default_make_avl_element(100);
        let mut t = AVLTree::new();
        for x in &v {
            t.insert(*x, -*x);
        }
        let values: Vec<_> = t.values().collect();
        v.sort();
        assert_eq!(values.len(), v.len());
        for i in 0..v.len() {
            assert_eq!(-v[i], *values[i]);
        }
    }

    #[test]
    fn test_avl_cursors() {
        let mut t = default_build_avl(100);
        {
            let mut cursors = AVLTree::find_cursors(&mut t, &50);
            assert_eq!(*cursors.get_ref().unwrap().0, 50);
            for _ in 0..10 {
                cursors.next();
            }
            assert_eq!(*cursors.get_ref().unwrap().0, 60);
            for _ in 0..5 {
                cursors.prev();
            }
            assert_eq!(*cursors.get_ref().unwrap().0, 55);
            cursors.erase_then_next(
                |x| {
                    assert!(x.is_some());
                    assert_eq!(x.unwrap().0, 55);
                }
            );
            assert_eq!(*cursors.get_ref().unwrap().0, 56);
            cursors.prev();
            assert_eq!(*cursors.get_ref().unwrap().0, 54);
            cursors.erase_then_prev(
                |x| {
                    assert!(x.is_some());
                    assert_eq!(x.unwrap().0, 54);
                }
            );
            assert_eq!(*cursors.get_ref().unwrap().0, 53);
            cursors.next();
            assert_eq!(*cursors.get_ref().unwrap().0, 56);

            *cursors.get_mut().unwrap().1 = None;
            assert_eq!(*cursors.get_ref().unwrap().1, None);

            cursors.erase_then_prev(|_| {});
        }
        assert_eq!(t.size(), 97);
        {
            let cursors = AVLTree::find_cursors(&mut t, &55);
            assert!(cursors.get_ref().is_none());
        }
    }
}

