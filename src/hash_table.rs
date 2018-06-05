use std::marker;
use std::mem;
use std::ptr;
use avl_node::{AVLNode, AVLNodePtr, AVLNodePtrBase, AVLRoot, AVLRootPtr};
use avl_node;
use list::{ListHead, ListHeadPtr, ListHeadPtrFn};
use std::hash::Hash;
use std::hash::BuildHasher;
use std::hash::Hasher;
use std::cmp;
use std::borrow::Borrow;
use std::cmp::Ordering;
use libc::{c_void, free, malloc};

pub type HashUint = usize;

const AVL_HASH_INIT_SIZE: usize = 8;

const DEFAULT_AVL_NODE: AVLNode = AVLNode {
    left: ptr::null_mut(),
    right: ptr::null_mut(),
    parent: ptr::null_mut(),
    height: 1i32,
};

pub struct HashNode<K> {
    pub hash_val: HashUint,
    pub key: *mut K,
    pub avl_node: AVLNode,
}

pub trait HashNodePtrOperation<K> {
    fn hash_val(self) -> HashUint;
    fn set_hash_val(self, hash_val: HashUint);
    fn avl_node_ptr(self) -> AVLNodePtr;
    fn key_ptr(self) -> *mut K;
    fn set_key_ptr(self, key: *mut K);
}

impl<K> HashNodePtrOperation<K> for *mut HashNode<K> {
    #[inline]
    fn hash_val(self) -> HashUint {
        unsafe { (*self).hash_val }
    }
    #[inline]
    fn set_hash_val(self, hash_val: HashUint) {
        unsafe {
            (*self).hash_val = hash_val;
        }
    }
    #[inline]
    fn avl_node_ptr(self) -> AVLNodePtr {
        unsafe { &mut (*self).avl_node as AVLNodePtr }
    }
    #[inline]
    fn key_ptr(self) -> *mut K {
        unsafe { (*self).key }
    }
    #[inline]
    fn set_key_ptr(self, key: *mut K) {
        unsafe {
            (*self).key = key;
        }
    }
}

trait ListHeadPtrOperateHashIndex {
    fn hash_index_deref_mut(self) -> *mut HashIndex;
}

impl ListHeadPtrOperateHashIndex for *mut ListHead {
    #[inline]
    fn hash_index_deref_mut(self) -> *mut HashIndex {
        container_of!(self, HashIndex, node)
    }
}

#[derive(Copy, Clone)]
pub struct HashIndex {
    avl_root: AVLRoot,
    node: ListHead,
}

impl Default for HashIndex {
    fn default() -> Self {
        HashIndex {
            avl_root: Default::default(),
            node: Default::default(),
        }
    }
}

pub trait HashIndexPtrOperation {
    fn avl_root_node(self) -> AVLNodePtr;
    fn node_ptr(self) -> ListHeadPtr;
    fn set_avl_root_node(self, root: AVLNodePtr);
    fn avl_root_ptr(self) -> AVLRootPtr;
    fn avl_root_node_ptr(self) -> *mut AVLNodePtr;
}

impl HashIndexPtrOperation for *mut HashIndex {
    #[inline]
    fn avl_root_node(self) -> AVLNodePtr {
        unsafe { (*self).avl_root.node }
    }

    #[inline]
    fn node_ptr(self) -> ListHeadPtr {
        unsafe { &mut (*self).node }
    }

    #[inline]
    fn set_avl_root_node(self, root: AVLNodePtr) {
        unsafe {
            (*self).avl_root.node = root;
        }
    }

    #[inline]
    fn avl_root_ptr(self) -> AVLRootPtr {
        unsafe { &mut (*self).avl_root as AVLRootPtr }
    }

    #[inline]
    fn avl_root_node_ptr(self) -> *mut AVLNodePtr {
        unsafe { &mut (*self).avl_root.node as *mut AVLNodePtr }
    }
}

pub struct HashTable<K, V> {
    count: usize,
    index_size: usize,
    index_mask: usize,
    head: ListHead,
    index: *mut HashIndex,
    init: [HashIndex; AVL_HASH_INIT_SIZE],
    _marker: marker::PhantomData<(K, V)>,
}

pub trait HashNodeOperation {
    fn avl_hash_deref_mut<K>(self) -> *mut HashNode<K>;
}

impl HashNodeOperation for *mut AVLNode {
    #[inline]
    fn avl_hash_deref_mut<K>(self) -> *mut HashNode<K> {
        container_of!(self, HashNode<K>, avl_node)
    }
}

#[inline]
pub fn make_hash<T: ?Sized, S>(hash_state: &S, t: &T) -> HashUint
where
    T: Hash,
    S: BuildHasher,
{
    let mut state = hash_state.build_hasher();
    t.hash(&mut state);
    state.finish() as HashUint
}

#[inline]
pub fn calc_limit(capacity: usize) -> usize {
    capacity.saturating_mul(6usize) / 4usize
}

#[inline]
pub unsafe fn find_duplicate_hash_node<K>(
    mut link: *mut AVLNodePtr,
    new_key: *mut K,
    hash_val: HashUint,
) -> (*mut HashNode<K>, AVLNodePtr, *mut AVLNodePtr)
where
    K: Ord,
{
    let mut parent = ptr::null_mut();
    while !(*link).is_null() {
        parent = *link;
        let snode = parent.avl_hash_deref_mut::<K>();
        let snode_hash = snode.hash_val();
        if hash_val != snode_hash {
            link = if hash_val < snode_hash {
                &mut (*parent).left
            } else {
                &mut (*parent).right
            };
        } else {
            match (*new_key).cmp(&(*snode.key_ptr())) {
                Ordering::Equal => {
                    return (snode, parent, link);
                }
                Ordering::Less => {
                    link = &mut (*parent).left;
                }
                Ordering::Greater => {
                    link = &mut (*parent).right;
                }
            }
        }
    }
    (ptr::null_mut(), parent, link)
}

impl<K, V> HashTable<K, V>
where
    K: Ord + Hash,
{
    #[inline]
    pub fn hash_find<Q: ?Sized>(&self, hash_val: HashUint, q: &Q) -> *mut HashNode<K>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let mut avl_node = self.get_hash_index(hash_val).avl_root_node();
        while avl_node.not_null() {
            let snode = avl_node.avl_hash_deref_mut::<K>();
            let shash_val = snode.hash_val();
            if hash_val == shash_val {
                match unsafe { q.cmp((*snode.key_ptr()).borrow()) } {
                    Ordering::Equal => {
                        return snode;
                    }
                    Ordering::Less => {
                        avl_node = avl_node.left();
                    }
                    Ordering::Greater => {
                        avl_node = avl_node.right();
                    }
                }
            } else {
                avl_node = if hash_val < shash_val {
                    avl_node.left()
                } else {
                    avl_node.right()
                }
            }
        }
        ptr::null_mut::<HashNode<K>>()
    }

    pub fn hash_swap(
        &mut self,
        new_index: *mut HashIndex,
        new_index_size: usize,
    ) -> *mut HashIndex {
        let old_index = self.index;
        let mut head = ListHead::default();
        let head_ptr = &mut head as ListHeadPtr;
        assert_ne!(new_index, old_index);
        self.index = new_index;
        self.index_size = new_index_size;
        self.index_mask = self.index_size - 1;
        self.count = 0;
        for i in 0..new_index_size as isize {
            unsafe {
                self.index.offset(i).set_avl_root_node(ptr::null_mut());
            }
            unsafe {
                self.index.offset(i).node_ptr().list_init();
            }
        }
        ListHeadPtr::list_replace(self.head_ptr(), head_ptr);
        self.head_ptr().list_init();
        while !head_ptr.list_is_empty() {
            let index = head.next.hash_index_deref_mut();
            self.recursive_hash_add(index.avl_root_node());
            index.node_ptr().list_del_init();
        }
        return if old_index == self.init.as_mut_ptr() {
            ptr::null_mut()
        } else {
            old_index
        };
    }

    fn recursive_hash_add(&mut self, node: AVLNodePtr) {
        if node.left().not_null() {
            self.recursive_hash_add(node.left());
        }
        if node.right().not_null() {
            self.recursive_hash_add(node.right());
        }
        let snode = node.avl_hash_deref_mut::<K>();
        unsafe {
            self.hash_add(snode);
        }
    }

    #[inline]
    pub fn rehash(&mut self, len: usize) {
        let old_index_size = self.index_size;
        let limit = calc_limit(len);
        if old_index_size >= limit {
            return;
        }
        let mut need = old_index_size;
        while need < limit {
            need = need.saturating_mul(2usize);
        }
        let buffer = unsafe {
            let (new_alloc_size, oflo) = need.overflowing_mul(mem::size_of::<HashIndex>());
            if oflo {
                panic!("capacity overflow");
            }
            let buffer = malloc(new_alloc_size) as *mut HashIndex;
            if buffer.is_null() {
                panic!("memory overflow");
            }
            buffer
        };
        let data_ptr = self.hash_swap(buffer, need);
        if !data_ptr.is_null() {
            unsafe {
                free(data_ptr as *mut c_void);
            }
        }
    }

    pub fn new_with_box() -> Box<Self> {
        let mut hash_table = Box::new(HashTable::new());
        hash_table.init();
        hash_table
    }
}

impl<K, V> HashTable<K, V> {
    #[inline]
    pub unsafe fn hash_add(&mut self, new_node: *mut HashNode<K>) -> *mut HashNode<K>
    where
        K: Ord,
    {
        let hash_val = new_node.hash_val();
        let index = self.get_hash_index(hash_val);
        let link = index.avl_root_node_ptr();
        let new_avl_node = new_node.avl_node_ptr();

        // for runtime, this part doesn't need to be combined into avl insertion
        // 77%
        if (*link).is_null() {
            (*link) = new_avl_node;
            ptr::write(new_avl_node, DEFAULT_AVL_NODE);
            self.head_ptr().list_add_tail(index.node_ptr());
            self.count += 1;
            return ptr::null_mut();
        }
        let (duplicate, parent, link) =
            find_duplicate_hash_node(link, new_node.key_ptr(), hash_val);
        if !duplicate.is_null() {
            avl_node::avl_node_replace(
                duplicate.avl_node_ptr(),
                new_avl_node,
                index.avl_root_ptr(),
            );
            return duplicate;
        }
        debug_assert_ne!(parent, new_avl_node);
        self.count += 1;
        avl_node::link_node(new_avl_node, parent, link);
        let avl_root_node = index.avl_root_node();
        if (*avl_root_node).height == 1 {
            // 19%
            (*avl_root_node).height = 2;
            (*new_avl_node).height = 1;
        } else {
            avl_node::node_post_insert(new_node.avl_node_ptr(), index.avl_root_ptr());
        }
        ptr::null_mut()
    }

    #[inline]
    pub fn index_size(&self) -> usize {
        self.index_size
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.index_size
    }

    pub fn get_max_node_of_single_index(&self) -> i32 {
        let mut head = self.head.next;
        let mut num = 0;
        while !self.head.is_eq_ptr(head) {
            num = cmp::max(
                num,
                head.hash_index_deref_mut().avl_root_node().get_node_num(),
            );
            head = head.next();
        }
        num
    }

    #[inline]
    pub fn pop_first_index(&mut self) -> AVLNodePtr {
        let head = self.head.next;
        if self.head.is_eq_ptr(head) {
            return ptr::null_mut();
        }
        let index = head.hash_index_deref_mut();
        let avl_node = index.avl_root_node();
        debug_assert!(avl_node.not_null());
        index.set_avl_root_node(ptr::null_mut());
        head.list_del_init();
        avl_node
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.count
    }

    #[inline]
    pub fn inc_count(&mut self, cnt: usize) {
        self.count += cnt;
    }

    #[inline]
    pub fn dec_count(&mut self, cnt: usize) {
        self.count -= cnt;
    }

    fn new() -> Self {
        HashTable {
            count: 0,
            index_size: 0,
            index_mask: 0,
            head: Default::default(),
            index: ptr::null_mut(),
            init: [HashIndex::default(); AVL_HASH_INIT_SIZE],
            _marker: marker::PhantomData,
        }
    }

    #[inline]
    pub fn head_ptr(&mut self) -> ListHeadPtr {
        &mut self.head as ListHeadPtr
    }

    #[inline]
    pub fn init(&mut self) {
        self.count = 0;
        self.index_size = AVL_HASH_INIT_SIZE;
        self.index_mask = self.index_size - 1;
        self.head_ptr().list_init();
        self.index = self.init.as_mut_ptr();
        for i in 0..AVL_HASH_INIT_SIZE {
            unsafe {
                (*self.index.offset(i as isize)).avl_root.node = ptr::null_mut();
                (&mut (*self.index.offset(i as isize)).node as ListHeadPtr).list_init();
            }
        }
    }

    #[inline]
    pub fn node_first(&self) -> *mut HashNode<K> {
        let head: ListHeadPtr = self.head.next as ListHeadPtr;
        if !self.head.is_eq_ptr(head) {
            let index: *mut HashIndex = head.hash_index_deref_mut();
            let avl_node = index.avl_root_node().first_node();
            if avl_node.is_null() {
                return ptr::null_mut();
            }
            return avl_node.avl_hash_deref_mut::<K>();
        }
        return ptr::null_mut();
    }

    #[inline]
    pub fn node_last(&self) -> *mut HashNode<K> {
        let head: ListHeadPtr = self.head.prev;
        if !self.head.is_eq_ptr(head) {
            let index: *mut HashIndex = head.hash_index_deref_mut();
            let avl_node = index.avl_root_node().last_node();
            if avl_node.is_null() {
                return ptr::null_mut();
            }
            return avl_node.avl_hash_deref_mut::<K>();
        }
        return ptr::null_mut();
    }

    #[inline]
    pub fn get_hash_index(&self, hash_val: HashUint) -> *mut HashIndex {
        unsafe { self.index.offset((hash_val & self.index_mask) as isize) }
    }

    #[inline]
    pub fn node_next(&self, node: *mut HashNode<K>) -> *mut HashNode<K> {
        if node.is_null() {
            return ptr::null_mut::<HashNode<K>>();
        }
        let mut avl_node = node.avl_node_ptr().next();
        if avl_node.not_null() {
            return avl_node.avl_hash_deref_mut::<K>();
        }
        let mut index = unsafe { self.get_hash_index((*node).hash_val) };
        let list_node = index.node_ptr().next();
        if self.head.is_eq_ptr(list_node) {
            return ptr::null_mut::<HashNode<K>>();
        }
        index = list_node.hash_index_deref_mut();
        avl_node = index.avl_root_node().first_node();
        if avl_node.is_null() {
            return ptr::null_mut::<HashNode<K>>();
        }
        return avl_node.avl_hash_deref_mut::<K>();
    }

    #[inline]
    pub fn node_prev(&self, node: *mut HashNode<K>) -> *mut HashNode<K> {
        if node.is_null() {
            return ptr::null_mut::<HashNode<K>>();
        }
        let mut avl_node = node.avl_node_ptr().prev();
        if avl_node.not_null() {
            return avl_node.avl_hash_deref_mut::<K>();
        }
        let mut index = unsafe { self.get_hash_index((*node).hash_val) };
        let list_node = index.node_ptr().prev();
        if self.head.is_eq_ptr(list_node) {
            return ptr::null_mut::<HashNode<K>>();
        }
        index = list_node.hash_index_deref_mut();
        avl_node = index.avl_root_node().last_node();
        if avl_node.is_null() {
            return ptr::null_mut::<HashNode<K>>();
        }
        return avl_node.avl_hash_deref_mut::<K>();
    }

    #[inline]
    pub fn hash_erase(&mut self, node: *mut HashNode<K>) {
        debug_assert!(!node.avl_node_ptr().empty());
        let index = self.get_hash_index(node.hash_val());
        if index.avl_root_node().height() == 1 {
            index.set_avl_root_node(ptr::null_mut());
            index.node_ptr().list_del_init();
        } else {
            unsafe {
                avl_node::erase_node(node.avl_node_ptr(), index.avl_root_ptr());
            }
        }
        node.avl_node_ptr().init();
        self.count -= 1;
    }
}

impl<K, V> Drop for HashTable<K, V> {
    fn drop(&mut self) {
        if self.index != self.init.as_mut_ptr() {
            unsafe {
                free(self.index as *mut c_void);
            }
        }
    }
}
