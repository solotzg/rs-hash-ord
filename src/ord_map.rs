use std::cmp::Ordering;
use std::marker;
use std::mem;
use std::ops::Index;
use std::iter::FromIterator;
use avl_node::{AVLNodePtr, AVLNode, AVLNodePtrBase, AVLRoot, AVLRootPtr};
use avl_node;
use std::ptr;
use fastbin::Fastbin;
use fastbin::VoidPtr;
use std::borrow::Borrow;

pub struct AVLEntry<K, V> {
    node: AVLNode,
    key: K,
    value: V,
}

trait AVLEntryOperation<K, V> {
    fn key(self) -> *mut K;
    fn value(self) -> *mut V;
    fn node_ptr(self) -> AVLNodePtr;
}

impl<K, V> AVLEntryOperation<K, V> for *mut AVLEntry<K, V> {
    fn key(self) -> *mut K {
        unsafe { &mut (*self).key as *mut K }
    }

    fn value(self) -> *mut V {
        unsafe { &mut (*self).value as *mut V }
    }

    fn node_ptr(self) -> AVLNodePtr {
        unsafe { &mut (*self).node as AVLNodePtr }
    }
}

trait AVLTreeNodeOperation {
    fn key_ref<'a, K, V>(self) -> &'a K;
    fn value_ref<'a, K, V>(self) -> &'a V;
    fn value_mut<'a, K, V>(self) -> &'a mut V;
    fn set_value<K, V>(self, value: V);
    fn avl_node_deref_to_entry<K, V>(self) -> *mut AVLEntry<K, V>;
}

impl AVLTreeNodeOperation for *mut AVLNode {
    #[inline]
    fn key_ref<'a, K, V>(self) -> &'a K {
        unsafe { &(*self.avl_node_deref_to_entry::<K, V>()).key }
    }

    #[inline]
    fn value_ref<'a, K, V>(self) -> &'a V {
        unsafe { &(*self.avl_node_deref_to_entry::<K, V>()).value }
    }

    #[inline]
    fn value_mut<'a, K, V>(self) -> &'a mut V {
        unsafe { &mut (*self.avl_node_deref_to_entry::<K, V>()).value }
    }

    #[inline]
    fn set_value<K, V>(self, value: V) {
        unsafe { (*self.avl_node_deref_to_entry::<K, V>()).value = value; }
    }

    #[inline]
    fn avl_node_deref_to_entry<K, V>(self) -> *mut AVLEntry<K, V> {
        container_of!(self, AVLEntry<K, V>, node)
    }
}

pub struct Cursors<'a, K, V> where K: Ord + 'a, V: 'a {
    tree_mut: &'a mut OrdMap<K, V>,
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

    pub fn get(&self) -> Option<(&K, &V)> {
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
            f(self.tree_mut.remove_node(node));
        }
    }

    pub fn erase_then_next<F>(&mut self, f: F) where F: Fn(Option<(K, V)>) {
        self.erase(f, CursorsOperation::NEXT);
    }

    pub fn erase_then_prev<F>(&mut self, f: F) where F: Fn(Option<(K, V)>) {
        self.erase(f, CursorsOperation::PREV);
    }
}

pub struct OrdMap<K, V> where K: Ord {
    root: AVLRoot,
    count: usize,
    entry_fastbin: Fastbin,
    _marker: marker::PhantomData<(K, V)>,
}

impl<K, V> OrdMap<K, V> where K: Ord {
    #[inline]
    pub fn find_cursors<Q>(&mut self, q: &Q) -> Cursors<K, V> where K: Borrow<Q>, Q: Ord {
        let node = self.find_node(q);
        Cursors { tree_mut: self, pos: node }
    }

    #[inline]
    pub fn max_height(&self) -> i32 {
        self.root.node.height()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
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
        OrdMap {
            root: Default::default(),
            count: 0,
            entry_fastbin: Fastbin::new(mem::size_of::<AVLEntry<K, V>>()),
            _marker: marker::PhantomData,
        }
    }

    #[inline]
    fn entry_alloc(&mut self, key: K, value: V) -> *mut AVLEntry<K, V> {
        let entry = self.entry_fastbin.alloc() as *mut AVLEntry<K, V>;
        debug_assert!(!entry.is_null());
        unsafe {
            ptr::write(entry.key(), key);
            ptr::write(entry.value(), value);
        }
        entry
    }

    fn deep_clone_node(&mut self, parent: AVLNodePtr, other_node: AVLNodePtr) -> AVLNodePtr where K: Clone, V: Clone {
        if other_node.is_null() {
            return ptr::null_mut();
        }
        let entry = self.entry_alloc(
            (*other_node.key_ref::<K, V>()).clone(),
            (*other_node.value_ref::<K, V>()).clone(),
        );
        let node = entry.node_ptr();
        node.reset(
            self.deep_clone_node(node, other_node.left()),
            self.deep_clone_node(node, other_node.right()),
            parent,
            other_node.height(),
        );
        node
    }

    pub fn clone_from(t: &OrdMap<K, V>) -> Self where K: Clone, V: Clone {
        let mut tree = OrdMap {
            root: Default::default(),
            count: 0,
            entry_fastbin: Fastbin::new(mem::size_of::<AVLEntry<K, V>>()),
            _marker: marker::PhantomData,
        };
        tree.root.node = tree.deep_clone_node(ptr::null_mut(), t.root.node);
        tree.count = t.count;
        tree
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
    pub fn find_node<Q: ? Sized>(&self, q: &Q) -> AVLNodePtr where K: Borrow<Q>, Q: Ord {
        let mut node = self.root.node;
        while node.not_null() {
            match q.cmp(node.key_ref::<K, V>().borrow()) {
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
    fn isomorphic(&self, t: &OrdMap<K, V>) -> bool {
        if self.len() != t.len() {
            return false;
        }
        self.root.node.isomorphic(t.root.node)
    }

    pub fn check_valid(&self) -> bool {
        self.root.node.check_valid()
    }

    fn bst_check(&self) -> bool {
        let mut iter = self.iter();
        let first = iter.next();
        if first.is_none() {
            return iter.size_hint().0 == self.len() && self.root.node.is_null();
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
        cnt == self.len()
    }

    fn bst_check_reverse(&self) -> bool {
        let mut iter = self.iter();
        let first = iter.next_back();
        if first.is_none() {
            return iter.size_hint().0 == self.len() && self.root.node.is_null();
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
        cnt == self.len()
    }

    #[inline]
    unsafe fn remove_node(&mut self, node: AVLNodePtr) -> Option<(K, V)> {
        if node.is_null() || node.empty() {
            return None;
        }
        avl_node::erase_node(node, self.get_root_ptr());
        node.set_parent(node);
        self.count -= 1;
        let old_entry = node.avl_node_deref_to_entry::<K, V>();
        let res = Some((ptr::read(old_entry.key()), ptr::read(old_entry.value())));
        self.entry_fastbin.del(old_entry as VoidPtr);
        res
    }

    #[inline]
    pub fn remove<Q: ? Sized>(&mut self, q: &Q) -> Option<(K, V)> where K: Borrow<Q>, Q: Ord {
        let node = self.find_node(q);
        unsafe { self.remove_node(node) }
    }

    #[inline]
    pub fn contains_key<Q: ? Sized>(&self, q: &Q) -> bool where K: Borrow<Q>, Q: Ord {
        self.find_node(q).not_null()
    }

    #[inline]
    pub fn get<Q: ? Sized>(&self, q: &Q) -> Option<&V> where K: Borrow<Q>, Q: Ord {
        let node = self.find_node(q);
        if node.is_null() {
            None
        } else {
            Some(node.value_ref::<K, V>())
        }
    }

    pub fn get_mut<Q: ? Sized>(&mut self, q: &Q) -> Option<&mut V> where K: Borrow<Q>, Q: Ord {
        let node = self.find_node(q);
        if node.is_null() {
            None
        } else {
            Some(node.value_mut::<K, V>())
        }
    }

    fn recursive_drop_node(&mut self, node: AVLNodePtr) {
        if node.left().not_null() {
            self.recursive_drop_node(node.left());
        }
        if node.right().not_null() {
            self.recursive_drop_node(node.right());
        }
        let entry = node.avl_node_deref_to_entry::<K, V>();
        if mem::needs_drop::<AVLEntry<K, V>>() {
            unsafe { ptr::drop_in_place(entry); }
        }
        self.entry_fastbin.del(entry as VoidPtr);
    }

    #[inline]
    pub fn clear(&mut self) {
        let node = self.root.node;
        if node.not_null() {
            self.recursive_drop_node(node);
        }
        self.root.node = ptr::null_mut();
        self.count = 0;
    }

    #[inline]
    fn destroy(&mut self) {
        self.clear();
    }

    #[inline]
    pub fn link_post_insert(&mut self, new_node: AVLNodePtr, parent: AVLNodePtr, cmp_node_ref: *mut AVLNodePtr) {
        unsafe { avl_node::link_node(new_node, parent, cmp_node_ref); }
        unsafe { avl_node::node_post_insert(new_node, self.get_root_ptr()); }
        self.count += 1;
    }

    #[inline]
    fn get_root_ptr(&mut self) -> AVLRootPtr {
        &mut self.root as AVLRootPtr
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        let (duplicate, parent, cmp_node_ref) = self.find_duplicate(&key);
        let entry = self.entry_alloc(key, value);
        if duplicate.is_null() {
            self.link_post_insert(entry.node_ptr(), parent, cmp_node_ref);
            None
        } else {
            unsafe {
                let old_entry = duplicate.avl_node_deref_to_entry::<K, V>();
                avl_node::avl_node_replace(duplicate, entry.node_ptr(), self.get_root_ptr());
                let res = Some((ptr::read(old_entry.key()), ptr::read(old_entry.value())));
                self.entry_fastbin.del(old_entry as VoidPtr);
                res
            }
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
            len: self.len(),
            _marker: marker::PhantomData,
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            head: self.first_node(),
            tail: self.last_node(),
            len: self.len(),
            _marker: marker::PhantomData,
        }
    }
}

impl<K, V> Drop for OrdMap<K, V> where K: Ord {
    fn drop(&mut self) {
        self.destroy();
    }
}

impl<K, V> Clone for OrdMap<K, V> where K: Ord + Clone, V: Clone {
    fn clone(&self) -> Self {
        OrdMap::clone_from(self)
    }
}

impl<K, V> PartialEq for OrdMap<K, V> where K: Eq + Ord, V: PartialEq, {
    fn eq(&self, other: &OrdMap<K, V>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().all(|(key, value)| {
            other.get(key).map_or(false, |v| *value == *v)
        })
    }
}

impl<K, V> Eq for OrdMap<K, V> where K: Eq + Ord, V: Eq {}

impl<'a, K, V> Index<&'a K> for OrdMap<K, V> where K: Ord {
    type Output = V;

    #[inline]
    fn index(&self, key: &K) -> &V {
        self.get(key).expect("no entry found for key")
    }
}

impl<K, V> FromIterator<(K, V)> for OrdMap<K, V> where K: Ord {
    fn from_iter<T: IntoIterator<Item=(K, V)>>(iter: T) -> OrdMap<K, V> {
        let mut tree = OrdMap::new();
        tree.extend(iter);
        tree
    }
}

impl<K, V> Extend<(K, V)> for OrdMap<K, V> where K: Ord {
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

pub struct IntoIter<K, V> where K: Ord {
    head: AVLNodePtr,
    tail: AVLNodePtr,
    len: usize,
    entry_fastbin: Fastbin,
    _marker: marker::PhantomData<(K, V)>,
}

impl<K, V> IntoIter<K, V> where K: Ord {
    fn remove(&mut self, node: AVLNodePtr) -> Option<(K, V)> {
        let parent = node.parent();
        if parent.not_null() {
            if parent.left() == node {
                parent.set_left(ptr::null_mut());
            } else {
                parent.set_right(ptr::null_mut())
            }
        }
        self.len -= 1;
        let old_entry = node.avl_node_deref_to_entry::<K, V>();
        let res = unsafe { Some((ptr::read(old_entry.key()), ptr::read(old_entry.value()))) };
        self.entry_fastbin.del(old_entry as VoidPtr);
        res
    }
}

impl<K: Ord, V> Drop for IntoIter<K, V> {
    fn drop(&mut self) {
        for (_, _) in self {}
    }
}

impl<K: Ord, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 || self.tail.is_null() {
            return None;
        }
        let node = self.tail;
        self.tail = self.tail.prev();
        self.remove(node)
    }
}

impl<K, V> Iterator for IntoIter<K, V> where K: Ord {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 || self.head.is_null() {
            return None;
        }
        let node = self.head;
        self.head = self.head.next();
        self.remove(node)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<K, V> IntoIterator for OrdMap<K, V> where K: Ord {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(mut self) -> IntoIter<K, V> {
        let iter = if self.root.node.is_null() {
            IntoIter {
                head: ptr::null_mut(),
                tail: ptr::null_mut(),
                len: 0,
                entry_fastbin: Default::default(),
                _marker: marker::PhantomData,
            }
        } else {
            IntoIter {
                head: self.first_node(),
                tail: self.last_node(),
                len: self.len(),
                entry_fastbin: self.entry_fastbin.move_to(),
                _marker: marker::PhantomData,
            }
        };
        iter
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

#[cfg(test)]
pub mod test {
    extern crate rand;

    use ord_map::OrdMap;
    use std::cmp::Ordering;
    use ord_map::AVLTreeNodeOperation;
    use avl_node::AVLNodePtrBase;
    use std::cell::RefCell;

    type DefaultType = OrdMap<i32, Option<i32>>;

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
            match t.remove(&x) {
                None => {}
                Some((k, v)) => {
                    assert_eq!(v.unwrap(), -x);
                    assert_eq!(k, x);
                }
            }
            assert!(t.find_node(&x).is_null());
        }
        assert!(t.bst_check());
        assert!(t.bst_check_reverse());
        assert!(t.check_valid());
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

        let mut t = OrdMap::<MyData, Option<i32>>::new();
        {
            t.insert(MyData { a: 1 }, None);
            assert_eq!(*t.root.node.key_ref::<MyData, Option<i32>>(), MyData { a: 1 });
            assert_eq!(t.root.node.height(), 1);
            t.insert(MyData { a: 2 }, None);
            assert_eq!(*t.root.node.key_ref::<MyData, Option<i32>>(), MyData { a: 1 });
            assert_eq!(t.root.node.height(), 2);

            *t.get_mut(&MyData { a: 1 }).unwrap() = Some(23333);
            assert_eq!((*t.get(&MyData { a: 1 }).unwrap()).unwrap(), 23333);
        }
    }

    #[test]
    fn test_avl_find() {
        let t = default_build_avl(1000);
        for num in 0..t.len() {
            let x = num as i32;
            assert_eq!(*t.get(&x).unwrap(), Some(-x));
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
        assert_eq!(t.len(), 0);
        for d in &v {
            t.insert(*d, Some(-*d));
        }
        t
    }

    #[test]
    fn test_avl_validate() {
        let test_num = 1000usize;
        let mut t = OrdMap::new();
        for i in 0..test_num {
            t.insert(i, i);
        }
        assert_eq!(t.len(), test_num);
        assert_eq!(t.root.node.height(), 10);
        let left = t.root.node.left();
        assert_eq!(left.height(), 9);
        let right = t.root.node.right();
        assert_eq!(right.height(), 9);

        assert!(t.bst_check());
        assert!(t.bst_check_reverse());
        assert!(t.check_valid());
    }

    #[test]
    fn test_avl_clear() {
        struct Node<'a> {
            b: &'a RefCell<i32>,
        }
        impl<'a> Drop for Node<'a> {
            fn drop(&mut self) {
                *self.b.borrow_mut() += 1;
            }
        }
        let cnt = RefCell::new(0);
        let test_num = 200;
        let mut map = OrdMap::new();
        for i in 0..test_num {
            map.insert(i, Node { b: &cnt });
        }
        assert_eq!(*cnt.borrow(), 0);
        map.clear();
        assert_eq!(*cnt.borrow(), test_num);
    }

    #[test]
    fn test_avl_clone_eq() {
        let test_num = 100usize;
        let ta = default_build_avl(test_num);
        let tb = ta.clone();
        assert!(ta.isomorphic(&tb));
        assert!(ta == tb);

        let ta = OrdMap::<i32, i32>::new();
        let tb = OrdMap::<i32, i32>::new();
        assert!(ta.isomorphic(&tb));
        assert!(ta == tb);
    }

    #[test]
    fn test_avl_iteration() {
        let v = default_make_avl_element(100);
        let mut t = OrdMap::new();
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
        let mut a = OrdMap::new();
        a.insert(2, 2);
        let mut b = OrdMap::new();
        b.insert(1, 1);
        b.insert(3, 3);
        a.extend(b.into_iter());
        assert_eq!(a.len(), 3);
        assert_eq!(a[&1], 1);
        assert_eq!(a[&2], 2);
        assert_eq!(a[&3], 3);
    }

    #[test]
    fn test_avl_keys() {
        let mut v = default_make_avl_element(100);
        let mut t = OrdMap::new();
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
    fn test_avl_values_index() {
        let mut v = default_make_avl_element(100);
        let mut t = OrdMap::new();
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
            let mut cursors = t.find_cursors(&50);
            assert_eq!(*cursors.get().unwrap().0, 50);
            for _ in 0..10 {
                cursors.next();
            }
            assert_eq!(*cursors.get().unwrap().0, 60);
            for _ in 0..5 {
                cursors.prev();
            }
            assert_eq!(*cursors.get().unwrap().0, 55);
            cursors.erase_then_next(
                |x| {
                    assert!(x.is_some());
                    assert_eq!(x.unwrap().0, 55);
                }
            );
            assert_eq!(*cursors.get().unwrap().0, 56);
            cursors.prev();
            assert_eq!(*cursors.get().unwrap().0, 54);
            cursors.erase_then_prev(
                |x| {
                    assert!(x.is_some());
                    assert_eq!(x.unwrap().0, 54);
                }
            );
            assert_eq!(*cursors.get().unwrap().0, 53);
            cursors.next();
            assert_eq!(*cursors.get().unwrap().0, 56);

            *cursors.get_mut().unwrap().1 = None;
            assert_eq!(*cursors.get().unwrap().1, None);

            cursors.erase_then_prev(|_| {});
        }
        assert_eq!(t.len(), 97);
        {
            let cursors = t.find_cursors(&55);
            assert!(cursors.get().is_none());
        }
    }

    #[test]
    fn test_memory_leak() {
        struct Node<'a> {
            b: &'a RefCell<i32>,
        }
        impl<'a> Drop for Node<'a> {
            fn drop(&mut self) {
                *self.b.borrow_mut() += 1;
            }
        }
        let cnt = RefCell::new(0);
        let test_num = 111;
        let mut map = OrdMap::new();
        for i in 0..test_num {
            map.insert(i, Node { b: &cnt });
        }
        for i in 0..test_num / 2 {
            map.remove(&i);
        }
        assert_eq!(*cnt.borrow(), test_num / 2);
        for i in test_num / 2..test_num {
            map.insert(i, Node { b: &cnt });
        }
        assert_eq!(*cnt.borrow(), test_num);
        map.clear();
        assert_eq!(*cnt.borrow(), test_num * 2 - test_num / 2);
    }

    #[test]
    fn test_from_iter() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];
        let map: OrdMap<_, _> = xs.iter().cloned().collect();
        for &(k, v) in &xs {
            assert_eq!(map.get(&k), Some(&v));
        }
    }
}

