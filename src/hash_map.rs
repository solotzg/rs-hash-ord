use fastbin::{Fastbin, VoidPtr};
use hash_table::{HashNode, HashTable, HashUint};
use hash_table;
use hash_table::HashNodePtrOperation;
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::mem;
use std::ptr;
use avl_node::AVLNodePtrBase;
use hash_table::HashIndexPtrOperation;
use hash_table::HashNodeOperation;
use list::ListHeadPtrFn;
use std::cmp::Ordering;
use avl_node;

pub struct HashMap<K, V, S = RandomState> where K: Ord + Hash, S: BuildHasher {
    entry_fastbin: Fastbin,
    kv_fastbin: Fastbin,
    hash_table: Box<HashTable<K, V>>,
    hash_builder: S,
}

struct HashEntry<K, V> {
    node: HashNode<K>,
    value: *mut V,
}

#[inline]
fn key_deref_to_kv<K, V>(key: *const K) -> *mut (K, V) {
    (key as isize - unsafe { &(*(0 as *const (K, V))).0 as *const _ as isize }) as *mut (K, V)
}

trait HashEntryBase<K, V> {
    fn node_ptr(self) -> *mut HashNode<K>;
    fn value(self) -> *mut V;
    fn set_value(self, value: *mut V);
    fn key(self) -> *const K;
    fn set_key(self, key: *const K);
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
}

trait HashNodeDerefToHashEntry<K, V> {
    fn deref_to_hash_entry(self) -> *mut HashEntry<K, V>;
}

impl<K, V> HashNodeDerefToHashEntry<K, V> for *mut HashNode<K> {
    fn deref_to_hash_entry(self) -> *mut HashEntry<K, V> {
        container_of!(self, HashEntry<K, V>, node)
    }
}

impl<K, V, S> HashMap<K, V, S> where K: Ord + Hash, S: BuildHasher {
    #[inline]
    pub fn get_max_node_of_single_index(&self) -> i32 {
        self.hash_table.get_max_node_of_single_index()
    }

    #[inline]
    fn make_hash<X: ? Sized>(&self, x: &X) -> HashUint where X: Hash {
        hash_table::make_hash(&self.hash_builder, x)
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

    pub fn with_hasher(hash_builder: S) -> Self {
        let mut map = HashMap {
            entry_fastbin: Fastbin::new(mem::size_of::<HashEntry<K, V>>()),
            kv_fastbin: Fastbin::new(mem::size_of::<(K, V)>()),
            hash_table: Box::new(HashTable::new()),
            hash_builder,
        };
        map.hash_table.init();
        map
    }

    fn recurse_destroy(&mut self, node: avl_node::AVLNodePtr) {
        if node.left().not_null() {
            self.recurse_destroy(node.left());
        }
        if node.right().not_null() {
            self.recurse_destroy(node.right());
        }
        let hash_node = node.avl_hash_deref_mut::<K>();
        let entry: *mut HashEntry<K, V> = hash_node.deref_to_hash_entry();
        self.entry_fastbin.del(entry as VoidPtr);
        let kv_ptr = key_deref_to_kv::<K, V>(hash_node.key_ptr());
        unsafe { ptr::drop_in_place(kv_ptr); }
        self.kv_fastbin.del(kv_ptr as VoidPtr);
        self.hash_table.dec_count(1);
    }

    fn clear_hash_entry(&mut self) {
        loop {
            let node = self.hash_table.pop_first_index();
            if node.is_null() { break; }
            self.recurse_destroy(node);
        }
        debug_assert_eq!(self.hash_table.size(), 0);
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.hash_table.size()
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
    fn destroy(&mut self) {
        self.clear_hash_entry();
        self.entry_fastbin.destroy();
        self.kv_fastbin.destroy();
    }

    #[inline]
    fn find(&self, key: &K) -> *mut HashEntry<K, V> {
        let node = self.hash_table.hash_find(self.make_hash(key), key as *const K);
        if node.is_null() {
            ptr::null_mut()
        } else {
            node.deref_to_hash_entry()
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let entry = self.find(key);
        if entry.is_null() {
            return None;
        }
        unsafe { Some(&(*entry.value())) }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let entry = self.find(key);
        if entry.is_null() {
            return None;
        }
        unsafe { Some(&mut (*entry.value())) }
    }

    fn entry_alloc(&mut self, key: *const K, value: *mut V) -> *mut HashEntry<K, V> {
        let entry = self.entry_fastbin.alloc() as *mut HashEntry<K, V>;
        debug_assert!(!entry.is_null());
        entry.set_value(value);
        entry.node_ptr().set_key_ptr(key);
        entry
    }

    fn kv_alloc(&mut self, key: K, value: V) -> (*mut K, *mut V) {
        let kv = self.kv_fastbin.alloc() as *mut (K, V);
        unsafe {
            let key_ptr = &mut (*kv).0 as *mut K;
            let value_ptr = &mut (*kv).1 as *mut V;
            ptr::write(key_ptr, key);
            ptr::write(value_ptr, value);
            (key_ptr, value_ptr)
        }
    }

    unsafe fn update(&mut self, key: *const K, value: *mut V, update: bool, old_kv_ptr: &mut *mut (K, V)) -> *mut HashEntry<K, V> {
        let hash_val = self.make_hash(&(*key));
        let index = self.hash_table.get_hash_index(hash_val);
        let mut link = index.avl_root_node_ptr();
        let mut parent = ptr::null_mut();
        if index.avl_root_node().is_null() {
            let entry = self.entry_alloc(key, value);
            entry.node_ptr().avl_node_ptr().reset(ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), 1);
            entry.node_ptr().set_hash_val(hash_val);
            index.set_avl_root_node(entry.node_ptr().avl_node_ptr());
            self.hash_table.head_ptr().list_add_tail(index.node_ptr());
            self.hash_table.inc_count(1);
            return entry;
        }
        while !(*link).is_null() {
            parent = *link;
            let snode = parent.avl_hash_deref_mut::<K>();
            let snode_hash = snode.hash_val();
            if hash_val != snode_hash {
                link = if hash_val < snode_hash { parent.left_mut() } else { parent.right_mut() };
            } else {
                match (*key).cmp(&(*snode.key_ptr())) {
                    Ordering::Equal => {
                        let entry = snode.deref_to_hash_entry();
                        if update {
                            *old_kv_ptr = key_deref_to_kv::<K, V>(entry.key());
                            entry.set_key(key);
                            entry.set_value(value);
                        }
                        return entry;
                    }
                    Ordering::Less => {
                        link = parent.left_mut();
                    }
                    Ordering::Greater => {
                        link = parent.right_mut();
                    }
                }
            }
        }
        let entry = self.entry_alloc(key, value);
        debug_assert_ne!(parent, entry.node_ptr().avl_node_ptr());
        debug_assert!(!entry.is_null());
        entry.node_ptr().set_hash_val(hash_val);
        avl_node::link_node(entry.node_ptr().avl_node_ptr(), parent, link);
        avl_node::node_post_insert(entry.node_ptr().avl_node_ptr(), index.avl_root_ptr());
        self.hash_table.inc_count(1);
        entry
    }

    #[inline]
    fn rehash(&mut self, capacity: usize) {
        self.hash_table.rehash(capacity);
    }

    pub fn reserve(&mut self, capacity: usize) {
        self.rehash(capacity);
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        self.insert_or_replace(key, value);
    }

    pub fn contain(&self, key: &K) -> bool {
        !self.find(key).is_null()
    }

    pub fn insert_or_replace(&mut self, key: K, value: V) -> Option<(K, V)> {
        let (key_ptr, value_ptr) = self.kv_alloc(key, value);
        let mut old_kv_ptr = ptr::null_mut();
        unsafe { self.update(key_ptr, value_ptr, true, &mut old_kv_ptr) };
        let cap = self.hash_table.count();
        self.rehash(cap);
        if old_kv_ptr.is_null() {
            None
        } else {
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
}

impl<K: Hash + Ord, V> HashMap<K, V, RandomState> {
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

impl<K, V, S> Drop for HashMap<K, V, S> where K: Ord + Hash, S: BuildHasher {
    #[inline]
    fn drop(&mut self) {
        self.destroy();
    }
}

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
        assert_eq!(m.size(), 100);
        let mut a = m.first();
        let mut cnt = 0;
        while !a.is_null() {
            cnt += 1;
            a = m.next(a);
        }
        assert_eq!(cnt, m.size());
        let mut a = m.last();
        let mut cnt = 0;
        while !a.is_null() {
            cnt += 1;
            a = m.prev(a);
        }
        assert_eq!(cnt, m.size());
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
        for i in 0..test_num/2 {
            map.remove(&i);
        }
        assert_eq!(*cnt.borrow(), test_num/2);
        for i in test_num/2..test_num {
            map.insert_or_replace(i, Node { b: &cnt });
        }
        assert_eq!(*cnt.borrow(), test_num);
    }
}