extern crate hash_ord;
extern crate rand;

use hash_ord::ord_map::OrdMap;
use std::ops::Bound::{Excluded, Included, Unbounded};
use std::cell::RefCell;
use hash_ord::ord_map::Entry::Vacant;
use hash_ord::ord_map::Entry::Occupied;
use std::rc::Rc;

type DefaultType = OrdMap<i32, Option<i32>>;

struct Node<'a> {
    b: &'a RefCell<usize>,
}
impl<'a> Drop for Node<'a> {
    fn drop(&mut self) {
        *self.b.borrow_mut() += 1;
    }
}

fn default_make_avl_element(n: usize) -> Vec<i32> {
    let mut v = vec![0i32; n];
    for idx in 0..v.len() {
        v[idx] = idx as i32;
        let pos = rand::random::<usize>() % (idx + 1);
        assert!(pos <= idx);
        v.swap(idx, pos);
    }
    v
}

fn default_build_avl(n: usize) -> DefaultType {
    let v = default_make_avl_element(n);
    let mut t = DefaultType::new();
    assert_eq!(t.len(), 0);
    for d in &v {
        t.insert(*d, Some(-*d));
    }
    t
}

#[test]
fn test_avl_split_off() {
    let cnt = RefCell::new(0);
    let test_num = 100;
    {
        let mut ma = OrdMap::new();
        for i in 0..test_num {
            ma.insert(i, Node { b: &cnt });
        }
        assert_eq!(test_num, ma.len());
        assert_eq!(*cnt.borrow(), 0);
        let mb = ma.split_off(&66);
        assert_eq!(ma.len(), 66);
        assert_eq!(mb.len(), 34);
        assert_eq!(*cnt.borrow(), 0);
        drop(ma);
        assert_eq!(*cnt.borrow(), 66);
        drop(mb);
        assert_eq!(*cnt.borrow(), test_num);
    }
    assert_eq!(*cnt.borrow(), test_num);
}

#[test]
fn test_avl_range() {
    let mut map = OrdMap::new();
    map.insert(3, "a");
    map.insert(5, "b");
    map.insert(8, "c");
    {
        assert_eq!(map.range((Included(&4), Included(&8))).count(), 2);
        assert_eq!(map.range((Included(&4), Excluded(&8))).count(), 1);
        assert_eq!(map.range((Excluded(&5), Included(&8))).count(), 1);
        assert_eq!(map.range((Excluded(&5), Excluded(&8))).count(), 0);

        assert_eq!(map.range((Unbounded, Included(&8))).count(), 3);
        assert_eq!(map.range((Unbounded, Excluded(&8))).count(), 2);
        assert_eq!(map.range(..).count(), 3);

        assert_eq!(map.range((Included(&5), Unbounded)).count(), 2);
        assert_eq!(map.range((Excluded(&3), Unbounded)).count(), 2);
    }
    {
        let mut range_iter = map.range((Included(&4), Included(&8)));
        let v: Vec<_> = range_iter.clone().collect();
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].0, &5);
        assert_eq!(v[1].0, &8);
        assert_eq!(Some((&8, &"c")), range_iter.next_back());
    }
    {
        let mut range_iter = map.range(4..);
        assert_eq!(Some((&8, &"c")), range_iter.next_back());
        assert_eq!(Some((&5, &"b")), range_iter.next_back());
        assert_eq!(None, range_iter.next_back());
        assert_eq!(None, range_iter.next());
    }
    {
        {
            let (k, v) = map.range_mut(4..).next().unwrap();
            assert_eq!(*k, 5);
            *v = "test";
        }
        assert_eq!(map[&5], "test");
    }
}

#[test]
fn test_avl_into_sorted_list() {
    let cnt = RefCell::new(0);
    let test_num = 100usize;
    let mut map = OrdMap::new();
    for i in 0..test_num {
        map.insert(i, Node { b: &cnt });
    }
    let mut o = Vec::<usize>::new();
    for (k, _) in map.into_iter().into_sorted_list().iter() {
        o.push(*k);
    }
    assert_eq!(o.len(), test_num as usize);
    assert_eq!(*cnt.borrow(), test_num);
    let sum: usize = o.iter().sum();
    assert_eq!(sum, (0..test_num).sum());
}

#[test]
fn test_avl_append() {
    let cnt = RefCell::new(0);
    let test_num = 100usize;
    let mut ma = OrdMap::new();
    for i in 0..test_num {
        ma.insert(i, Node { b: &cnt });
    }
    let mut mb = OrdMap::new();
    for i in test_num / 2..test_num * 2 {
        mb.insert(i, Node { b: &cnt });
    }
    assert_eq!(*cnt.borrow(), 0);
    ma.append(&mut mb);
    assert!(ma.check_balanced());
    assert!(ma.check_ord_valid());
    assert_eq!(ma.len(), test_num * 2);
    assert_eq!(mb.len(), 0);
    assert_eq!(*cnt.borrow(), (test_num - test_num / 2));
    drop(mb);
    assert_eq!(*cnt.borrow(), (test_num - test_num / 2));
    drop(ma);
    assert_eq!(*cnt.borrow(), 2 * test_num + (test_num - test_num / 2));
}

#[test]
fn test_avl_entry() {
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

#[test]
fn test_avl_from_iter() {
    let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];
    let map: OrdMap<_, _> = xs.iter().cloned().collect();
    for &(k, v) in &xs {
        assert_eq!(map.get(&k), Some(&v));
    }
}

#[test]
fn test_avl_memory_leak() {
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
fn test_avl_clear() {
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

    let a = default_build_avl(100);
    let mut s = 0;
    for (k, _) in a.into_iter() {
        s += k;
    }
    assert_eq!(s, (0..100).sum());
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
fn test_avl_find() {
    let t = default_build_avl(1000);
    for num in 0..t.len() {
        let x = num as i32;
        assert_eq!(*t.get(&x).unwrap(), Some(-x));
    }
}

#[test]
fn test_avl_erase() {
    let test_num = 100usize;
    let mut t = default_build_avl(test_num);
    assert!(t.check_ord_valid());
    for _ in 0..60 {
        let x = (rand::random::<usize>() % test_num) as i32;
        match t.remove(&x) {
            None => {}
            Some((k, v)) => {
                assert_eq!(v.unwrap(), -x);
                assert_eq!(k, x);
            }
        }
        assert!(!t.contains_key(&x));
    }
    assert!(t.check_ord_valid());
    assert!(t.check_balanced());
}
