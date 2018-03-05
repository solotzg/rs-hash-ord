use std::cmp::{Ordering, max};
use std::ptr;
use std::marker;
use std::mem;
use std::ops::Index;
use std::iter::FromIterator;

pub struct AVLNode<K, V> where K: Ord {
    left: NodePtr<K, V>,
    right: NodePtr<K, V>,
    parent: NodePtr<K, V>,
    key: K,
    value: V,
    height: i32,
}

#[derive(Debug)]
struct NodePtr<K, V>(*mut AVLNode<K, V>) where K: Ord;

impl<K: Ord, V> PartialEq for NodePtr<K, V> {
    fn eq(&self, other: &NodePtr<K, V>) -> bool {
        self.0 == other.0
    }
}

impl<K: Ord, V> Eq for NodePtr<K, V> {}

impl<K, V> Clone for NodePtr<K, V> where K: Ord {
    fn clone(&self) -> NodePtr<K, V> {
        NodePtr(self.0)
    }
}

impl<K, V> Copy for NodePtr<K, V> where K: Ord {}

impl<K, V> AVLNode<K, V> where K: Ord {
    #[inline]
    fn get_pair(self) -> (K, V) {
        (self.key, self.value)
    }
}

impl<K, V> NodePtr<K, V> where K: Ord {
    fn is_isomorphic(&self, node: NodePtr<K, V>) -> bool {
        if self.is_null() && node.is_null() {
            return true;
        }
        if self.is_null() || node.is_null() {
            return false;
        }
        if self.height() != node.height() {
            return false;
        }
        self.left().is_isomorphic(node.left()) && self.right().is_isomorphic(node.right())
    }

    fn deep_clone(node: NodePtr<K, V>, parent: NodePtr<K, V>) -> Self where K: Clone, V: Clone {
        if node.is_null() {
            return node;
        }
        let res = NodePtr::new(node.key_ref().clone(), node.value_ref().clone());
        res.set_parent(parent);
        res.set_left(NodePtr::deep_clone(node.left(), res));
        res.set_right(NodePtr::deep_clone(node.right(), res));
        res.set_height(node.height());
        res
    }

    #[inline]
    fn key_ref(&self) -> &K {
        unsafe { &(*self.0).key }
    }

    #[inline]
    fn value_ref(&self) -> &V {
        unsafe { &(*self.0).value }
    }

    #[inline]
    fn get_pair(self) -> (K, V) {
        unsafe { Box::from_raw(self.0).get_pair() }
    }

    #[inline]
    fn destroy(&self) {
        unsafe { Box::from_raw(self.0); }
    }

    #[inline]
    fn height_update(&self) {
        self.set_height(max(self.left_height(), self.right_height()) + 1);
    }

    fn new(k: K, v: V) -> NodePtr<K, V> {
        NodePtr(Box::into_raw(Box::new(AVLNode {
            left: NodePtr::null(),
            right: NodePtr::null(),
            parent: NodePtr::null(),
            key: k,
            value: v,
            height: 0,
        })))
    }

    #[inline]
    fn height(&self) -> i32 {
        if self.is_null() {
            return 0;
        }
        unsafe { (*self.0).height }
    }

    #[inline]
    fn next(&self) -> NodePtr<K, V> {
        if self.is_null() {
            return NodePtr::null();
        }
        let mut node = *self;
        if self.right().not_null() {
            node = node.right();
            while node.left().not_null() {
                node = node.left();
            }
        } else {
            loop {
                let last = node;
                node = node.parent();
                if node.is_null() {
                    break;
                }
                if node.left() == last {
                    break;
                }
            }
        }
        node
    }

    #[inline]
    fn prev(&self) -> NodePtr<K, V> {
        if self.is_null() {
            return NodePtr::null();
        }
        let mut node = *self;
        if node.left().not_null() {
            node = node.left();
            while node.right().not_null() {
                node = node.right();
            }
        } else {
            loop {
                let last = node;
                node = node.parent();
                if node.is_null() {
                    break;
                }
                if node.right() == last {
                    break;
                }
            }
        }
        node
    }

    #[inline]
    fn set_parent(&self, parent: NodePtr<K, V>) {
        unsafe { (*self.0).parent = parent }
    }

    #[inline]
    fn set_left(&self, left: NodePtr<K, V>) {
        unsafe { (*self.0).left = left }
    }

    #[inline]
    fn set_right(&self, right: NodePtr<K, V>) {
        unsafe { (*self.0).right = right }
    }


    #[inline]
    fn parent(&self) -> NodePtr<K, V> {
        unsafe { (*self.0).parent }
    }

    #[inline]
    fn left(&self) -> NodePtr<K, V> {
        unsafe { (*self.0).left }
    }

    #[inline]
    fn right(&self) -> NodePtr<K, V> {
        unsafe { (*self.0).right }
    }

    #[inline]
    fn left_mut(&self) -> *mut NodePtr<K, V> {
        unsafe { &mut (*self.0).left }
    }

    #[inline]
    fn right_mut(&self) -> *mut NodePtr<K, V> {
        unsafe { &mut (*self.0).right }
    }

    #[inline]
    fn null() -> NodePtr<K, V> {
        NodePtr(ptr::null_mut())
    }

    #[inline]
    fn is_null(&self) -> bool {
        self.0.is_null()
    }

    #[inline]
    fn not_null(&self) -> bool {
        !self.0.is_null()
    }

    #[inline]
    fn set_height(&self, height: i32) {
        unsafe { (*self.0).height = height; }
    }

    #[inline]
    fn set_value(&mut self, value: V) {
        unsafe { (*self.0).value = value; }
    }

    #[inline]
    fn left_height(&self) -> i32 {
        self.left().height()
    }

    #[inline]
    fn right_height(&self) -> i32 {
        self.right().height()
    }
}

pub struct AVLTree<K, V> where K: Ord {
    root: NodePtr<K, V>,
    count: usize,
}

impl<K, V> AVLTree<K, V> where K: Ord {
    #[inline]
    pub fn max_height(&self) -> i32 {
        self.root.height()
    }

    #[inline]
    pub fn empty(&self) -> bool {
        self.size() == 0
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.count
    }

    fn first_node(&self) -> NodePtr<K, V> {
        let mut ptr = self.root;
        if ptr.is_null() {
            return NodePtr::null();
        }
        while ptr.left().not_null() {
            ptr = ptr.left();
        }
        ptr
    }

    fn last_node(&self) -> NodePtr<K, V> {
        let mut ptr = self.root;
        if ptr.is_null() {
            return NodePtr::null();
        }
        while ptr.right().not_null() {
            ptr = ptr.right();
        }
        ptr
    }

    pub fn new() -> Self {
        AVLTree { root: NodePtr::null(), count: 0 }
    }

    pub fn clone_from(&mut self, t: &AVLTree<K, V>) where K: Clone, V: Clone {
        self.root = NodePtr::deep_clone(t.root, NodePtr::null());
        self.count = t.count;
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        let (mut duplicate, parent, cmp_node_ref) = self.find_duplicate(&key);
        if duplicate.is_null() {
            self.link_post_insert(key, value, parent, cmp_node_ref);
        } else {
            duplicate.set_value(value);
        }
    }

    #[inline]
    fn link_post_insert(&mut self, key: K, value: V, parent: NodePtr<K, V>, cmp_node_ref: *mut NodePtr<K, V>) {
        let new_node = NodePtr::new(key, value);
        unsafe { AVLTree::link_node(new_node, parent, cmp_node_ref); }
        unsafe { self.node_post_insert(new_node); }
        self.count += 1;
    }

    #[inline]
    fn find_duplicate(&mut self, key: &K) -> (NodePtr<K, V>, NodePtr<K, V>, *mut NodePtr<K, V>) {
        unsafe {
            let mut duplicate = NodePtr::null();
            let mut cmp_node_ref: *mut NodePtr<K, V> = &mut self.root;
            let mut parent = NodePtr::null();
            while (*cmp_node_ref).not_null() {
                parent = *cmp_node_ref;
                match key.cmp(parent.key_ref()) {
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
    unsafe fn find_node(&self, what: &K) -> NodePtr<K, V> {
        let mut node = self.root;
        let mut res_node = NodePtr::null();
        while node.not_null() {
            match what.cmp(&(*node.0).key) {
                Ordering::Equal => {
                    res_node = node;
                    break;
                }
                Ordering::Less => {
                    node = node.left();
                }
                Ordering::Greater => {
                    node = node.right();
                }
            }
        }
        res_node
    }

    #[inline]
    unsafe fn link_node(new_node: NodePtr<K, V>, parent: NodePtr<K, V>, cmp_node: *mut NodePtr<K, V>) {
        new_node.set_parent(parent);
        new_node.set_height(0);
        new_node.set_left(NodePtr::null());
        new_node.set_right(NodePtr::null());
        *cmp_node = new_node;
    }

    #[inline]
    unsafe fn node_post_insert(&mut self, mut node: NodePtr<K, V>) {
        node.set_height(1);
        node = node.parent();
        while node.not_null() {
            let h0 = node.left_height();
            let h1 = node.right_height();
            let height = max(h1, h0) + 1;
            let diff = h0 - h1;
            if node.height() == height {
                break;
            }
            node.set_height(height);
            if diff <= -2 {
                node = self.node_fix_l(node);
            } else if diff >= 2 {
                node = self.node_fix_r(node);
            }
            node = node.parent();
        }
    }

    unsafe fn node_fix_l(&mut self, mut node: NodePtr<K, V>) -> NodePtr<K, V> {
        let right = node.right();
        let rh0 = right.left_height();
        let rh1 = right.right_height();
        if rh0 > rh1 {
            let right = self.node_rotate_right(right);
            right.right().height_update();
            right.height_update();
        }
        node = self.node_rotate_left(node);
        node.left().height_update();
        node.height_update();
        node
    }

    unsafe fn node_fix_r(&mut self, mut node: NodePtr<K, V>) -> NodePtr<K, V> {
        let left = node.left();
        let rh0 = left.left_height();
        let rh1 = left.right_height();
        if rh0 < rh1 {
            let left = self.node_rotate_left(left);
            left.left().height_update();
            left.height_update();
        }
        node = self.node_rotate_right(node);
        node.right().height_update();
        node.height_update();
        node
    }

    unsafe fn node_rotate_right(&mut self, node: NodePtr<K, V>) -> NodePtr<K, V> {
        let left = node.left();
        let parent = node.parent();
        node.set_left(left.right());
        if left.right().not_null() {
            left.right().set_parent(node);
        }
        left.set_right(node);
        left.set_parent(parent);
        self.child_replace(node, left, parent);
        node.set_parent(left);
        left
    }

    unsafe fn node_rotate_left(&mut self, node: NodePtr<K, V>) -> NodePtr<K, V> {
        let right = node.right();
        let parent = node.parent();
        node.set_right(right.left());
        if right.left().not_null() {
            right.left().set_parent(node);
        }
        right.set_left(node);
        right.set_parent(parent);
        self.child_replace(node, right, parent);
        node.set_parent(right);
        right
    }

    unsafe fn child_replace(&mut self, old_node: NodePtr<K, V>, new_node: NodePtr<K, V>, parent: NodePtr<K, V>) {
        if parent.is_null() {
            self.root = new_node;
        } else {
            if parent.left() == old_node {
                parent.set_left(new_node);
            } else {
                parent.set_right(new_node);
            }
        }
    }

    fn is_isomorphic(&self, t: &AVLTree<K, V>) -> bool {
        if self.size() != t.size() {
            return false;
        }
        self.root.is_isomorphic(t.root)
    }

    fn bst_check(&self) -> bool {
        let mut iter = self.iter();
        let first = iter.next();
        if first.is_none() {
            return iter.size_hint().0 == self.size() && self.root.is_null();
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
            return iter.size_hint().0 == self.size() && self.root.is_null();
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

    unsafe fn remove_node(&mut self, node: NodePtr<K, V>) {
        if node.is_null() {
            return;
        }
        self.erase_node(node);
        node.set_parent(node);
        self.count -= 1;
    }

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

    pub fn get_ref(&self, what: &K) -> Option<&V> {
        unsafe {
            let node = self.find_node(what);
            if node.is_null() {
                None
            } else {
                Some(&(*node.0).value)
            }
        }
    }

    pub fn get_mut(&mut self, what: &K) -> Option<&mut V> {
        unsafe {
            let node = self.find_node(what);
            if node.is_null() {
                None
            } else {
                Some(&mut (*node.0).value)
            }
        }
    }

    unsafe fn erase_node(&mut self, mut node: NodePtr<K, V>) {
        let parent = if node.left().not_null() && node.right().not_null() {
            let old = node;
            node = node.right();
            while node.left().not_null() {
                node = node.left();
            }
            let child = node.right();
            let mut parent = node.parent();
            if child.not_null() {
                child.set_parent(parent);
            }
            self.child_replace(node, child, parent);
            if node.parent() == old {
                parent = node;
            }
            node.set_left(old.left());
            node.set_right(old.right());
            node.set_parent(old.parent());
            node.set_height(old.height());
            self.child_replace(old, node, old.parent());
            old.left().set_parent(node);
            if old.right().not_null() {
                old.right().set_parent(node);
            }
            parent
        } else {
            let child = if node.left().is_null() {
                node.right()
            } else {
                node.left()
            };
            let parent = node.parent();
            self.child_replace(node, child, parent);
            if child.not_null() {
                child.set_parent(parent);
            }
            parent
        };
        if parent.not_null() {
            self.rebalance_node(parent);
        }
    }

    #[inline]
    unsafe fn rebalance_node(&mut self, mut node: NodePtr<K, V>) {
        while node.not_null() {
            let h0 = node.left_height();
            let h1 = node.right_height();
            let diff = h0 - h1;
            let height = max(h0, h1) + 1;
            if node.height() != height {
                break;
            } else if diff >= -1 && diff <= 1 {
                break;
            }
            if diff <= -2 {
                node = self.node_fix_l(node);
            } else if diff >= 2 {
                node = self.node_fix_r(node);
            }
            node = node.parent();
        }
    }

    #[inline]
    fn drop_node(node: NodePtr<K, V>) {
        if node.not_null() {
            AVLTree::drop_node(node.left());
            AVLTree::drop_node(node.right());
            node.destroy();
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        AVLTree::drop_node(self.root);
        self.root = NodePtr::null();
        self.count = 0;
    }

    #[inline]
    pub fn insert_or_replace(&mut self, key: K, mut value: V) -> Option<V> {
        unsafe {
            let (duplicate, parent, cmp_node_ref) = self.find_duplicate(&key);
            if duplicate.is_null() {
                self.link_post_insert(key, value, parent, cmp_node_ref);
                None
            } else {
                mem::swap(&mut value, &mut (*duplicate.0).value);
                Some(value)
            }
        }
    }

    #[inline]
    pub fn keys(&self) -> Keys<K, V> {
        Keys { inner: self.iter() }
    }

    #[inline]
    pub fn values(&self) -> Values<K, V> {
        Values { inner: self.iter() }
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        ValuesMut { inner: self.iter_mut() }
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
        self.root = NodePtr::null();
        self.count = 0;
    }
}

#[test]
fn just_for_compile() {}

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
}

impl<'a, K: Ord, V> Clone for Keys<'a, K, V> {
    fn clone(&self) -> Keys<'a, K, V> {
        Keys { inner: self.inner.clone() }
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
}

impl<'a, K: Ord, V> Clone for Values<'a, K, V> {
    fn clone(&self) -> Values<'a, K, V> {
        Values { inner: self.inner.clone() }
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
}

impl<'a, K: Ord, V> Clone for ValuesMut<'a, K, V> {
    fn clone(&self) -> ValuesMut<'a, K, V> {
        ValuesMut { inner: self.inner.clone() }
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
    head: NodePtr<K, V>,
    tail: NodePtr<K, V>,
    len: usize,
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
    head: NodePtr<K, V>,
    tail: NodePtr<K, V>,
    len: usize,
    _marker: marker::PhantomData<&'a ()>,
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

        let (k, v) = unsafe { (&(*self.head.0).key, &(*self.head.0).value) };
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
        let (k, v) = unsafe { (&(*self.tail.0).key, &(*self.tail.0).value) };
        self.tail = self.tail.prev();
        self.len -= 1;
        Some((k, v))
    }
}

pub struct IterMut<'a, K: Ord + 'a, V: 'a> {
    head: NodePtr<K, V>,
    tail: NodePtr<K, V>,
    len: usize,
    _marker: marker::PhantomData<&'a ()>,
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

        let (k, v) = unsafe { (&(*self.head.0).key, &mut (*self.head.0).value) };
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
        let (k, v) = unsafe { (&(*self.tail.0).key, &mut (*self.tail.0).value) };
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
        let iter = if self.root.is_null() {
            IntoIter {
                head: NodePtr::null(),
                tail: NodePtr::null(),
                len: 0,
            }
        } else {
            IntoIter {
                head: self.first_node(),
                tail: self.last_node(),
                len: self.size(),
            }
        };
        unsafe { self.set_empty(); }
        iter
    }
}

pub mod test {
    extern crate rand;

    use avl::AVLTree;
    use std::cmp::Ordering;
    use std::collections::HashMap;

    type DefaultType = AVLTree<i32, Option<i32>>;

    #[test]
    fn test_avl_basic() {
        let mut t = DefaultType::new();
        {
            assert!(t.root.is_null());
            t.insert(3, None);
            assert_eq!(*t.root.key_ref(), 3);
            assert_eq!(t.root.height(), 1);
            assert!(t.root.left().is_null());
            assert!(t.root.right().is_null());

            t.insert(2, None);
            assert_eq!(*t.root.key_ref(), 3);
            assert_eq!(t.root.height(), 2);
            assert_eq!(*t.root.left().key_ref(), 2);

            t.insert(1, None);
            assert_eq!(*t.root.key_ref(), 2);
            assert_eq!(t.root.height(), 2);
            assert_eq!(*t.root.left().key_ref(), 1);
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
            assert_eq!(*t.root.key_ref(), 3);
            assert_eq!(t.root.height(), 1);
            t.insert(2, None);
            assert_eq!(*t.root.key_ref(), 3);
            assert_eq!(t.root.height(), 2);
            t.insert(1, None);
            assert_eq!(*t.root.key_ref(), 2);
            assert_eq!(t.root.height(), 2);
        }
    }

    #[test]
    fn test_avl_rotate_left() {
        let mut t = DefaultType::new();
        {
            t.insert(1, None);
            assert_eq!(*t.root.key_ref(), 1);
            assert_eq!(t.root.height(), 1);
            t.insert(2, None);
            assert_eq!(*t.root.key_ref(), 1);
            assert_eq!(t.root.height(), 2);
            t.insert(3, None);
            assert_eq!(*t.root.key_ref(), 2);
            assert_eq!(t.root.height(), 2);
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
            assert_eq!(*t.root.key_ref(), MyData { a: 1 });
            assert_eq!(t.root.height(), 1);
            t.insert(MyData { a: 2 }, None);
            assert_eq!(*t.root.key_ref(), MyData { a: 1 });
            assert_eq!(t.root.height(), 2);

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
        let t = default_build_avl(test_num);
        assert_eq!(t.size(), test_num);
        assert_eq!(t.root.height(), 12);
        let left = t.root.left();
        assert!(left.height() <= 11);
        assert!(left.height() >= 10);
        let right = t.root.right();
        assert!(right.height() <= 11);
        assert!(right.height() >= 10);

        assert!(t.bst_check());
        assert!(t.bst_check_reverse());
    }

    #[test]
    fn test_avl_clear() {
        let test_num = 200usize;
        let mut t = default_build_avl(test_num);
        t.clear();
        assert!(t.empty());
        assert!(t.root.is_null());
    }

    #[test]
    fn test_avl_clone() {
        let test_num = 500usize;
        let ta = default_build_avl(test_num);
        let tb = ta.clone();
        assert!(ta.is_isomorphic(&tb));
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
}

