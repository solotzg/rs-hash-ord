extern crate fnv;

use fastbin::{Fastbin, VoidPtr};
use hash_table::{HashIndexPtrOperation, HashNode, HashNodeOperation, HashNodePtrOperation,
                 HashTable, HashUint};
use hash_table;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::{mem, ptr};
use avl_node::{AVLNodePtr, AVLNodePtrBase};
use list::ListHeadPtrFn;
use avl_node;
use std::ops::Index;
use std::borrow::Borrow;
use std::iter::FromIterator;
use self::fnv::FnvBuildHasher as RandomState;

/// A hash map which uses AVL to resolve collision.
///
/// Usually, hashing algorithm seems to play an important role in HashMap. However, any generated
/// hash-value needs to be mapped into a limited collection, and that will brings in collision.
/// Since collision is unavoidable, in this lib, we focus on solving problem in the worst case,
/// such as Collision-Attack.
///
/// As a kind of Self-Balancing BST, AVL is not worse than RBTree, and has a few advantages like
/// simpler structure and smaller height. So we put an AVL under every Hash-Index to resolve
/// collision. When facing Collision Attack, the runtime complexity of STL HashMap is O(n*n), but
/// ours is just O(n log n).
///
/// To improve performance, raw pointer is used frequently. Because Rust uses a similar memory model
/// to C/C++, two classic macros `offset_of` and `container_of` are used to dereference member
/// variables into main struct. `Fastbin` is implemented to reduce the cost of memory allocation.
///
///
/// # Examples
///
/// ```
/// use hash_ord::hash_map::HashMap;
///
/// // type inference lets us omit an explicit type signature (which
/// // would be `HashMap<&str, &str>` in this example).
/// let mut book_reviews = HashMap::new();
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
/// `HashMap` also implements an [`Entry API`](#method.entry), which allows
/// for more complex methods of getting, setting, updating and removing keys and
/// their values:
///
/// ```
/// use hash_ord::hash_map::HashMap;
///
/// // type inference lets us omit an explicit type signature (which
/// // would be `HashMap<&str, u8>` in this example).
/// let mut player_stats = HashMap::new();
///
/// fn random_stat_buff() -> u8 {
///     // could actually return some random value here - let's just return
///     // some fixed value for now
///     42
/// }
///
/// // insert a key only if it doesn't already exist
/// player_stats.entry("health").or_insert(100);
///
/// // insert a key using a function that provides a new value only if it
/// // doesn't already exist
/// player_stats.entry("defence").or_insert_with(random_stat_buff);
///
/// // update a key, guarding against the key possibly not being set
/// let stat = player_stats.entry("attack").or_insert(100);
/// *stat += random_stat_buff();
/// ```
///
/// The easiest way to use `HashMap` with a custom type as key is to derive `Ord` and `Hash`.
///
/// ```
/// use hash_ord::hash_map::HashMap;
/// use std::cmp::Ordering;
///
/// #[derive(Hash, Eq, Debug)]
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
/// //  Use a HashMap to store the vikings' health points.
/// let mut vikings = HashMap::new();
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
/// A `HashMap` with fixed list of elements can be initialized from an array:
///
/// ```
/// use hash_ord::hash_map::HashMap;
///
/// let timber_resources: HashMap<&str, i32> =
///     [("Norway", 100),
///      ("Denmark", 50),
///      ("Iceland", 10)]
///      .iter().cloned().collect();
///
pub struct HashMap<K, V, S = RandomState> {
    entry_fastbin: Fastbin,
    kv_fastbin: Fastbin,
    hash_table: Box<HashTable<K, V>>,
    hash_builder: S,
}

struct InternalHashEntry<K, V> {
    node: HashNode<K>,
    value: *mut V,
}

/// An iterator over the keys of a `HashMap`.
///
/// This `struct` is created by the [`keys`] method on [`HashMap`]. See its
/// documentation for more.
///
/// [`keys`]: struct.HashMap.html#method.keys
/// [`HashMap`]: struct.HashMap.html
pub struct Keys<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
    inner: Iter<'a, K, V, S>,
}

impl<'a, K, V, S> Iterator for Keys<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
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

/// An iterator over the values of a `HashMap`.
///
/// This `struct` is created by the [`values`] method on [`HashMap`]. See its
/// documentation for more.
///
/// [`values`]: struct.HashMap.html#method.values
/// [`HashMap`]: struct.HashMap.html
pub struct Values<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
    inner: Iter<'a, K, V, S>,
}

impl<'a, K, V, S> Iterator for Values<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
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

/// A mutable iterator over the values of a `HashMap`.
///
/// This `struct` is created by the [`values_mut`] method on [`HashMap`]. See its
/// documentation for more.
///
/// [`values_mut`]: struct.HashMap.html#method.values_mut
/// [`HashMap`]: struct.HashMap.html
pub struct ValuesMut<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
    inner: IterMut<'a, K, V, S>,
}

impl<'a, K, V, S> Iterator for ValuesMut<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
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

/// An iterator over the (key, value) of a `HashMap`.
pub struct Iter<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
    inner: *mut InternalHashEntry<K, V>,
    map: &'a HashMap<K, V, S>,
    len: usize,
}

impl<'a, K, V, S> Iterator for Iter<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
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

/// An iterator over the (key, mut value) of a `HashMap`.
pub struct IterMut<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
    inner: *mut InternalHashEntry<K, V>,
    map: &'a HashMap<K, V, S>,
    len: usize,
}

impl<'a, K, V, S> Iterator for IterMut<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
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
fn key_deref_to_kv<K, V>(key: *mut K) -> *mut (K, V) {
    container_of!(key, (K, V), 0)
}

trait HashEntryBase<K, V> {
    fn node_ptr(self) -> *mut HashNode<K>;
    fn value(self) -> *mut V;
    fn set_value(self, value: *mut V);
    fn key(self) -> *mut K;
    fn set_key(self, key: *mut K);
    fn set_hash_value(self, hash_value: HashUint);
}

impl<K, V> HashEntryBase<K, V> for *mut InternalHashEntry<K, V> {
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
        unsafe {
            (*self).value = value;
        }
    }
    #[inline]
    fn key(self) -> *mut K {
        unsafe { (*self).node.key }
    }
    #[inline]
    fn set_key(self, key: *mut K) {
        unsafe {
            (*self).node.key = key;
        }
    }
    #[inline]
    fn set_hash_value(self, hash_value: HashUint) {
        unsafe {
            (*self).node.hash_val = hash_value;
        }
    }
}

trait HashNodeDerefToHashEntry<K, V> {
    fn deref_to_hash_entry(self) -> *mut InternalHashEntry<K, V>;
}

impl<K, V> HashNodeDerefToHashEntry<K, V> for *mut HashNode<K> {
    /// Dereference HashNode to InternalHashEntry
    fn deref_to_hash_entry(self) -> *mut InternalHashEntry<K, V> {
        container_of!(self, InternalHashEntry<K, V>, node)
    }
}

/// Update HashTable with a new InternalHashEntry and return the old one.
#[inline]
unsafe fn hash_table_update<K, V>(
    hash_table: &mut HashTable<K, V>,
    new_entry: *mut InternalHashEntry<K, V>,
) -> *mut InternalHashEntry<K, V>
where
    K: Ord + Hash,
{
    debug_assert!(!new_entry.is_null());
    let new_node = new_entry.node_ptr();
    let duplicate = hash_table.hash_add(new_node);
    if !duplicate.is_null() {
        return duplicate.deref_to_hash_entry();
    }
    ptr::null_mut()
}

/// Alloc an InternalHashEntry in `Fastbin`
#[inline]
fn entry_alloc<K, V>(
    entry_fastbin: &mut Fastbin,
    key: *mut K,
    value: *mut V,
    hash_value: HashUint,
) -> *mut InternalHashEntry<K, V> {
    let entry = entry_fastbin.alloc() as *mut InternalHashEntry<K, V>;
    debug_assert!(!entry.is_null());
    entry.set_value(value);
    entry.set_key(key);
    entry.set_hash_value(hash_value);
    entry
}

/// Alloc a (Key, Value) pair in `Fastbin`
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

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`HashMap`].
///
/// [`HashMap`]: struct.HashMap.html
/// [`entry`]: struct.HashMap.html#method.entry
pub enum Entry<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V, S>),

    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V, S>),
}

impl<'a, K, V, S> Entry<'a, K, V, S> {
    /// Returns a reference to this entry's key.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map: HashMap<&str, u32> = HashMap::new();
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    pub fn key(&self) -> &K {
        match *self {
            Entry::Occupied(ref entry) => entry.key(),
            Entry::Vacant(ref entry) => entry.key(),
        }
    }
}

impl<'a, K, V, S> Entry<'a, K, V, S>
where
    K: Ord + Hash,
    S: BuildHasher,
{
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map: HashMap<&str, u32> = HashMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    ///
    /// *map.entry("poneyland").or_insert(12) += 10;
    /// assert_eq!(map["poneyland"], 22);
    /// ```
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map: HashMap<&str, String> = HashMap::new();
    /// let s = "hoho".to_string();
    ///
    /// map.entry("poneyland").or_insert_with(|| s);
    ///
    /// assert_eq!(map["poneyland"], "hoho".to_string());
    /// ```
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map: HashMap<&str, u32> = HashMap::new();
    ///
    /// map.entry("poneyland")
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map["poneyland"], 42);
    ///
    /// map.entry("poneyland")
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map["poneyland"], 43);
    /// ```
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

/// A view into an occupied entry in a `HashMap`.
/// It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct OccupiedEntry<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
    key: Option<K>,
    hash_entry: *mut InternalHashEntry<K, V>,
    hash_map_mut: &'a mut HashMap<K, V, S>,
}

/// A view into a vacant entry in a `HashMap`.
/// It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct VacantEntry<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
    hash_value: HashUint,
    key: K,
    parent: AVLNodePtr,
    link: *mut AVLNodePtr,
    hash_map_mut: &'a mut HashMap<K, V, S>,
}

impl<'a, K, V, S> OccupiedEntry<'a, K, V, S>
where
    K: Ord + Hash,
    S: BuildHasher,
{
    /// Take the ownership of the key and value from the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::{HashMap, Entry};
    ///
    /// let mut map: HashMap<&str, u32> = HashMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     // We delete the entry from the map.
    ///     o.remove_entry();
    /// }
    ///
    /// assert_eq!(map.contains_key("poneyland"), false);
    /// ```
    pub fn remove_entry(self) -> (K, V) {
        let hash_entry = self.hash_entry;
        self.hash_map_mut.erase(hash_entry).unwrap()
    }

    /// Takes the value out of the entry, and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::{HashMap, Entry};
    ///
    /// let mut map: HashMap<&str, u32> = HashMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     assert_eq!(o.remove(), 12);
    /// }
    ///
    /// assert_eq!(map.contains_key("poneyland"), false);
    /// ```
    pub fn remove(self) -> V {
        self.remove_entry().1
    }
}

impl<'a, K, V, S> OccupiedEntry<'a, K, V, S> {
    /// Gets a reference to the key in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map: HashMap<&str, u32> = HashMap::new();
    /// map.entry("poneyland").or_insert(12);
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    pub fn key(&self) -> &K {
        unsafe { &*self.hash_entry.key() }
    }

    /// Returns a key that was used for search.
    ///
    /// The key was retained for further use.
    fn take_key(&mut self) -> Option<K> {
        self.key.take()
    }

    /// Replaces the key in the hash map with the key used to create this entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::{HashMap, Entry};
    /// use std::rc::Rc;
    ///
    /// let mut map: HashMap<Rc<String>, u32> = HashMap::new();
    /// let mut known_strings: Vec<Rc<String>> = Vec::new();
    ///
    /// // Initialise known strings, run program, etc.
    ///
    /// reclaim_memory(&mut map, &known_strings);
    ///
    /// fn reclaim_memory(map: &mut HashMap<Rc<String>, u32>, known_strings: &[Rc<String>] ) {
    ///     for s in known_strings {
    ///         if let Entry::Occupied(entry) = map.entry(s.clone()) {
    ///             // Replaces the entry's key with our version of it in `known_strings`.
    ///             entry.replace_key();
    ///         }
    ///     }
    /// }
    /// ```
    pub fn replace_key(self) -> K {
        let old_key = unsafe { &mut *self.hash_entry.key() };
        mem::replace(old_key, self.key.unwrap())
    }

    /// Gets a reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::{HashMap, Entry};
    ///
    /// let mut map: HashMap<&str, u32> = HashMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     assert_eq!(o.get(), &12);
    /// }
    /// ```
    pub fn get(&self) -> &V {
        unsafe { &*self.hash_entry.value() }
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::{HashMap, Entry};
    ///
    /// let mut map: HashMap<&str, u32> = HashMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// if let Entry::Occupied(mut o) = map.entry("poneyland") {
    ///      *o.get_mut() += 10;
    /// }
    ///
    /// assert_eq!(map["poneyland"], 22);
    /// ```
    pub fn get_mut(&mut self) -> &mut V {
        unsafe { &mut *self.hash_entry.value() }
    }

    /// Converts the OccupiedEntry into a mutable reference to the value in the entry
    /// with a lifetime bound to the map itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::{HashMap, Entry};
    ///
    /// let mut map: HashMap<&str, u32> = HashMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     *o.into_mut() += 10;
    /// }
    ///
    /// assert_eq!(map["poneyland"], 22);
    /// ```
    pub fn into_mut(self) -> &'a mut V {
        unsafe { &mut *self.hash_entry.value() }
    }

    /// Sets the value of the entry, and returns the entry's old value.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::{HashMap, Entry};
    ///
    /// let mut map: HashMap<&str, u32> = HashMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(mut o) = map.entry("poneyland") {
    ///     assert_eq!(o.insert(15), 12);
    /// }
    ///
    /// assert_eq!(map["poneyland"], 15);
    /// ```
    pub fn insert(&mut self, mut value: V) -> V {
        let old_value = self.get_mut();
        mem::swap(&mut value, old_value);
        value
    }

    /// Replaces the entry, returning the old key and value. The new key in the hash map will be
    /// the key used to create this entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::{HashMap, Entry};
    /// use std::rc::Rc;
    ///
    /// let mut map: HashMap<Rc<String>, u32> = HashMap::new();
    /// map.insert(Rc::new("Stringthing".to_string()), 15);
    ///
    /// let my_key = Rc::new("Stringthing".to_string());
    ///
    /// if let Entry::Occupied(entry) = map.entry(my_key) {
    ///     // Also replace the key with a handle to our other key.
    ///     let (old_key, old_value): (Rc<String>, u32) = entry.replace_entry(16);
    ///     assert_eq!(Rc::strong_count(&old_key), 1);
    ///     assert!(old_value == 15);
    /// }
    ///
    /// ```
    pub fn replace_entry(self, value: V) -> (K, V) {
        let (old_key, old_value) =
            unsafe { (&mut *self.hash_entry.key(), &mut *self.hash_entry.value()) };
        let old_key = mem::replace(old_key, self.key.unwrap());
        let old_value = mem::replace(old_value, value);
        (old_key, old_value)
    }
}

impl<'a, K, V, S> VacantEntry<'a, K, V, S> {
    /// Gets a reference to the key that would be used when inserting a value
    /// through the `VacantEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map: HashMap<&str, u32> = HashMap::new();
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Take ownership of the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::{HashMap, Entry};
    ///
    /// let mut map: HashMap<&str, u32> = HashMap::new();
    ///
    /// if let Entry::Vacant(v) = map.entry("poneyland") {
    ///     v.into_key();
    /// }
    /// ```
    pub fn into_key(self) -> K {
        self.key
    }
}

impl<'a, K, V, S> VacantEntry<'a, K, V, S>
where
    K: Ord + Hash,
    S: BuildHasher,
{
    unsafe fn _internal_insert(self, value: V) -> &'a mut V {
        let hash_value = self.hash_value;
        let index = self.hash_map_mut.hash_table.get_hash_index(hash_value);
        let key = self.key;
        let kv_ptr = self.hash_map_mut.kv_alloc(key, value);
        let new_entry = self.hash_map_mut.entry_alloc(
            &mut (*kv_ptr).0 as *mut K,
            &mut (*kv_ptr).1 as *mut V,
            hash_value,
        );
        let new_node = new_entry.node_ptr();
        if index.avl_root_node().is_null() {
            self.hash_map_mut
                .hash_table
                .head_ptr()
                .list_add_tail(index.node_ptr());
        }
        avl_node::link_node(new_node.avl_node_ptr(), self.parent, self.link);
        avl_node::node_post_insert(new_node.avl_node_ptr(), index.avl_root_ptr());
        self.hash_map_mut.hash_table.inc_count(1);
        let new_len = self.hash_map_mut.len();
        self.hash_map_mut.rehash(new_len);
        &mut *new_entry.value()
    }

    /// Sets the value of the entry with the VacantEntry's key,
    /// and returns a mutable reference to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::{HashMap, Entry};
    ///
    /// let mut map: HashMap<&str, u32> = HashMap::new();
    /// if let Entry::Vacant(o) = map.entry("poneyland") {
    ///     o.insert(37);
    /// }
    /// assert_eq!(map["poneyland"], 37);
    /// ```
    pub fn insert(self, value: V) -> &'a mut V {
        unsafe { self._internal_insert(value) }
    }
}

impl<K, V, S> HashMap<K, V, S> {
    fn recurse_destroy<F>(&mut self, node: avl_node::AVLNodePtr, f: &mut F)
    where
        F: FnMut((K, V)),
    {
        if node.left().not_null() {
            self.recurse_destroy(node.left(), f);
        }
        if node.right().not_null() {
            self.recurse_destroy(node.right(), f);
        }
        let hash_node = node.avl_hash_deref_mut::<K>();
        let entry: *mut InternalHashEntry<K, V> = hash_node.deref_to_hash_entry();
        self.entry_fastbin.del(entry as VoidPtr);
        let kv_ptr = key_deref_to_kv::<K, V>(hash_node.key_ptr());
        unsafe { (*f)(ptr::read(kv_ptr)) };
        self.kv_fastbin.del(kv_ptr as VoidPtr);
        self.hash_table.dec_count(1);
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory
    /// for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut a = HashMap::new();
    /// a.insert(1, "a");
    /// a.clear();
    /// assert!(a.is_empty());
    /// ```
    pub fn clear(&mut self) {
        let mut destroy_callback = |_| {};
        loop {
            let node = self.hash_table.pop_first_index();
            if node.is_null() {
                break;
            }
            self.recurse_destroy(node, &mut destroy_callback);
        }
        debug_assert_eq!(self.hash_table.size(), 0);
    }

    /// Returns the number of elements the map can hold without reallocating.
    ///
    /// This number is a lower bound; the `HashMap<K, V>` might be able to hold
    /// more, but is guaranteed to be able to hold at least this many.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// let map: HashMap<isize, isize> = HashMap::with_capacity(100);
    /// assert!(map.capacity() >= 100);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.hash_table.capacity()
    }

    /// Returns the maximum node count under a simgle HashIndex
    #[inline]
    pub fn get_max_node_of_single_index(&self) -> i32 {
        self.hash_table.get_max_node_of_single_index()
    }

    #[inline]
    fn first(&self) -> *mut InternalHashEntry<K, V> {
        let hash_node = self.hash_table.node_first();
        if hash_node.is_null() {
            return ptr::null_mut();
        }
        hash_node.deref_to_hash_entry()
    }

    #[inline]
    fn last(&self) -> *mut InternalHashEntry<K, V> {
        let hash_node = self.hash_table.node_last();
        if hash_node.is_null() {
            return ptr::null_mut();
        }
        hash_node.deref_to_hash_entry()
    }

    #[inline]
    fn next(&self, entry: *mut InternalHashEntry<K, V>) -> *mut InternalHashEntry<K, V> {
        let hash_node = self.hash_table.node_next(entry.node_ptr());
        if hash_node.is_null() {
            return ptr::null_mut();
        }
        hash_node.deref_to_hash_entry()
    }

    #[inline]
    fn prev(&self, entry: *mut InternalHashEntry<K, V>) -> *mut InternalHashEntry<K, V> {
        let hash_node = self.hash_table.node_prev(entry.node_ptr());
        if hash_node.is_null() {
            return ptr::null_mut();
        }
        hash_node.deref_to_hash_entry()
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut a = HashMap::new();
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.hash_table.size()
    }

    /// Returns true if the map contains no element.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut a = HashMap::new();
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn entry_alloc(
        &mut self,
        key: *mut K,
        value: *mut V,
        hash_value: HashUint,
    ) -> *mut InternalHashEntry<K, V> {
        entry_alloc(&mut self.entry_fastbin, key, value, hash_value)
    }

    #[inline]
    fn kv_alloc(&mut self, key: K, value: V) -> *mut (K, V) {
        kv_alloc(&mut self.kv_fastbin, key, value)
    }

    /// An iterator visiting all keys in arbitrary order.
    /// The iterator element type is `&'a K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for key in map.keys() {
    ///     println!("{}", key);
    /// }
    /// ```
    pub fn keys(&self) -> Keys<K, V, S> {
        Keys { inner: self.iter() }
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&'a V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for val in map.values() {
    ///     println!("{}", val);
    /// }
    /// ```
    pub fn values(&self) -> Values<K, V, S> {
        Values { inner: self.iter() }
    }

    /// An iterator visiting all values mutably in arbitrary order.
    /// The iterator element type is `&'a mut V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map = HashMap::new();
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
    pub fn values_mut(&mut self) -> ValuesMut<K, V, S> {
        ValuesMut {
            inner: self.iter_mut(),
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    /// The iterator element type is `(&'a K, &'a V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for (key, val) in map.iter() {
    ///     println!("key: {} val: {}", key, val);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<K, V, S> {
        Iter {
            inner: self.first(),
            map: self,
            len: self.len(),
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order,
    /// with mutable references to the values.
    /// The iterator element type is `(&'a K, &'a mut V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map = HashMap::new();
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
    pub fn iter_mut(&mut self) -> IterMut<K, V, S> {
        IterMut {
            inner: self.first(),
            map: self,
            len: self.len(),
        }
    }

    /// Clears the map, returning all key-value pairs as an iterator. Keeps the
    /// allocated memory for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut a = HashMap::new();
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    ///
    /// for (k, v) in a.drain().take(1) {
    ///     assert!(k == 1 || k == 2);
    ///     assert!(v == "a" || v == "b");
    /// }
    ///
    /// assert!(a.is_empty());
    /// ```
    #[inline]
    pub fn drain(&mut self) -> Drain<K, V, S> {
        Drain {
            entry: self.first(),
            map: self,
        }
    }

    fn erase(&mut self, entry: *mut InternalHashEntry<K, V>) -> Option<(K, V)> {
        debug_assert!(!entry.is_null());
        debug_assert!(!entry.node_ptr().avl_node_ptr().empty());
        self.hash_table.hash_erase(entry.node_ptr());
        let kv = key_deref_to_kv::<K, V>(entry.key());
        self.entry_fastbin.del(entry as VoidPtr);
        let res = unsafe { Some(ptr::read(kv)) };
        self.kv_fastbin.del(kv as VoidPtr);
        res
    }
}

impl<K, V, S> HashMap<K, V, S>
where
    K: Ord + Hash,
    S: BuildHasher,
{
    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut letters = HashMap::new();
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
    pub fn entry(&mut self, mut key: K) -> Entry<K, V, S> {
        let hash_val = self.make_hash(&key);
        let link = self.hash_table.get_hash_index(hash_val).avl_root_node_ptr();
        let (duplicate, parent, link) =
            unsafe { hash_table::find_duplicate_hash_node(link, &mut key as *mut K, hash_val) };
        if duplicate.is_null() {
            return Entry::Vacant(VacantEntry {
                hash_value: hash_val,
                key,
                parent,
                link,
                hash_map_mut: self,
            });
        } else {
            return Entry::Occupied(OccupiedEntry {
                key: Some(key),
                hash_entry: duplicate.deref_to_hash_entry(),
                hash_map_mut: self,
            });
        };
    }

    #[inline]
    fn make_hash<X: ?Sized>(&self, x: &X) -> HashUint
    where
        X: Hash,
    {
        hash_table::make_hash(&self.hash_builder, x)
    }

    /// Creates an empty `HashMap` which will use the given hash builder to hash
    /// keys.
    ///
    /// The created map has the default initial capacity.
    ///
    /// Warning: `hash_builder` is normally randomly generated, and
    /// is designed to allow HashMaps to be resistant to attacks that
    /// cause many collisions and very poor performance. Setting it
    /// manually using this function can expose a DoS attack vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    /// use std::collections::hash_map::RandomState;
    ///
    /// let s = RandomState::new();
    /// let mut map = HashMap::with_hasher(s);
    /// map.insert(1, 2);
    /// ```
    pub fn with_hasher(hash_builder: S) -> Self {
        HashMap::with_capacity_and_hasher(0, hash_builder)
    }

    #[inline]
    fn find<Q: ?Sized>(&self, q: &Q) -> *mut InternalHashEntry<K, V>
    where
        K: Borrow<Q>,
        Q: Ord + Hash,
    {
        let node = self.hash_table.hash_find(self.make_hash(q), q);
        if node.is_null() {
            ptr::null_mut()
        } else {
            node.deref_to_hash_entry()
        }
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// `Hash` and `Ord` on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    #[inline]
    pub fn get<Q: ?Sized>(&self, q: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Ord,
    {
        let entry = self.find(q);
        if entry.is_null() {
            return None;
        }
        unsafe { Some(&(*entry.value())) }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// `Hash` and `Ord` on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert(1, "a");
    /// if let Some(x) = map.get_mut(&1) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map[&1], "b");
    /// ```
    #[inline]
    pub fn get_mut<Q: ?Sized>(&mut self, q: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Ord,
    {
        let entry = self.find(q);
        if entry.is_null() {
            return None;
        }
        unsafe { Some(&mut (*entry.value())) }
    }

    #[inline]
    fn rehash(&mut self, len: usize) {
        self.hash_table.rehash(len);
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `HashMap`. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map: HashMap<&str, isize> = HashMap::new();
    /// map.reserve(10);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.try_reserve(additional);
    }

    pub fn try_reserve(&mut self, additional: usize) {
        let remaining = self.capacity() - self.len();
        if remaining < additional {
            match self.len().checked_add(additional) {
                None => panic!("capacity overflow"),
                Some(min_cap) => self.rehash(min_cap),
            };
        }
        // we use BST to restore concrete data, so there is no need to do
        // any thing if capacity is equal to len
    }

    /// Returns true if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// `Hash` and `Ord` on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
    pub fn contains_key<Q: ?Sized>(&self, q: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Ord,
    {
        !self.find(q).is_null()
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, update map with new (key, value)
    /// and return the old one.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some((37, "b")));
    /// assert_eq!(map[&37], "c");
    /// ```
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        self.reserve(1);
        let hash_value = self.make_hash(&key);
        let kv_ptr = self.kv_alloc(key, value);
        let new_entry = unsafe {
            self.entry_alloc(
                &mut (*kv_ptr).0 as *mut K,
                &mut (*kv_ptr).1 as *mut V,
                hash_value,
            )
        };
        let old_entry = unsafe { hash_table_update(self.hash_table.as_mut(), new_entry) };
        if old_entry.is_null() {
            None
        } else {
            let old_kv_ptr = key_deref_to_kv(old_entry.key());
            let res = unsafe { Some(ptr::read(old_kv_ptr)) };
            self.kv_fastbin.del(old_kv_ptr as VoidPtr);
            self.entry_fastbin.del(old_entry as VoidPtr);
            res
        }
    }

    /// Removes a key from the map, returning the stored key and value if the
    /// key was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// `Hash` and `Ord` on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(&1), Some((1, "a")));
    /// assert_eq!(map.remove(&1), None);
    /// ```
    #[inline]
    pub fn remove<Q: ?Sized>(&mut self, q: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Ord,
    {
        let entry = self.find(q);
        if entry.is_null() {
            return None;
        }
        self.erase(entry)
    }

    /// Creates an empty `HashMap` with the specified capacity, using `hash_builder`
    /// to hash the keys.
    ///
    /// The hash map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash map will not allocate.
    ///
    /// Warning: `hash_builder` is normally randomly generated, and
    /// is designed to allow HashMaps to be resistant to attacks that
    /// cause many collisions and very poor performance. Setting it
    /// manually using this function can expose a DoS attack vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    /// use std::collections::hash_map::RandomState;
    ///
    /// let s = RandomState::new();
    /// let mut map = HashMap::with_capacity_and_hasher(10, s);
    /// map.insert(1, 2);
    /// ```
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> HashMap<K, V, S> {
        let mut hash_map = HashMap {
            entry_fastbin: Fastbin::new(mem::size_of::<InternalHashEntry<K, V>>()),
            kv_fastbin: Fastbin::new(mem::size_of::<(K, V)>()),
            hash_table: hash_table::HashTable::new_with_box(),
            hash_builder,
        };
        hash_map.reserve(capacity);
        hash_map
    }

    /// Shrinks the capacity of the map as much as possible. It will drop
    /// down as much as possible while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    ///
    /// let mut map: HashMap<isize, isize> = HashMap::with_capacity(100);
    /// map.insert(1, 2);
    /// map.insert(3, 4);
    /// assert!(map.capacity() >= 100);
    /// map.shrink_to_fit();
    /// assert!(map.capacity() >= 2);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        let limit = hash_table::calc_limit(self.len());
        let old_cap = self.capacity();
        let new_cap = {
            let mut tmp = old_cap / 2;
            let mut res = old_cap;
            while tmp >= limit {
                res = tmp;
                tmp /= 2;
            }
            res
        };
        if new_cap >= old_cap {
            return;
        }
        let mut new_entry_fastbin = Fastbin::new(mem::size_of::<InternalHashEntry<K, V>>());
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
                if node.is_null() {
                    break;
                }
                self.recurse_destroy(node, &mut destroy_callback);
            }
            debug_assert_eq!(self.hash_table.size(), 0);
        }
        for kv_ptr in new_kv_vec {
            unsafe {
                let key_ptr = &mut (*kv_ptr).0 as *mut K;
                let value_ptr = &mut (*kv_ptr).1 as *mut V;
                let entry = entry_alloc(
                    &mut new_entry_fastbin,
                    key_ptr,
                    value_ptr,
                    self.make_hash(&(*key_ptr)),
                );
                hash_table_update(&mut new_hash_table, entry);
            }
        }
        self.kv_fastbin = new_kv_fastbin;
        self.entry_fastbin = new_entry_fastbin;
        self.hash_table = new_hash_table;
    }
}

impl<K, V> HashMap<K, V, RandomState>
where
    K: Hash + Ord,
{
    /// Creates an empty `HashMap`.
    ///
    /// The hash map is initially created with a capacity of 0, so it will not allocate until it
    /// is first inserted into.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_ord::hash_map::HashMap;
    /// let mut map: HashMap<&str, isize> = HashMap::new();
    /// ```
    #[inline]
    pub fn new() -> HashMap<K, V, RandomState> {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> HashMap<K, V, RandomState> {
        let mut hash_map = HashMap::<K, V, RandomState>::default();
        hash_map.reserve(capacity);
        hash_map
    }
}

impl<K, V, S> Default for HashMap<K, V, S>
where
    K: Ord + Hash,
    S: BuildHasher + Default,
{
    /// Creates an empty `HashMap<K, V, S>`, with the `Default` value for the hasher.
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

/// A draining iterator over the entries of a `HashMap`.
///
/// This `struct` is created by the [`drain`] method on [`HashMap`]. See its
/// documentation for more.
///
/// [`drain`]: struct.HashMap.html#method.drain
/// [`HashMap`]: struct.HashMap.html
pub struct Drain<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
    entry: *mut InternalHashEntry<K, V>,
    map: &'a mut HashMap<K, V, S>,
}

impl<'a, K, V, S> Drop for Drain<'a, K, V, S>
where
    K: 'a,
    V: 'a,
    S: 'a,
{
    fn drop(&mut self) {
        for _ in self {}
    }
}

impl<'a, K, V, S> Iterator for Drain<'a, K, V, S> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<(K, V)> {
        let entry = self.entry;
        if entry.is_null() {
            return None;
        }
        self.entry = self.map.next(entry);
        self.map.erase(entry)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.map.len(), Some(self.map.len()))
    }
}

impl<'a, K, V, S> ExactSizeIterator for Drain<'a, K, V, S> {
    #[inline]
    fn len(&self) -> usize {
        self.map.len()
    }
}

impl<'a, K, Q, V, S> Index<&'a Q> for HashMap<K, V, S>
where
    Q: ?Sized + Hash + Ord,
    K: Hash + Ord + Borrow<Q>,
    S: BuildHasher,
{
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the `HashMap`.
    #[inline]
    fn index(&self, q: &Q) -> &Self::Output {
        self.get(q).expect("no entry found for key")
    }
}

impl<K, V, S> Extend<(K, V)> for HashMap<K, V, S>
where
    K: Ord + Hash,
    S: BuildHasher,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
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

impl<'a, K, V, S> Extend<(&'a K, &'a V)> for HashMap<K, V, S>
where
    K: Ord + Hash + Copy,
    V: Copy,
    S: BuildHasher,
{
    fn extend<T: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: T) {
        self.extend(iter.into_iter().map(|(&key, &value)| (key, value)));
    }
}

impl<'a, K, V, S> IntoIterator for &'a HashMap<K, V, S>
where
    K: Ord + Hash,
    S: BuildHasher,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V, S>;

    fn into_iter(self) -> Iter<'a, K, V, S> {
        self.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut HashMap<K, V, S>
where
    K: Ord + Hash,
    S: BuildHasher,
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V, S>;

    fn into_iter(self) -> IterMut<'a, K, V, S> {
        self.iter_mut()
    }
}

impl<K, V, S> IntoIterator for HashMap<K, V, S>
where
    K: Ord + Hash,
    S: BuildHasher,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V, S>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            entry: self.first(),
            map: self,
        }
    }
}

/// An owning iterator over the entries of a `HashMap`.
///
/// This `struct` is created by the [`into_iter`] method on [`HashMap`][`HashMap`]
/// (provided by the `IntoIterator` trait). See its documentation for more.
///
/// [`into_iter`]: struct.HashMap.html#method.into_iter
/// [`HashMap`]: struct.HashMap.html
pub struct IntoIter<K, V, S>
where
    K: Ord + Hash,
    S: BuildHasher,
{
    entry: *mut InternalHashEntry<K, V>,
    map: HashMap<K, V, S>,
}

impl<K, V, S> Drop for IntoIter<K, V, S>
where
    K: Ord + Hash,
    S: BuildHasher,
{
    fn drop(&mut self) {
        for (_, _) in self {}
    }
}

impl<K, V, S> Iterator for IntoIter<K, V, S>
where
    K: Ord + Hash,
    S: BuildHasher,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.entry;
        if entry.is_null() {
            return None;
        }
        self.entry = self.map.next(entry);
        self.map.erase(entry)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.map.len(), Some(self.map.len()))
    }
}

impl<K, V, S> FromIterator<(K, V)> for HashMap<K, V, S>
where
    K: Ord + Hash,
    S: BuildHasher + Default,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> HashMap<K, V, S> {
        let mut map = HashMap::with_hasher(Default::default());
        map.extend(iter);
        map
    }
}

impl<K, V, S> Clone for HashMap<K, V, S>
where
    K: Ord + Hash + Clone,
    V: Clone,
    S: BuildHasher + Clone,
{
    fn clone(&self) -> Self {
        let mut map = HashMap::with_capacity_and_hasher(self.len(), self.hash_builder.clone());
        for (k, v) in self.iter() {
            map.insert(k.clone(), v.clone());
        }
        map
    }
}

impl<K, V, S> PartialEq for HashMap<K, V, S>
where
    K: Ord + Hash,
    V: PartialEq,
    S: BuildHasher,
{
    fn eq(&self, other: &HashMap<K, V, S>) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter()
            .all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<K, V, S> Eq for HashMap<K, V, S>
where
    K: Ord + Hash,
    V: Eq,
    S: BuildHasher,
{
}

#[cfg(test)]
mod test {
    use hash_map::HashMap;
    use std::cell::RefCell;
    use hash_map::Entry::*;

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

    #[test]
    fn test_empty_entry() {
        let mut m: HashMap<isize, bool> = HashMap::new();
        match m.entry(0) {
            Occupied(_) => panic!(),
            Vacant(_) => {}
        }
        assert!(*m.entry(0).or_insert(true));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn test_entry() {
        let xs = [(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)];

        let mut map: HashMap<_, _> = xs.iter().cloned().collect();

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
    }
}
