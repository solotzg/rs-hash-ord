use fastbin::{Fastbin, VoidPtr};
use hash_table::{HashNode, HashTable, HashUint};
use hash_table;
use hash_table::HashNodePtrOperation;
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::mem;
use std::ptr;
use avl_node::{AVLNodePtrBase, AVLNodePtr};
use hash_table::HashIndexPtrOperation;
use hash_table::HashNodeOperation;
use list::ListHeadPtrFn;
use std::cmp::Ordering;
use avl_node;
use std::ops::Index;
use std::borrow::Borrow;
use std::iter::FromIterator;

pub struct HashMap<K, V, S = RandomState> {
    entry_fastbin: Fastbin,
    kv_fastbin: Fastbin,
    hash_table: Box<HashTable<K, V>>,
    hash_builder: S,
}

struct HashEntry<K, V> {
    node: HashNode<K>,
    value: *mut V,
}

pub struct Keys<'a, K, V, S> where K: 'a, V: 'a, S: 'a {
    inner: Iter<'a, K, V, S>,
}

impl<'a, K, V, S> Iterator for Keys<'a, K, V, S> where K: 'a, V: 'a, S: 'a {
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

pub struct Values<'a, K, V, S> where K: 'a, V: 'a, S: 'a {
    inner: Iter<'a, K, V, S>,
}

impl<'a, K, V, S> Iterator for Values<'a, K, V, S> where K: 'a, V: 'a, S: 'a {
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

pub struct ValuesMut<'a, K, V, S> where K: 'a, V: 'a, S: 'a {
    inner: IterMut<'a, K, V, S>,
}

impl<'a, K, V, S> Iterator for ValuesMut<'a, K, V, S> where K: 'a, V: 'a, S: 'a {
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

pub struct Iter<'a, K, V, S> where K: 'a, V: 'a, S: 'a {
    inner: *mut HashEntry<K, V>,
    map: &'a HashMap<K, V, S>,
    len: usize,
}

impl<'a, K, V, S> Iterator for Iter<'a, K, V, S> where K: 'a, V: 'a, S: 'a {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        let entry = self.inner;
        if entry.is_null() || self.len == 0 {
            return None;
        }
        let res = unsafe { Some((&(*entry.key()), &(*entry.value()))) };
        self.inner = self.map.next(entry);
        self.len -= 1;
        res
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

pub struct IterMut<'a, K, V, S> where K: 'a, V: 'a, S: 'a {
    inner: *mut HashEntry<K, V>,
    map: &'a HashMap<K, V, S>,
    len: usize,
}

impl<'a, K, V, S> Iterator for IterMut<'a, K, V, S> where K: 'a, V: 'a, S: 'a {
    type Item = (&'a K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        let entry = self.inner;
        if entry.is_null() || self.len == 0 {
            return None;
        }
        let res = unsafe { Some((&(*entry.key()), &mut (*entry.value()))) };
        self.inner = self.map.next(entry);
        self.len -= 1;
        res
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

#[inline]
fn key_deref_to_kv<K, V>(key: *const K) -> *mut (K, V) {
    container_of!(key, (K, V), 0)
}

trait HashEntryBase<K, V> {
    fn node_ptr(self) -> *mut HashNode<K>;
    fn value(self) -> *mut V;
    fn set_value(self, value: *mut V);
    fn key(self) -> *const K;
    fn set_key(self, key: *const K);
    fn set_hash_value(self, hash_value: HashUint);
}

impl<K, V> HashEntryBase<K, V> for *mut HashEntry<K, V> {
    #[inline]
    fn node_ptr(self) -> *mut HashNode<K> {
        unsafe { &mut (*self).node as *mut HashNode<K> }
    }
    #[inline]
    fn value(self) -> *mut V {
        unsafe { (*self).value }
    }
    #[inline]
    fn set_value(self, value: *mut V) {
        unsafe { (*self).value = value; }
    }
    #[inline]
    fn key(self) -> *const K {
        unsafe { (*self).node.key }
    }
    #[inline]
    fn set_key(self, key: *const K) {
        unsafe { (*self).node.key = key; }
    }
    #[inline]
    fn set_hash_value(self, hash_value: HashUint) {
        unsafe { (*self).node.hash_val = hash_value; }
    }
}

trait HashNodeDerefToHashEntry<K, V> {
    fn deref_to_hash_entry(self) -> *mut HashEntry<K, V>;
}

impl<K, V> HashNodeDerefToHashEntry<K, V> for *mut HashNode<K> {
    fn deref_to_hash_entry(self) -> *mut HashEntry<K, V> {
        container_of!(self, HashEntry<K, V>, node)
    }
}

#[inline]
unsafe fn find_duplicate_hash_node<K>(mut link: *mut AVLNodePtr, new_key: *const K, hash_val: HashUint)
                                      -> (*mut HashNode<K>, AVLNodePtr, *mut AVLNodePtr) where K: Ord
{
    let mut parent = ptr::null_mut();
    let mut duplicate = ptr::null_mut();
    while !(*link).is_null() {
        parent = *link;
        let snode = parent.avl_hash_deref_mut::<K>();
        let snode_hash = snode.hash_val();
        if hash_val != snode_hash {
            link = if hash_val < snode_hash { parent.left_mut() } else { parent.right_mut() };
        } else {
            match (*new_key).cmp(&(*snode.key_ptr())) {
                Ordering::Equal => {
                    duplicate = snode;
                    break;
                }
                Ordering::Less => { link = parent.left_mut(); }
                Ordering::Greater => { link = parent.right_mut(); }
            }
        }
    }
    (duplicate, parent, link)
}

unsafe fn hash_table_update<K, V>(hash_table: &mut HashTable<K, V>, new_entry: *mut HashEntry<K, V>) -> *mut HashEntry<K, V> where K: Ord + Hash {
    let new_node = new_entry.node_ptr();
    let hash_val = new_node.hash_val();
    let index = hash_table.get_hash_index(hash_val);
    let link = index.avl_root_node_ptr();
    if (*link).is_null() {
        new_node.avl_node_ptr().reset(ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), 1);
        (*link) = new_node.avl_node_ptr();
        hash_table.head_ptr().list_add_tail(index.node_ptr());
        hash_table.inc_count(1);
        return ptr::null_mut();
    }
    let (duplicate, parent, link) = find_duplicate_hash_node(link, new_entry.key(), hash_val);
    if !duplicate.is_null() {
        let old_entry = duplicate.deref_to_hash_entry();
        avl_node::avl_node_replace(duplicate.avl_node_ptr(), new_node.avl_node_ptr(), index.avl_root_ptr());
        return old_entry;
    }
    debug_assert_ne!(parent, new_node.avl_node_ptr());
    debug_assert!(!new_entry.is_null());
    avl_node::link_node(new_node.avl_node_ptr(), parent, link);
    avl_node::node_post_insert(new_node.avl_node_ptr(), index.avl_root_ptr());
    hash_table.inc_count(1);
    ptr::null_mut()
}

#[inline]
fn entry_alloc<K, V>(entry_fastbin: &mut Fastbin, key: *const K, value: *mut V, hash_value: HashUint) -> *mut HashEntry<K, V> {
    let entry = entry_fastbin.alloc() as *mut HashEntry<K, V>;
    debug_assert!(!entry.is_null());
    entry.set_value(value);
    entry.set_key(key);
    entry.set_hash_value(hash_value);
    entry
}

#[inline]
fn kv_alloc<K, V>(kv_fastbin: &mut Fastbin, key: K, value: V) -> *mut (K, V) {
    let kv = kv_fastbin.alloc() as *mut (K, V);
    unsafe {
        let key_ptr = &mut (*kv).0 as *mut K;
        let value_ptr = &mut (*kv).1 as *mut V;
        ptr::write(key_ptr, key);
        ptr::write(value_ptr, value);
    }
    kv
}

pub enum Entry<'a, K, V, S> where K: 'a, V: 'a, S: 'a {
    Occupied(OccupiedEntry<'a, K, V, S>),
    Vacant(VacantEntry<'a, K, V, S>),
}

pub struct OccupiedEntry<'a, K, V, S> where K: 'a, V: 'a, S: 'a {
    key: Option<K>,
    hash_entry: *mut HashEntry<K, V>,
    hash_map_mut: &'a mut HashMap<K, V, S>,
}

pub struct VacantEntry<'a, K, V, S> where K: 'a, V: 'a, S: 'a {
    hash: HashUint,
    key: K,
    duplicate: *mut HashNode<K>,
    parent: *mut HashNode<K>,
    link: AVLNodePtr,
    hash_map_mut: &'a mut HashMap<K, V, S>,
}

impl<K, V, S> HashMap<K, V, S> {
    fn recurse_destroy<F>(&mut self, node: avl_node::AVLNodePtr, f: &mut F) where F: FnMut((K, V)) {
        if node.left().not_null() {
            self.recurse_destroy(node.left(), f);
        }
        if node.right().not_null() {
            self.recurse_destroy(node.right(), f);
        }
        let hash_node = node.avl_hash_deref_mut::<K>();
        let entry: *mut HashEntry<K, V> = hash_node.deref_to_hash_entry();
        self.entry_fastbin.del(entry as VoidPtr);
        let kv_ptr = key_deref_to_kv::<K, V>(hash_node.key_ptr());
        unsafe { (*f)(ptr::read(kv_ptr)) };
        self.kv_fastbin.del(kv_ptr as VoidPtr);
        self.hash_table.dec_count(1);
    }

    pub fn clear(&mut self) {
        let mut destroy_callback = |_| {};
        loop {
            let node = self.hash_table.pop_first_index();
            if node.is_null() { break; }
            self.recurse_destroy(node, &mut destroy_callback);
        }
        debug_assert_eq!(self.hash_table.size(), 0);
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.hash_table.capacity()
    }

    #[inline]
    pub fn get_max_node_of_single_index(&self) -> i32 {
        self.hash_table.get_max_node_of_single_index()
    }

    #[inline]
    fn first(&self) -> *mut HashEntry<K, V> {
        let hash_node = self.hash_table.node_first();
        if hash_node.is_null() {
            return ptr::null_mut();
        }
        hash_node.deref_to_hash_entry()
    }

    #[inline]
    fn last(&self) -> *mut HashEntry<K, V> {
        let hash_node = self.hash_table.node_last();
        if hash_node.is_null() {
            return ptr::null_mut();
        }
        hash_node.deref_to_hash_entry()
    }

    #[inline]
    fn next(&self, entry: *mut HashEntry<K, V>) -> *mut HashEntry<K, V> {
        let hash_node = self.hash_table.node_next(entry.node_ptr());
        if hash_node.is_null() {
            return ptr::null_mut();
        }
        hash_node.deref_to_hash_entry()
    }

    #[inline]
    fn prev(&self, entry: *mut HashEntry<K, V>) -> *mut HashEntry<K, V> {
        let hash_node = self.hash_table.node_prev(entry.node_ptr());
        if hash_node.is_null() {
            return ptr::null_mut();
        }
        hash_node.deref_to_hash_entry()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.hash_table.size()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn entry_alloc(&mut self, key: *const K, value: *mut V, hash_value: HashUint) -> *mut HashEntry<K, V> {
        entry_alloc(&mut self.entry_fastbin, key, value, hash_value)
    }

    #[inline]
    fn kv_alloc(&mut self, key: K, value: V) -> *mut (K, V) {
        kv_alloc(&mut self.kv_fastbin, key, value)
    }

    pub fn keys(&self) -> Keys<K, V, S> {
        Keys { inner: self.iter() }
    }

    pub fn values(&self) -> Values<K, V, S> {
        Values { inner: self.iter() }
    }

    pub fn values_mut(&mut self) -> ValuesMut<K, V, S> {
        ValuesMut { inner: self.iter_mut() }
    }

    pub fn iter(&self) -> Iter<K, V, S> {
        Iter { inner: self.first(), map: self, len: self.len() }
    }

    fn iter_mut(&mut self) -> IterMut<K, V, S> {
        IterMut { inner: self.first(), map: self, len: self.len() }
    }
}

impl<K, V, S> HashMap<K, V, S> where K: Ord + Hash, S: BuildHasher {
    #[inline]
    fn make_hash<X: ? Sized>(&self, x: &X) -> HashUint where X: Hash {
        hash_table::make_hash(&self.hash_builder, x)
    }

    pub fn with_hasher(hash_builder: S) -> Self {
        HashMap::with_capacity_and_hasher(0, hash_builder)
    }

    fn erase(&mut self, entry: *mut HashEntry<K, V>) -> Option<(K, V)> {
        debug_assert!(!entry.is_null());
        debug_assert!(!entry.node_ptr().avl_node_ptr().empty());
        self.hash_table.hash_erase(entry.node_ptr());
        entry.node_ptr().avl_node_ptr().init();
        let kv = key_deref_to_kv::<K, V>(entry.key());
        entry.node_ptr().set_key_ptr(ptr::null());
        entry.set_value(ptr::null_mut());
        self.entry_fastbin.del(entry as VoidPtr);
        let res = unsafe { Some(ptr::read(kv)) };
        self.kv_fastbin.del(kv as VoidPtr);
        res
    }

    #[inline]
    fn find<Q: ? Sized>(&self, q: &Q) -> *mut HashEntry<K, V> where K: Borrow<Q>, Q: Ord + Hash {
        let node = self.hash_table.hash_find(self.make_hash(q), q);
        if node.is_null() {
            ptr::null_mut()
        } else {
            node.deref_to_hash_entry()
        }
    }


    #[inline]
    pub fn get<Q: ? Sized>(&self, q: &Q) -> Option<&V> where K: Borrow<Q>, Q: Hash + Ord {
        let entry = self.find(q);
        if entry.is_null() {
            return None;
        }
        unsafe { Some(&(*entry.value())) }
    }

    #[inline]
    pub fn get_mut<Q: ? Sized>(&mut self, q: &Q) -> Option<&mut V> where K: Borrow<Q>, Q: Hash + Ord {
        let entry = self.find(q);
        if entry.is_null() {
            return None;
        }
        unsafe { Some(&mut (*entry.value())) }
    }

    #[inline]
    fn rehash(&mut self, capacity: usize) {
        self.hash_table.rehash(capacity);
    }

    pub fn reserve(&mut self, capacity: usize) {
        self.rehash(capacity);
    }

    pub fn contains_key<Q: ? Sized>(&self, q: &Q) -> bool where K: Borrow<Q>, Q: Hash + Ord {
        !self.find(q).is_null()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        let hash_value = self.make_hash(&key);
        let kv_ptr = self.kv_alloc(key, value);
        let new_entry = unsafe { self.entry_alloc(&(*kv_ptr).0 as *const K, &mut (*kv_ptr).1 as *mut V, hash_value) };
        let old_entry = unsafe { hash_table_update(self.hash_table.as_mut(), new_entry) };
        let cap = self.hash_table.size();
        self.rehash(cap);
        if old_entry.is_null() {
            None
        } else {
            let old_kv_ptr = key_deref_to_kv(old_entry.key());
            let res = unsafe { Some(ptr::read(old_kv_ptr)) };
            self.kv_fastbin.del(old_kv_ptr as VoidPtr);
            res
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<(K, V)> {
        let entry = self.find(key);
        if entry.is_null() {
            return None;
        }
        self.erase(entry)
    }

    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> HashMap<K, V, S> {
        let mut hash_map = HashMap {
            entry_fastbin: Fastbin::new(mem::size_of::<HashEntry<K, V>>()),
            kv_fastbin: Fastbin::new(mem::size_of::<(K, V)>()),
            hash_table: hash_table::HashTable::new_with_box(),
            hash_builder,
        };
        hash_map.rehash(capacity);
        hash_map
    }

    pub fn shrink_to_fit(&mut self) {
        let limit = hash_table::calc_limit(self.len());
        let old_index_size = self.hash_table.index_size();
        let mut new_index_size = old_index_size;
        while new_index_size / 2 >= limit {
            new_index_size /= 2;
        }
        if new_index_size >= old_index_size { return; }
        let mut new_entry_fastbin = Fastbin::new(mem::size_of::<HashEntry<K, V>>());
        let mut new_kv_fastbin = Fastbin::new(mem::size_of::<(K, V)>());
        let mut new_hash_table = hash_table::HashTable::new_with_box();
        new_hash_table.rehash(self.len());
        let mut new_kv_vec = Vec::with_capacity(self.len());
        {
            let mut destroy_callback = |(k, v): (K, V)| {
                let kv_ptr = kv_alloc(&mut new_kv_fastbin, k, v);
                new_kv_vec.push(kv_ptr);
            };
            loop {
                let node = self.hash_table.pop_first_index();
                if node.is_null() { break; }
                self.recurse_destroy(node, &mut destroy_callback);
            }
            debug_assert_eq!(self.hash_table.size(), 0);
        }
        for kv_ptr in new_kv_vec {
            unsafe {
                let key_ptr = &(*kv_ptr).0 as *const K;
                let value_ptr = &mut (*kv_ptr).1 as *mut V;
                let entry = entry_alloc(&mut new_entry_fastbin, key_ptr, value_ptr, self.make_hash(&(*key_ptr)));
                hash_table_update(
                    &mut new_hash_table,
                    entry,
                );
            }
        }
        self.kv_fastbin = new_kv_fastbin;
        self.entry_fastbin = new_entry_fastbin;
        self.hash_table = new_hash_table;
    }
}

impl<K, V> HashMap<K, V, RandomState> where K: Hash + Ord {
    #[inline]
    pub fn new() -> HashMap<K, V, RandomState> {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> HashMap<K, V, RandomState> {
        let mut hash_map = HashMap::<K, V, RandomState>::default();
        hash_map.rehash(capacity);
        hash_map
    }
}

impl<K, V, S> Default for HashMap<K, V, S>
    where K: Ord + Hash,
          S: BuildHasher + Default
{
    fn default() -> HashMap<K, V, S> {
        HashMap::with_hasher(Default::default())
    }
}

impl<K, V, S> Drop for HashMap<K, V, S> {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

impl<'a, K, Q, V, S> Index<&'a Q> for HashMap<K, V, S>
    where Q: ? Sized + Hash + Ord, K: Hash + Ord + Borrow<Q>, S: BuildHasher
{
    type Output = V;

    #[inline]
    fn index(&self, q: &Q) -> &Self::Output {
        self.get(q).expect("no entry found for key")
    }
}

impl<K, V, S> Extend<(K, V)> for HashMap<K, V, S>
    where K: Ord + Hash,
          S: BuildHasher
{
    fn extend<T: IntoIterator<Item=(K, V)>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let reserve = if self.is_empty() {
            iter.size_hint().0
        } else {
            (iter.size_hint().0 + 1) / 2 + self.len()
        };
        self.reserve(reserve);
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<K, V, S> IntoIterator for HashMap<K, V, S> where K: Ord + Hash, S: BuildHasher {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V, S>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            entry: self.first(),
            len: self.len(),
            map: self,
        }
    }
}

pub struct IntoIter<K, V, S> where K: Ord + Hash, S: BuildHasher {
    entry: *mut HashEntry<K, V>,
    map: HashMap<K, V, S>,
    len: usize,
}

impl<K, V, S> Drop for IntoIter<K, V, S> where K: Ord + Hash, S: BuildHasher {
    fn drop(&mut self) {
        for (_, _) in self {}
    }
}

impl<K, V, S> Iterator for IntoIter<K, V, S> where K: Ord + Hash, S: BuildHasher {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.entry;
        if self.len == 0 || entry.is_null() {
            return None;
        }
        self.entry = self.map.next(entry);
        self.len -= 1;
        self.map.erase(entry)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<K, V, S> FromIterator<(K, V)> for HashMap<K, V, S>
    where K: Ord + Hash,
          S: BuildHasher + Default
{
    fn from_iter<T: IntoIterator<Item=(K, V)>>(iter: T) -> HashMap<K, V, S> {
        let mut map = HashMap::with_hasher(Default::default());
        map.extend(iter);
        map
    }
}

impl<K, V, S> Clone for HashMap<K, V, S> where K: Ord + Hash + Clone, V: Clone, S: BuildHasher + Clone {
    fn clone(&self) -> Self {
        let mut map = HashMap::with_capacity_and_hasher(
            self.len(),
            self.hash_builder.clone(),
        );
        for (k, v) in self.iter() {
            map.insert(k.clone(), v.clone());
        }
        map
    }
}

impl<K, V, S> PartialEq for HashMap<K, V, S> where K: Ord + Hash, V: PartialEq, S: BuildHasher {
    fn eq(&self, other: &HashMap<K, V, S>) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter().all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<K, V, S> Eq for HashMap<K, V, S> where K: Ord + Hash, V: Eq, S: BuildHasher {}

#[cfg(test)]
mod test {
    use hash_map::HashMap;
    use std::cell::RefCell;

    #[test]
    fn test_hash_map() {
        let mut m = HashMap::new();
        for i in 100..200 {
            m.insert(i, -i);
        }
        assert!(m.contains_key(&100));
        assert_eq!(m[&111], -111);
        assert_eq!(m.len(), 100);
        let mut a = m.first();
        let mut cnt = 0;
        while !a.is_null() {
            cnt += 1;
            a = m.next(a);
        }
        assert_eq!(cnt, m.len());
        let mut a = m.last();
        let mut cnt = 0;
        while !a.is_null() {
            cnt += 1;
            a = m.prev(a);
        }
        assert_eq!(cnt, m.len());
        assert_eq!(*m.get(&111).unwrap(), -111);
        {
            let v = m.get_mut(&111).unwrap();
            *v *= -1;
        }
        assert_eq!(*m.get(&111).unwrap(), 111);
        assert!(m.get(&-100).is_none());
    }

    #[test]
    fn test_hash_map_remove() {
        struct Node<'a> {
            b: &'a RefCell<i32>,
        }
        impl<'a> Drop for Node<'a> {
            fn drop(&mut self) {
                *self.b.borrow_mut() += 1;
            }
        }
        let cnt = RefCell::new(0);
        let test_num = 199;
        let mut map = HashMap::new();
        for i in 0..test_num {
            map.insert(i, Node { b: &cnt });
        }
        assert_eq!(*cnt.borrow(), 0);
        for i in 0..test_num / 2 {
            map.remove(&i);
        }
        assert_eq!(*cnt.borrow(), test_num / 2);
        for i in test_num / 2..test_num {
            map.insert(i, Node { b: &cnt });
        }
        assert_eq!(*cnt.borrow(), test_num);
    }

    #[test]
    fn test_hash_map_clear() {
        struct Node<'a> {
            b: &'a RefCell<i32>,
        }
        impl<'a> Drop for Node<'a> {
            fn drop(&mut self) {
                *self.b.borrow_mut() += 1;
            }
        }
        let cnt = RefCell::new(0);
        let test_num = 199;
        let mut map = HashMap::new();
        for i in 0..test_num {
            map.insert(i, Node { b: &cnt });
        }
        assert_eq!(*cnt.borrow(), 0);
        for i in 0..test_num / 2 {
            map.remove(&i);
        }
        assert_eq!(*cnt.borrow(), test_num / 2);
        map.clear();
        assert_eq!(*cnt.borrow(), test_num);
    }

    #[test]
    fn test_hash_map_insert_duplicate() {
        struct Node<'a> {
            b: &'a RefCell<i32>,
        }
        impl<'a> Drop for Node<'a> {
            fn drop(&mut self) {
                *self.b.borrow_mut() += 1;
            }
        }
        let cnt = RefCell::new(0);
        let test_num = 100;
        let mut map = HashMap::new();
        for i in 0..test_num {
            map.insert(i, Node { b: &cnt });
        }
        assert_eq!(test_num as usize, map.len());
        assert_eq!(*cnt.borrow(), 0);
        for i in 0..test_num / 2 {
            map.insert(i, Node { b: &cnt });
        }
        assert_eq!(*cnt.borrow(), test_num / 2);
    }

    #[test]
    fn test_hash_map_keys() {
        let test_num = 100;
        let mut m = HashMap::new();
        for i in 0..test_num {
            m.insert(i, -i);
        }
        let mut sum = 0;
        for key in m.keys() {
            sum += *key;
        }
        assert_eq!(sum, test_num * (test_num - 1) / 2);
    }

    #[test]
    fn test_hash_map_values() {
        let test_num = 100;
        let mut m = HashMap::new();
        for i in 0..test_num {
            m.insert(i, -i);
        }
        let mut sum = 0;
        for value in m.values() {
            sum += *value;
        }
        assert_eq!(sum, -test_num * (test_num - 1) / 2);
    }

    #[test]
    fn test_hash_map_values_mut() {
        let test_num = 100;
        let mut m = HashMap::new();
        for i in 0..test_num {
            m.insert(i, -i);
        }
        let mut sum = 0;
        for value in m.values_mut() {
            *value *= 2;
        }
        for value in m.values() {
            sum += *value;
        }
        assert_eq!(sum, -test_num * (test_num - 1));
    }

    #[test]
    fn test_hash_map_iter() {
        let test_num = 100;
        let mut m = HashMap::new();
        for i in 0..test_num {
            m.insert(i, -i);
        }
        let mut sum = 0;
        let mut sum1 = 0;
        for value in m.iter() {
            sum += *value.0;
            sum1 += *value.1;
        }
        assert_eq!(sum, test_num * (test_num - 1) / 2);
        assert_eq!(sum1, -test_num * (test_num - 1) / 2);
    }

    #[test]
    fn test_hash_map_iter_mut() {
        let test_num = 100;
        let mut m = HashMap::new();
        for i in 0..test_num {
            m.insert(i, -i);
        }
        for (_, value) in m.iter_mut() {
            *value *= 2;
        }
        let mut sum = 0;
        let mut sum1 = 0;
        for value in m.iter() {
            sum += *value.0;
            sum1 += *value.1;
        }
        assert_eq!(sum, test_num * (test_num - 1) / 2);
        assert_eq!(sum1, -test_num * (test_num - 1));
    }

    #[test]
    fn test_hash_map_into_iter() {
        let test_num = 100;
        let mut a = HashMap::new();
        let mut b = HashMap::new();
        for i in 0..test_num as i32 {
            a.insert(i, -i);
        }
        for i in 0..test_num as i32 {
            b.insert(-i, i);
        }
        assert_eq!(a.len(), test_num);
        assert_eq!(b.len(), test_num);
        a.extend(b.into_iter());
        assert_eq!(a.len(), test_num * 2 - 1);
    }

    #[test]
    fn test_hash_map_shrink_to_fit() {
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
        let mut map = HashMap::new();
        for i in 0..test_num {
            map.insert(i, Node { b: &cnt });
        }
        for i in 10..test_num {
            map.remove(&i);
        }
        assert_eq!(*cnt.borrow(), test_num - 10);
        assert!(map.capacity() >= test_num as usize);
        map.shrink_to_fit();
        assert!(map.capacity() < test_num as usize);
        assert!(map.capacity() >= 10);
        assert_eq!(*cnt.borrow(), test_num - 10);
        drop(map);
        assert_eq!(*cnt.borrow(), test_num);
    }

    #[test]
    fn test_hash_map_clone_equal() {
        let mut a = HashMap::new();
        for i in 0..100 {
            a.insert(i, -i);
        }
        let mut b = a.clone();
        assert_eq!(b.len(), a.len());
        for (k, v) in a.iter() {
            assert_eq!(b[k], *v);
        }
        assert!(a == b);
        b.remove(&99);
        assert!(a != b);
    }

    #[test]
    fn test_from_iter() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];
        let map: HashMap<_, _> = xs.iter().cloned().collect();
        for &(k, v) in &xs {
            assert_eq!(map.get(&k), Some(&v));
        }
    }

    #[test]
    fn test_size_hint() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];
        let map: HashMap<_, _> = xs.iter().cloned().collect();
        let mut iter = map.iter();
        for _ in iter.by_ref().take(3) {}
        assert_eq!(iter.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_mut_size_hint() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];
        let mut map: HashMap<_, _> = xs.iter().cloned().collect();
        let mut iter = map.iter_mut();
        for _ in iter.by_ref().take(3) {}
        assert_eq!(iter.size_hint(), (3, Some(3)));
    }

    #[test]
    #[should_panic]
    fn test_index_nonexistent() {
        let mut map = HashMap::new();
        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(3, 4);
        map[&4];
    }
}