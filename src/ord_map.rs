use std::cmp::Ordering;
use std::{marker, mem, ptr};
use std::ops::Index;
use std::iter::FromIterator;
use avl_node::{AVLNode, AVLNodePtr, AVLNodePtrBase, AVLRoot, AVLRootPtr};
use avl_node;
use fastbin::{Fastbin, VoidPtr};
use std::borrow::Borrow;

struct AVLEntry<K, V> {
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
    fn key_mut<'a, K, V>(self) -> &'a mut K;
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

    fn key_mut<'a, K, V>(self) -> &'a mut K {
        unsafe { &mut (*self.avl_node_deref_to_entry::<K, V>()).key }
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
        unsafe {
            (*self.avl_node_deref_to_entry::<K, V>()).value = value;
        }
    }

    #[inline]
    fn avl_node_deref_to_entry<K, V>(self) -> *mut AVLEntry<K, V> {
        container_of!(self, AVLEntry<K, V>, node)
    }
}

/// An cursor of a `OrdMap`.
///
/// This struct is constructed from the [`find_cursors`] method on [`OrdMap`].
///
/// [`OrdMap`]: struct.OrdMap.html
/// [`find_cursors`]: struct.OrdMap.html#method.find_cursors
pub struct Cursors<'a, K, V>
where
    K: Ord + 'a,
    V: 'a,
{
    tree_mut: &'a mut OrdMap<K, V>,
    pos: AVLNodePtr,
}

enum CursorsOperation {
    NEXT,
    PREV,
}

impl<'a, K, V> Cursors<'a, K, V>
where
    K: Ord,
{
    /// Move cursor to next pos.
    pub fn next(&mut self) {
        self.pos = self.pos.next();
    }

    /// Move cursor to next pos.
    pub fn prev(&mut self) {
        self.pos = self.pos.prev();
    }

    /// Returns the (&Key, &Value) pair of current pos.
    ///
    /// # Examples
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    /// use hash_ord::ord_map::Cursors;
    ///
    /// let mut map = OrdMap::new();
    /// map.insert(1, 1);
    /// map.insert(2, 2);
    /// map.insert(3, 3);
    /// let mut cursors = map.find_cursors(&2);
    /// assert_eq!(*cursors.get().unwrap().0, 2);
    /// ```
    pub fn get(&self) -> Option<(&K, &V)> {
        if self.pos.not_null() {
            Some((self.pos.key_ref::<K, V>(), self.pos.value_ref::<K, V>()))
        } else {
            None
        }
    }

    /// Returns the (&Key, &mut Value) pair of current pos.
    ///
    /// # Examples
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    /// use hash_ord::ord_map::Cursors;
    ///
    /// let mut map = OrdMap::new();
    /// map.insert(1, 1);
    /// map.insert(2, 2);
    /// map.insert(3, 3);
    /// let mut cursors = map.find_cursors(&2);
    /// *cursors.get_mut().unwrap().1 = -2;
    /// assert_eq!(*cursors.get().unwrap().1, -2);
    /// ```
    pub fn get_mut(&mut self) -> Option<(&K, &mut V)> {
        if self.pos.not_null() {
            Some((self.pos.key_ref::<K, V>(), self.pos.value_mut::<K, V>()))
        } else {
            None
        }
    }

    fn erase(&mut self, op: CursorsOperation) -> Option<(K, V)> {
        if self.pos.is_null() {
            return None;
        }
        let node = self.pos;
        match op {
            CursorsOperation::NEXT => self.next(),
            CursorsOperation::PREV => self.prev(),
        }
        unsafe {
            return self.tree_mut.remove_node(node);
        }
    }

    /// Erase current pos, and move to next.
    ///
    /// # Examples
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    /// use hash_ord::ord_map::Cursors;
    ///
    /// let mut map = OrdMap::new();
    /// map.insert(1, 1);
    /// map.insert(2, 2);
    /// map.insert(3, 3);
    /// let mut cursors = map.find_cursors(&2);
    /// let x = cursors.erase_then_next();
    /// assert_eq!(x.unwrap().0, 2);
    /// assert_eq!(*cursors.get().unwrap().0, 3);
    /// ```
    pub fn erase_then_next(&mut self) -> Option<(K, V)> {
        self.erase(CursorsOperation::NEXT)
    }

    /// Erase current pos, and move to prev.
    ///
    /// # Examples
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    /// use hash_ord::ord_map::Cursors;
    ///
    /// let mut map = OrdMap::new();
    /// map.insert(1, 1);
    /// map.insert(2, 2);
    /// map.insert(3, 3);
    /// let mut cursors = map.find_cursors(&2);
    /// let x = cursors.erase_then_prev();
    /// assert_eq!(x.unwrap().0, 2);
    /// assert_eq!(*cursors.get().unwrap().0, 1);
    /// ```
    pub fn erase_then_prev(&mut self) -> Option<(K, V)> {
        self.erase(CursorsOperation::PREV)
    }
}

/// Optimized AVL.
///
/// To improve performance, raw pointer is used frequently. Because Rust uses a similar memory model
/// to C/C++, two classic macros `offset_of` and `container_of` are used to dereference member
/// variables into main struct. `Fastbin` is implemented to reduce the cost of memory allocation.
///
///
/// # Examples
///
/// ```
/// use hash_ord::ord_map::OrdMap;
///
/// // type inference lets us omit an explicit type signature (which
/// // would be `OrdMap<&str, &str>` in this example).
/// let mut book_reviews = OrdMap::new();
///
/// // review some books.
/// book_reviews.insert("Adventures of Huckleberry Finn",    "My favorite book.");
/// book_reviews.insert("Grimms' Fairy Tales",               "Masterpiece.");
/// book_reviews.insert("Pride and Prejudice",               "Very enjoyable.");
/// book_reviews.insert("The Adventures of Sherlock Holmes", "Eye lyked it alot.");
///
/// // check for a specific one.
/// if !book_reviews.contains_key("Les Misérables") {
///     println!("We've got {} reviews, but Les Misérables ain't one.",
///              book_reviews.len());
/// }
///
/// // oops, this review has a lot of spelling mistakes, let's delete it.
/// book_reviews.remove("The Adventures of Sherlock Holmes");
///
/// // look up the values associated with some keys.
/// let to_find = ["Pride and Prejudice", "Alice's Adventure in Wonderland"];
/// for book in &to_find {
///     match book_reviews.get(book) {
///         Some(review) => println!("{}: {}", book, review),
///         None => println!("{} is unreviewed.", book)
///     }
/// }
///
/// // iterate over everything.
/// for (book, review) in &book_reviews {
///     println!("{}: \"{}\"", book, review);
/// }
/// ```
///
/// The easiest way to use `OrdMap` with a custom type as key is to derive `Ord``.
///
/// ```
/// use hash_ord::ord_map::OrdMap;
/// use std::cmp::Ordering;
///
/// #[derive(Eq, Debug)]
/// struct Viking {
///    name: String,
///    country: String,
/// }
///
/// impl Ord for Viking {
///     fn cmp(&self, other: &Self) -> Ordering {
///         let tmp = self.name.cmp(&other.name);
///         return if tmp != Ordering::Equal { tmp } else {
///             self.country.cmp(&other.country)
///         };
///     }
/// }
///
/// impl PartialOrd for Viking {
///     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
///         Some(self.cmp(other))
///     }
/// }
///
/// impl PartialEq for Viking {
///     fn eq(&self, other: &Self) -> bool {
///         self.name == other.name && self.country == other.country
///     }
/// }
///
/// impl Viking {
///     /// / Create a new Viking.
///     fn new(name: &str, country: &str) -> Viking {
///         Viking { name: name.to_string(), country: country.to_string() }
///     }
/// }
///
/// //  Use a OrdMap to store the vikings' health points.
/// let mut vikings = OrdMap::new();
///
/// vikings.insert(Viking::new("Einar", "Norway"), 25);
/// vikings.insert(Viking::new("Olaf", "Denmark"), 24);
/// vikings.insert(Viking::new("Harald", "Iceland"), 12);
///
/// // Use derived implementation to print the status of the vikings.
/// for (viking, health) in &vikings {
///     println!("{:?} has {} hp", viking, health);
/// }
/// ```
///
/// A `OrdMap` with fixed list of elements can be initialized from an array:
///
/// ```
/// use hash_ord::ord_map::OrdMap;
///
/// let timber_resources: OrdMap<&str, i32> =
///     [("Norway", 100),
///      ("Denmark", 50),
///      ("Iceland", 10)]
///      .iter().cloned().collect();
///
pub struct OrdMap<K, V> {
    root: AVLRoot,
    count: usize,
    entry_fastbin: Fastbin,
    _marker: marker::PhantomData<(K, V)>,
}

/// A view into an occupied entry in a `OrdMap`.
/// It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct OccupiedEntry<'a, K, V>
where
    K: 'a,
    V: 'a,
{
    key: Option<K>,
    node: AVLNodePtr,
    ord_map_mut: &'a mut OrdMap<K, V>,
}

/// A view into a vacant entry in a `OrdMap`.
/// It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct VacantEntry<'a, K, V>
where
    K: 'a,
    V: 'a,
{
    key: K,
    parent: AVLNodePtr,
    link: *mut AVLNodePtr,
    ord_map_mut: &'a mut OrdMap<K, V>,
}

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`OrdMap`].
///
/// [`OrdMap`]: struct.OrdMap.html
/// [`entry`]: struct.OrdMap.html#method.entry
pub enum Entry<'a, K, V>
where
    K: 'a,
    V: 'a,
{
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V>),

    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Ord,
{
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }

    pub fn and_modify<F>(self, mut f: F) -> Self
    where
        F: FnMut(&mut V),
    {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }
}

impl<'a, K, V> OccupiedEntry<'a, K, V>
where
    K: Ord,
{
    pub fn remove_entry(self) -> (K, V) {
        unsafe { self.ord_map_mut.remove_node(self.node).unwrap() }
    }

    pub fn remove(self) -> V {
        self.remove_entry().1
    }

    pub fn key(&self) -> &K {
        &*self.node.key_ref::<K, V>()
    }

    fn take_key(&mut self) -> Option<K> {
        self.key.take()
    }

    pub fn replace_key(self) -> K {
        let old_key = self.node.key_mut::<K, V>();
        mem::replace(old_key, self.key.unwrap())
    }

    pub fn get(&self) -> &V {
        self.node.value_ref::<K, V>()
    }

    pub fn get_mut(&mut self) -> &mut V {
        self.node.value_mut::<K, V>()
    }

    pub fn into_mut(self) -> &'a mut V {
        self.node.value_mut::<K, V>()
    }

    pub fn insert(&mut self, mut value: V) -> V {
        let old_value = self.get_mut();
        mem::swap(&mut value, old_value);
        value
    }

    pub fn replace_entry(self, value: V) -> (K, V) {
        let old_key = self.node.key_mut::<K, V>();
        let old_key = mem::replace(old_key, self.key.unwrap());
        let old_value = self.node.value_mut::<K, V>();
        let old_value = mem::replace(old_value, value);
        (old_key, old_value)
    }
}

impl<'a, K, V> VacantEntry<'a, K, V>
where
    K: Ord,
{
    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn into_key(self) -> K {
        self.key
    }

    unsafe fn _internal_insert(self, value: V) -> &'a mut V {
        let key = self.key;
        let new_entry = self.ord_map_mut.entry_alloc(key, value);
        let new_node = new_entry.node_ptr();
        avl_node::link_node(new_node, self.parent, self.link);
        avl_node::node_post_insert(new_node, self.ord_map_mut.get_root_ptr());
        self.ord_map_mut.count += 1;
        &mut *new_entry.value()
    }

    pub fn insert(self, value: V) -> &'a mut V {
        unsafe { self._internal_insert(value) }
    }
}

impl<K, V> OrdMap<K, V> {
    fn recursive_drop_node(&mut self, node: AVLNodePtr) {
        if node.left().not_null() {
            self.recursive_drop_node(node.left());
        }
        if node.right().not_null() {
            self.recursive_drop_node(node.right());
        }
        let entry = node.avl_node_deref_to_entry::<K, V>();
        if mem::needs_drop::<AVLEntry<K, V>>() {
            unsafe {
                ptr::drop_in_place(entry);
            }
        }
        self.entry_fastbin.del(entry as VoidPtr);
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory
    /// for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    ///
    /// let mut a = OrdMap::new();
    /// a.insert(1, "a");
    /// a.clear();
    /// assert!(a.is_empty());
    /// ```
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
}

impl<K, V> OrdMap<K, V>
where
    K: Ord,
{
    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    ///
    /// let mut letters = OrdMap::new();
    ///
    /// for ch in "a short treatise on fungi".chars() {
    ///     let counter = letters.entry(ch).or_insert(0);
    ///     *counter += 1;
    /// }
    ///
    /// assert_eq!(letters[&'s'], 2);
    /// assert_eq!(letters[&'t'], 3);
    /// assert_eq!(letters[&'u'], 1);
    /// assert_eq!(letters.get(&'y'), None);
    /// ```
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        let (duplicate, parent, link) = unsafe { self.find_duplicate(&key) };
        if duplicate.is_null() {
            return Entry::Vacant(VacantEntry {
                key,
                parent,
                link,
                ord_map_mut: self,
            });
        } else {
            return Entry::Occupied(OccupiedEntry {
                key: Some(key),
                node: duplicate,
                ord_map_mut: self,
            });
        };
    }

    /// Returns the cursors of a found pos.
    #[inline]
    pub fn find_cursors<Q>(&mut self, q: &Q) -> Cursors<K, V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let node = self.find_node(q);
        Cursors {
            tree_mut: self,
            pos: node,
        }
    }

    /// Returns the max height of the tree.
    #[inline]
    pub fn max_height(&self) -> i32 {
        self.root.node.height()
    }

    /// Returns true if the map contains no element.
    ///
    /// # Examples
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    ///
    /// let mut map = OrdMap::new();
    /// assert!(map.is_empty());
    /// map.insert(1, 1);
    /// assert!(!map.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    ///
    /// let mut a = OrdMap::new();
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
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

    /// Creates an empty `OrdMap`.
    ///
    /// The hash map is initially created with a capacity of 0, so it will not allocate until it
    /// is first inserted into.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    /// let mut map: OrdMap<&str, isize> = OrdMap::new();
    /// ```
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

    fn deep_clone_node(&mut self, parent: AVLNodePtr, other_node: AVLNodePtr) -> AVLNodePtr
    where
        K: Clone,
        V: Clone,
    {
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

    fn clone_from(t: &OrdMap<K, V>) -> Self
    where
        K: Clone,
        V: Clone,
    {
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
    unsafe fn find_duplicate(&mut self, key: &K) -> (AVLNodePtr, AVLNodePtr, *mut AVLNodePtr) {
        let mut cmp_node_ref = &mut self.root.node as *mut AVLNodePtr;
        let mut parent = ptr::null_mut();
        while (*cmp_node_ref).not_null() {
            parent = *cmp_node_ref;
            match key.cmp(parent.key_ref::<K, V>()) {
                Ordering::Less => {
                    cmp_node_ref = parent.left_mut();
                }
                Ordering::Equal => {
                    return (parent, parent, cmp_node_ref);
                }
                Ordering::Greater => {
                    cmp_node_ref = parent.right_mut();
                }
            }
        }
        (ptr::null_mut(), parent, cmp_node_ref)
    }

    #[inline]
    fn find_node<Q: ?Sized>(&self, q: &Q) -> AVLNodePtr
    where
        K: Borrow<Q>,
        Q: Ord,
    {
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

    /// Return true if two tree are isomorphic.
    #[inline]
    pub fn isomorphic(&self, other: &OrdMap<K, V>) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.root.node.isomorphic(other.root.node)
    }

    /// Return true if tree is valid.
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
                None => {
                    break;
                }
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
                None => {
                    break;
                }
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

    /// Removes a key from the map, returning the stored key and value if the
    /// key was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but `Ord` on the borrowed
    /// form *must* match those for the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    ///
    /// let mut map = OrdMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(&1), Some((1, "a")));
    /// assert_eq!(map.remove(&1), None);
    /// ```
    #[inline]
    pub fn remove<Q: ?Sized>(&mut self, q: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let node = self.find_node(q);
        unsafe { self.remove_node(node) }
    }

    /// Returns true if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but `Ord` on the borrowed
    /// form *must* match those for the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    ///
    /// let mut map = OrdMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
    #[inline]
    pub fn contains_key<Q: ?Sized>(&self, q: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.find_node(q).not_null()
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but `Ord` on the borrowed
    /// form *must* match those for the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    ///
    /// let mut map = OrdMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    #[inline]
    pub fn get<Q: ?Sized>(&self, q: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let node = self.find_node(q);
        if node.is_null() {
            None
        } else {
            Some(node.value_ref::<K, V>())
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// `Ord` on the borrowed form *must* match those for the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    ///
    /// let mut map = OrdMap::new();
    /// map.insert(1, "a");
    /// if let Some(x) = map.get_mut(&1) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map[&1], "b");
    /// ```
    pub fn get_mut<Q: ?Sized>(&mut self, q: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let node = self.find_node(q);
        if node.is_null() {
            None
        } else {
            Some(node.value_mut::<K, V>())
        }
    }

    #[inline]
    fn link_post_insert(
        &mut self,
        new_node: AVLNodePtr,
        parent: AVLNodePtr,
        cmp_node_ref: *mut AVLNodePtr,
    ) {
        unsafe {
            avl_node::link_node(new_node, parent, cmp_node_ref);
        }
        unsafe {
            avl_node::node_post_insert(new_node, self.get_root_ptr());
        }
        self.count += 1;
    }

    #[inline]
    fn get_root_ptr(&mut self) -> AVLRootPtr {
        &mut self.root as AVLRootPtr
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, update map with new (key, value) and
    /// return the old one.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    ///
    /// let mut map = OrdMap::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some((37, "b")));
    /// assert_eq!(map[&37], "c");
    /// ```
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        let (duplicate, parent, cmp_node_ref) = unsafe { self.find_duplicate(&key) };
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

    /// An iterator visiting all keys in incremental order.
    /// The iterator element type is `&'a K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    ///
    /// let mut map = OrdMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for key in map.keys() {
    ///     println!("{}", key);
    /// }
    /// ```
    #[inline]
    pub fn keys(&self) -> Keys<K, V> {
        Keys {
            inner: self.iter(),
            _marker: marker::PhantomData,
        }
    }

    /// An iterator visiting all values in incremental order.
    /// The iterator element type is `&'a V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    ///
    /// let mut map = OrdMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for val in map.values() {
    ///     println!("{}", val);
    /// }
    /// ```
    #[inline]
    pub fn values(&self) -> Values<K, V> {
        Values {
            inner: self.iter(),
            _marker: marker::PhantomData,
        }
    }

    /// An iterator visiting all values mutably in incremental order.
    /// The iterator element type is `&'a mut V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    ///
    /// let mut map = OrdMap::new();
    ///
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for val in map.values_mut() {
    ///     *val = *val + 10;
    /// }
    ///
    /// for val in map.values() {
    ///     println!("{}", val);
    /// }
    /// ```
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        ValuesMut {
            inner: self.iter_mut(),
            _marker: marker::PhantomData,
        }
    }

    /// An iterator visiting all key-value pairs in incremental order.
    /// The iterator element type is `(&'a K, &'a V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    ///
    /// let mut map = OrdMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for (key, val) in map.iter() {
    ///     println!("key: {} val: {}", key, val);
    /// }
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            head: self.first_node(),
            tail: self.last_node(),
            len: self.len(),
            _marker: marker::PhantomData,
        }
    }

    /// An iterator visiting all key-value pairs in incremental order,
    /// with mutable references to the values.
    /// The iterator element type is `(&'a K, &'a mut V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::ord_map::OrdMap;
    ///
    /// let mut map = OrdMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// // Update all values
    /// for (_, val) in map.iter_mut() {
    ///     *val *= 2;
    /// }
    ///
    /// for (key, val) in &map {
    ///     println!("key: {} val: {}", key, val);
    /// }
    /// ```
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

impl<K, V> Drop for OrdMap<K, V> {
    fn drop(&mut self) {
        self.destroy();
    }
}

impl<K, V> Clone for OrdMap<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        OrdMap::clone_from(self)
    }
}

impl<K, V> PartialEq for OrdMap<K, V>
where
    K: Eq + Ord,
    V: PartialEq,
{
    fn eq(&self, other: &OrdMap<K, V>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<K, V> Eq for OrdMap<K, V>
where
    K: Eq + Ord,
    V: Eq,
{
}

impl<'a, K, V> Index<&'a K> for OrdMap<K, V>
where
    K: Ord,
{
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the `OrdMap`.
    #[inline]
    fn index(&self, key: &K) -> &V {
        self.get(key).expect("no entry found for key")
    }
}

impl<K, V> FromIterator<(K, V)> for OrdMap<K, V>
where
    K: Ord,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> OrdMap<K, V> {
        let mut tree = OrdMap::new();
        tree.extend(iter);
        tree
    }
}

impl<K, V> Extend<(K, V)> for OrdMap<K, V>
where
    K: Ord,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

/// An iterator over the keys of a `OrdMap`.
///
/// This `struct` is created by the [`keys`] method on [`OrdMap`]. See its
/// documentation for more.
///
/// [`keys`]: struct.OrdMap.html#method.keys
/// [`OrdMap`]: struct.OrdMap.html
pub struct Keys<'a, K: Ord + 'a, V: 'a> {
    inner: Iter<'a, K, V>,
    _marker: marker::PhantomData<&'a (K, V)>,
}

impl<'a, K: Ord, V> Clone for Keys<'a, K, V> {
    fn clone(&self) -> Keys<'a, K, V> {
        Keys {
            inner: self.inner.clone(),
            _marker: marker::PhantomData,
        }
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

/// An iterator over the values of a `OrdMap`.
///
/// This `struct` is created by the [`values`] method on [`OrdMap`]. See its
/// documentation for more.
///
/// [`values`]: struct.OrdMap.html#method.values
/// [`OrdMap`]: struct.OrdMap.html
pub struct Values<'a, K: 'a + Ord, V: 'a> {
    inner: Iter<'a, K, V>,
    _marker: marker::PhantomData<&'a (K, V)>,
}

impl<'a, K: Ord, V> Clone for Values<'a, K, V> {
    fn clone(&self) -> Values<'a, K, V> {
        Values {
            inner: self.inner.clone(),
            _marker: marker::PhantomData,
        }
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

/// A mutable iterator over the values of a `OrdMap`.
///
/// This `struct` is created by the [`values_mut`] method on [`OrdMap`]. See its
/// documentation for more.
///
/// [`values_mut`]: struct.OrdMap.html#method.values_mut
/// [`OrdMap`]: struct.OrdMap.html
pub struct ValuesMut<'a, K: 'a + Ord, V: 'a> {
    inner: IterMut<'a, K, V>,
    _marker: marker::PhantomData<(K, V)>,
}

impl<'a, K: Ord, V> Clone for ValuesMut<'a, K, V> {
    fn clone(&self) -> ValuesMut<'a, K, V> {
        ValuesMut {
            inner: self.inner.clone(),
            _marker: marker::PhantomData,
        }
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

/// An owning iterator over the entries of a `OrdMap`.
///
/// This `struct` is created by the [`into_iter`] method on [`OrdMap`][`OrdMap`]
/// (provided by the `IntoIterator` trait). See its documentation for more.
///
/// [`into_iter`]: struct.OrdMap.html#method.into_iter
/// [`OrdMap`]: struct.OrdMap.html
pub struct IntoIter<K, V>
where
    K: Ord,
{
    head: AVLNodePtr,
    tail: AVLNodePtr,
    len: usize,
    entry_fastbin: Fastbin,
    _marker: marker::PhantomData<(K, V)>,
}

impl<K, V> IntoIter<K, V>
where
    K: Ord,
{
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

impl<K, V> Iterator for IntoIter<K, V>
where
    K: Ord,
{
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

impl<K, V> IntoIterator for OrdMap<K, V>
where
    K: Ord,
{
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

/// An iterator over the (key, value) of a `OrdMap`.
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

impl<'a, K, V> IntoIterator for &'a OrdMap<K, V>
where
    K: Ord,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
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

/// An iterator over the (key, mut value) of a `OrdMap`.
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
    use ord_map::Entry::*;
    use std::rc::Rc;

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
            assert_eq!(
                *t.root.node.key_ref::<MyData, Option<i32>>(),
                MyData { a: 1 }
            );
            assert_eq!(t.root.node.height(), 1);
            t.insert(MyData { a: 2 }, None);
            assert_eq!(
                *t.root.node.key_ref::<MyData, Option<i32>>(),
                MyData { a: 1 }
            );
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
            let x = cursors.erase_then_next();

            assert!(x.is_some());
            assert_eq!(x.unwrap().0, 55);

            assert_eq!(*cursors.get().unwrap().0, 56);
            cursors.prev();
            assert_eq!(*cursors.get().unwrap().0, 54);

            let x = cursors.erase_then_prev();

            assert!(x.is_some());
            assert_eq!(x.unwrap().0, 54);

            assert_eq!(*cursors.get().unwrap().0, 53);
            cursors.next();
            assert_eq!(*cursors.get().unwrap().0, 56);

            *cursors.get_mut().unwrap().1 = None;
            assert_eq!(*cursors.get().unwrap().1, None);

            cursors.erase_then_prev();
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

    #[test]
    fn test_entry() {
        let xs = [(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)];

        let mut map: OrdMap<_, _> = xs.iter().cloned().collect();

        match map.entry(1) {
            Vacant(_) => unreachable!(),
            Occupied(mut view) => {
                assert_eq!(view.get(), &10);
                assert_eq!(view.insert(100), 10);
            }
        }
        assert_eq!(map.get(&1).unwrap(), &100);
        assert_eq!(map.len(), 6);

        match map.entry(2) {
            Vacant(_) => unreachable!(),
            Occupied(mut view) => {
                let v = view.get_mut();
                let new_v = (*v) * 10;
                *v = new_v;
            }
        }
        assert_eq!(map.get(&2).unwrap(), &200);
        assert_eq!(map.len(), 6);

        match map.entry(3) {
            Vacant(_) => unreachable!(),
            Occupied(view) => {
                assert_eq!(view.remove(), 30);
            }
        }
        assert_eq!(map.get(&3), None);
        assert_eq!(map.len(), 5);

        match map.entry(10) {
            Occupied(_) => unreachable!(),
            Vacant(view) => {
                assert_eq!(*view.insert(1000), 1000);
            }
        }
        assert_eq!(map.get(&10).unwrap(), &1000);
        assert_eq!(map.len(), 6);

        let mut map: OrdMap<Rc<String>, u32> = OrdMap::new();
        map.insert(Rc::new("Stringthing".to_string()), 15);

        let my_key = Rc::new("Stringthing".to_string());

        if let Occupied(entry) = map.entry(my_key) {
            // Also replace the key with a handle to our other key.
            let (old_key, old_value): (Rc<String>, u32) = entry.replace_entry(16);
            assert_eq!(Rc::strong_count(&old_key), 1);
            assert_eq!(old_value, 15);
        }
    }
}
