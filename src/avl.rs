use std::cmp::{Ordering, max};
use std::ptr;

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
    #[inline]
    fn key_ref(&self) -> &K {
        unsafe { &(*self.0).key }
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
    fn is_left_child(&self) -> bool {
        self.parent().left() == *self
    }

    #[inline]
    fn is_right_child(&self) -> bool {
        self.parent().right() == *self
    }

    #[inline]
    fn next(&self) -> NodePtr<K, V> {
        if self.is_null() {
            return NodePtr::null();
        }
        let mut node = *self;
        if !self.right().is_null() {
            node = node.right();
            while !node.left().is_null() {
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
        if !node.left().is_null() {
            node = node.left();
            while !node.right().is_null() {
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
        unsafe { (*self.0).parent.clone() }
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

pub struct AVLTree<K, V> where K: Ord + Clone {
    root: NodePtr<K, V>,
    count: usize,
}

impl<K, V> AVLTree<K, V> where K: Ord + Clone {
    #[inline]
    pub fn max_height(&self) -> i32 {
        self.root.height()
    }

    #[inline]
    pub fn empty(&self) -> bool {
        self.count == 0
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.count
    }

    unsafe fn first_node(&self) -> NodePtr<K, V> {
        let mut ptr = self.root;
        if ptr.is_null() {
            return NodePtr::null();
        }
        while !ptr.left().is_null() {
            ptr = ptr.left();
        }
        ptr
    }

    unsafe fn last_node_ptr(&self) -> NodePtr<K, V> {
        let mut ptr = self.root;
        if ptr.is_null() {
            return NodePtr::null();
        }
        while !ptr.right().is_null() {
            ptr = ptr.right();
        }
        ptr
    }

    pub fn new() -> Self {
        AVLTree { root: NodePtr::null(), count: 0 }
    }

    #[inline]
    pub fn set(&mut self, key: K, value: V) {
        unsafe {
            let mut duplicate = NodePtr::null();
            let mut cmp_node_ref: *mut NodePtr<K, V> = &mut self.root;
            let mut parent = NodePtr::null();
            while !(*cmp_node_ref).is_null() {
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
            if duplicate.is_null() {
                let new_node = NodePtr::new(key, value);
                AVLTree::link_node(new_node, parent, cmp_node_ref);
                self.node_post_insert(new_node);
                self.count += 1;
            } else {
                duplicate.set_value(value);
            }
        }
    }

    #[inline]
    unsafe fn find_node(&self, what: &K) -> NodePtr<K, V> {
        let mut node = self.root;
        let mut res_node = NodePtr::null();
        while !node.is_null() {
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
        while !node.is_null() {
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
        if !left.right().is_null() {
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
        if !right.left().is_null() {
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

    fn bst_check(&self) -> bool {
        unsafe {
            let mut node = self.first_node();
            if node.is_null() {
                assert_eq!(self.size(), 0);
                return true;
            }
            let mut cnt = 1usize;
            let mut value = &(*node.0).key;
            node = node.next();
            while !node.is_null() {
                let x = &(*node.0).key;
                if *x <= *value {
                    return false;
                }
                value = x;
                node = node.next();
                cnt += 1;
            }
            assert_eq!(cnt, self.count);
            return true;
        }
    }

    fn bst_check_reverse(&self) -> bool {
        unsafe {
            let mut node = self.last_node_ptr();
            if node.is_null() {
                assert_eq!(self.size(), 0);
                return true;
            }
            let mut cnt = 1usize;
            let mut value = &(*node.0).key;
            node = node.prev();
            while !node.is_null() {
                let x = &(*node.0).key;
                if *x >= *value {
                    return false;
                }
                value = x;
                node = node.prev();
                cnt += 1;
            }
            assert_eq!(cnt, self.count);
            return true;
        }
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
        unsafe { !self.find_node(what).is_null() }
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

    pub fn get_mut(&self, what: &K) -> Option<&mut V> {
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
        let mut parent = NodePtr::null();
        if !node.left().is_null() && !node.right().is_null() {
            let old = node;
            node = node.right();
            while !node.left().is_null() {
                node = node.left();
            }
            let child = node.right();
            parent = node.parent();
            if !child.is_null() {
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
            if !old.right().is_null() {
                old.right().set_parent(node);
            }
        } else {
            let child = if node.left().is_null() {
                node.right()
            } else {
                node.left()
            };
            parent = node.parent();
            self.child_replace(node, child, parent);
            if !child.is_null() {
                child.set_parent(parent);
            }
        }
        if !parent.is_null() {
            self.rebalance_node(parent);
        }
    }

    #[inline]
    unsafe fn rebalance_node(&mut self, mut node: NodePtr<K, V>) {
        while !node.is_null() {
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
        if !node.is_null() {
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

}

#[test]
fn just_for_compile() {}

impl<K, V> Drop for AVLTree<K, V> where K: Ord + Clone {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<K, V> Clone for AVLTree<K, V> where K: Ord + Clone, V: Clone {
    fn clone(&self) -> Self {
        unsafe {
            let tree = AVLTree::new();
            tree
        }
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
        unsafe {
            assert!(t.root.is_null());
            t.set(3, None);
            assert_eq!(*t.root.key_ref(), 3);
            assert_eq!(t.root.height(), 1);
            assert!(t.root.left().is_null());
            assert!(t.root.right().is_null());

            t.set(2, None);
            assert_eq!(*t.root.key_ref(), 3);
            assert_eq!(t.root.height(), 2);
            assert!(t.root.left().is_left_child());
            assert!(t.root.right().is_null());
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
        unsafe {
            t.set(3, None);
            assert_eq!(*t.root.key_ref(), 3);
            assert_eq!(t.root.height(), 1);
            t.set(2, None);
            assert_eq!(*t.root.key_ref(), 3);
            assert_eq!(t.root.height(), 2);
            t.set(1, None);
            assert_eq!(*t.root.key_ref(), 2);
            assert_eq!(t.root.height(), 2);
        }
    }

    #[test]
    fn test_avl_rotate_left() {
        let mut t = DefaultType::new();
        unsafe {
            t.set(1, None);
            assert_eq!(*t.root.key_ref(), 1);
            assert_eq!(t.root.height(), 1);
            t.set(2, None);
            assert_eq!(*t.root.key_ref(), 1);
            assert_eq!(t.root.height(), 2);
            t.set(3, None);
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
        unsafe {
            t.set(MyData { a: 1 }, None);
            assert_eq!(*t.root.key_ref(), MyData { a: 1 });
            assert_eq!(t.root.height(), 1);
            t.set(MyData { a: 2 }, None);
            assert_eq!(*t.root.key_ref(), MyData { a: 1 });
            assert_eq!(t.root.height(), 2);
        }
    }

    #[test]
    fn test_avl_find() {
        let mut t = default_build_avl(1000);
        for num in 0..t.size() {
            let x = num as i32;
            unsafe {
                assert_eq!(*t.get_ref(&x).unwrap(), Some(-x));
            }
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
            t.set(*d, Some(-*d));
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
}

