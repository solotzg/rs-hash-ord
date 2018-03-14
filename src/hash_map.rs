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

trait HashEntryBase<K, V> {
    fn node_ptr(self) -> *mut HashNode<K>;
    fn value(self) -> *mut V;
    fn set_value(self, value: *mut V);
    fn key(self) -> *const K;
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

    fn clear(&mut self) {
        loop {
            let entry = self.first();
            if entry.is_null() { break; }
            self.erase(entry, false);
        }
        debug_assert_eq!(self.hash_table.size(), 0);
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.hash_table.size()
    }

    fn erase(&mut self, entry: *mut HashEntry<K, V>, need_ret: bool) -> Option<(K, V)> {
        debug_assert!(!entry.is_null());
        debug_assert!(!entry.node_ptr().avl_node_ptr().empty());
        self.hash_table.hash_erase(entry.node_ptr());
        entry.node_ptr().avl_node_ptr().init();
        let kv = entry.node_ptr().key_ptr() as *mut (K, V);
        entry.node_ptr().set_key_ptr(ptr::null());
        entry.set_value(ptr::null_mut());
        self.entry_fastbin.del(entry as VoidPtr);
        if !need_ret {
            self.kv_fastbin.del(kv as VoidPtr);
            None
        } else {
            unsafe {
                let key = &mut (*kv).0 as *mut K;
                let value = &mut (*kv).1 as *mut V;
                let res = Some((ptr::read(key), ptr::read(value)));
                self.kv_fastbin.del(kv as VoidPtr);
                res
            }
        }
    }

    #[inline]
    fn destroy(&mut self) {
        self.clear();
        let ptr = self.hash_table.hash_swap(ptr::null_mut(), 0);
        if !ptr.is_null() {
            unsafe { Box::from_raw(ptr); }
        }
        self.entry_fastbin.destroy();
        self.kv_fastbin.destroy();
    }

    fn find(&self, key: &K) -> *mut HashEntry<K, V> {
        let mut dummy = HashNode {
            hash_val: self.make_hash(key),
            key: key as *const K,
            avl_node: Default::default(),
        };
        let node = self.hash_table.hash_find(&mut dummy as *mut HashNode<K>);
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
            let key_ptr = unsafe {&mut (*kv).0 as *mut K};
            let value_ptr = unsafe {&mut (*kv).1 as *mut V};
            ptr::write(key_ptr, key);
            ptr::write(value_ptr, value);
            (key_ptr, value_ptr)
        }
    }

    unsafe fn update(&mut self, key: *const K, value: *mut V, update: bool) -> *mut HashEntry<K, V> {
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
            self.hash_table.inc_count();
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
                            Box::from_raw(entry.value());
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
        self.hash_table.inc_count();
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
        let (key_ptr, value_ptr) = self.kv_alloc(key, value);
        unsafe { self.update(key_ptr, value_ptr, false) };
        let cap = self.hash_table.count();
        self.rehash(cap);
    }

    pub fn pop(&mut self, key: &K) -> Option<(K, V)> {
        let entry = self.find(key);
        if entry.is_null() {
            return None;
        }
        self.erase(entry, true)
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

#[test]
fn just_for_compile() {}

mod test {
    use hash_map::{HashMap, HashEntryBase};
    use std::collections::hash_map::RandomState;

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
        assert_eq!(*m.get(&111).unwrap(), -111);
        {
            let v = m.get_mut(&111).unwrap();
            *v *= -1;
        }
        assert_eq!(*m.get(&111).unwrap(), 111);
    }

    #[test]
    fn test_hash_map_pop() {
        let mut m = HashMap::new();
        for i in 100..200 {
            m.insert(i, Some('1'));
        }
        for i in 100..200 {
            let kv = m.pop(&i);
            assert!(kv.is_some());
            let tp = kv.unwrap();
            assert_eq!(tp.0, i);
            assert_eq!(tp.1, Some('1'));
        }
    }
}