use std::collections::{LinkedList, HashMap};
use std::ptr::NonNull;
use std::cmp::{Ordering, max};
use std::mem;

pub struct Node<K, V> {
    left: Option<NonNull<Node<K, V>>>,
    right: Option<NonNull<Node<K, V>>>,
    parent: Option<NonNull<Node<K, V>>>,
    key: K,
    value: V,
    height: i32,
}

impl<K: Ord, V> Node<K, V> {
    pub unsafe fn node_next(mut node: Option<NonNull<Node<K, V>>>) -> Option<NonNull<Node<K, V>>> {
        match node {
            None => {return None;},
            Some(ptr) => {
                match ptr.as_ref().right {
                    None => {
                        while true {
                            let last = node;
                            node = node.unwrap().as_ref().parent;
                            if node.is_none() {break;}
                            if cmp_node_ptr(&node.unwrap().as_ref().left, &last) {break;}
                        }
                    },
                    Some(_) => {
                        node = ptr.as_ref().right;
                        while node.unwrap().as_ref().left.is_some() {
                            node = node.unwrap().as_ref().left;
                        }
                    }
                }
            }
        }
        node
    }
}

pub struct Tree<K: Ord, V> {
    root: Option<NonNull<Node<K, V>>>,
}

#[inline]
fn cmp_node_ptr<K, V>(a: &Option<NonNull<Node<K, V>>>, b: &Option<NonNull<Node<K, V>>>) -> bool {
    match *a {
        Some(x) => {
            match *b {
                Some(y) => { x.as_ptr() == y.as_ptr() }
                None => false
            }
        }
        None => { b.is_none() }
    }
}


impl<K: Ord, V> Tree<K, V> {

    unsafe fn avl_node_first(&self) -> Option<NonNull<Node<K, V>>> {
        let mut node = self.root;
        if node.is_none() {
            return None;
        }
        while node.unwrap().as_ref().left.is_some() {
            node = node.unwrap().as_ref().left;
        }
        node
    }

    unsafe fn avl_node_last(&self) -> Option<NonNull<Node<K, V>>> {
        let mut node = self.root;
        if node.is_none() {
            return None;
        }
        while node.unwrap().as_ref().right.is_some() {
            node = node.unwrap().as_ref().right;
        }
        node
    }

    fn new() -> Self {
        Tree { root: None }
    }

    #[inline]
    unsafe fn avl_node_find_link(&mut self, key: K, value: V) -> (Option<NonNull<Node<K, V>>>, Option<(Option<NonNull<Node<K, V>>>, K, V)>) {
        let mut duplicate = None;
        let mut cmp_node = &mut self.root;
        let mut parent = None;
        while let Some(node) = *cmp_node {
            parent = *cmp_node;
            match key.cmp(&node.as_ref().key) {
                Ordering::Less => {
                    cmp_node = &mut (*node.as_ptr()).left;
                }
                Ordering::Equal => {
                    duplicate = *cmp_node;
                    break;
                }
                Ordering::Greater => {
                    cmp_node = &mut (*node.as_ptr()).right;
                }
            }
        }
        if duplicate.is_none() {
            let node_to_insert = Tree::avl_node_link(Box::new(Node {
                left: None,
                right: None,
                parent: None,
                key: key,
                value: value,
                height: 0,
            }), parent, cmp_node);
            assert!(node_to_insert.is_some());
            return (node_to_insert, None);
        }
        (None, Some((duplicate, key, value)))
    }

    #[inline]
    pub fn avl_add_element(&mut self, key: K, value: V) -> bool {
        unsafe {
            self.avl_add_element_with_duplicate(key, value).is_some()
        }
    }

    #[inline]
    pub unsafe fn avl_add_element_with_duplicate(&mut self, key: K, value: V) -> Option<(NonNull<Node<K, V>>, K, V)> {
        let (node_to_insert, res) = self.avl_node_find_link(key, value);
        match res {
            None => {
                self.avl_node_post_insert(node_to_insert);
                return None;
            }
            Some((duplicate, key, value)) => {
                return Some((duplicate.unwrap(), key, value));
            }
        }
    }

    pub fn avl_set_element(&mut self, key: K, value: V) -> Option<V> {
        unsafe {
            match self.avl_add_element_with_duplicate(key, value) {
                None => None,
                Some((mut ptr, _, mut v)) => {
                    mem::swap(&mut ptr.as_mut().value, &mut v);
                    Some(v)
                }
            }
        }
    }


    #[inline]
    pub unsafe fn avl_node_find(&mut self, what: &K) -> Option<NonNull<Node<K, V>>> {
        let mut node = self.root;
        let mut res_node = None;
        unsafe {
            while let Some(ptr) = node {
                match what.cmp(&ptr.as_ref().key) {
                    Ordering::Equal => {
                        res_node = node;
                        break;
                    }
                    Ordering::Less => {
                        node = ptr.as_ref().left;
                    }
                    Ordering::Greater => {
                        node = ptr.as_ref().right;
                    }
                }
            }
        }
        res_node
    }

    #[inline]
    fn avl_node_link(mut new_node: Box<Node<K, V>>, parent: Option<NonNull<Node<K, V>>>, cmp_node: &mut Option<NonNull<Node<K, V>>>) -> Option<NonNull<Node<K, V>>> {
        new_node.parent = parent;
        new_node.height = 0;
        new_node.left = None;
        new_node.right = None;
        *cmp_node = Some(Box::into_raw_non_null(new_node));
        *cmp_node
    }

    unsafe fn avl_left_height(node: NonNull<Node<K, V>>) -> i32 {
        if let Some(node_ptr) = node.as_ref().left {
            node_ptr.as_ref().height
        } else {
            0
        }
    }

    unsafe fn avl_right_height(node: NonNull<Node<K, V>>) -> i32 {
        if let Some(node_ptr) = node.as_ref().right {
            node_ptr.as_ref().height
        } else {
            0
        }
    }

    unsafe fn avl_node_post_insert(&mut self, mut ori_node: Option<NonNull<Node<K, V>>>) {
        ori_node.unwrap().as_mut().height = 1;
        ori_node = ori_node.unwrap().as_ref().parent;
        while let Some(mut node) = ori_node {
            let h0 = Tree::avl_left_height(node);
            let h1 = Tree::avl_right_height(node);
            let height = max(h1, h0) + 1;
            if node.as_ref().height == height {
                break;
            }
            node.as_mut().height = height;
            let diff = h0 - h1;
            if diff <= -2 {
                node = self._avl_node_fix_l(Some(node)).unwrap();
            } else if diff >= 2 {
                node = self._avl_node_fix_r(Some(node)).unwrap();
            }
            ori_node = (*node.as_ptr()).parent;
        }
    }

    unsafe fn _avl_node_fix_l(&mut self, mut node: Option<NonNull<Node<K, V>>>) -> Option<NonNull<Node<K, V>>> {
        let right = node.unwrap().as_ref().right;
        let rh0 = Tree::avl_left_height(right.unwrap());
        let rh1 = Tree::avl_right_height(right.unwrap());
        if rh0 > rh1 {
            let right = self._avl_node_rotate_right(right);
            Tree::_avl_node_height_update(right.unwrap().as_mut().right);
            Tree::_avl_node_height_update(right);
        }
        node = self._avl_node_rotate_left(node);
        Tree::_avl_node_height_update(node.unwrap().as_ref().left);
        Tree::_avl_node_height_update(node);
        node
    }

    unsafe fn _avl_node_fix_r(&mut self, mut node: Option<NonNull<Node<K, V>>>) -> Option<NonNull<Node<K, V>>> {
        let left = node.unwrap().as_ref().left;
        let rh0 = Tree::avl_left_height(left.unwrap());
        let rh1 = Tree::avl_right_height(left.unwrap());
        if rh0 < rh1 {
            let left = self._avl_node_rotate_left(left);
            Tree::_avl_node_height_update(left.unwrap().as_mut().left);
            Tree::_avl_node_height_update(left);
        }
        node = self._avl_node_rotate_right(node);
        Tree::_avl_node_height_update(node.unwrap().as_ref().right);
        Tree::_avl_node_height_update(node);
        node
    }

    unsafe fn _avl_node_height_update(node: Option<NonNull<Node<K, V>>>) {
        let mut node = node.unwrap();
        let h0 = unsafe { Tree::avl_left_height(node) };
        let h1 = unsafe { Tree::avl_right_height(node) };
        node.as_mut().height = max(h0, h1) + 1;
    }

    unsafe fn _avl_node_rotate_right(&mut self, node: Option<NonNull<Node<K, V>>>) -> Option<NonNull<Node<K, V>>> {
        let left = node.unwrap().as_mut().left;
        let parent = node.unwrap().as_mut().parent;
        node.unwrap().as_mut().left = left.unwrap().as_ref().right;
        if let Some(mut x) = left.unwrap().as_ref().right {
            x.as_mut().parent = node;
        }
        left.unwrap().as_mut().right = node;
        left.unwrap().as_mut().parent = parent;
        self._avl_child_replace(node, left, parent);
        node.unwrap().as_mut().parent = left;
        left
    }

    unsafe fn _avl_node_rotate_left(&mut self, node: Option<NonNull<Node<K, V>>>) -> Option<NonNull<Node<K, V>>> {
        let right = node.unwrap().as_mut().right;
        let parent = node.unwrap().as_mut().parent;
        node.unwrap().as_mut().right = right.unwrap().as_ref().left;
        if let Some(mut x) = right.unwrap().as_ref().left {
            x.as_mut().parent = node;
        }
        right.unwrap().as_mut().left = node;
        right.unwrap().as_mut().parent = parent;
        self._avl_child_replace(node, right, parent);
        node.unwrap().as_mut().parent = right;
        right
    }

    unsafe fn _avl_child_replace(&mut self, old_node: Option<NonNull<Node<K, V>>>, new_node: Option<NonNull<Node<K, V>>>, parent: Option<NonNull<Node<K, V>>>) {
        if let Some(mut p) = parent {
            if cmp_node_ptr(&p.as_ref().left, &old_node) {
                p.as_mut().left = new_node;
            } else {
                p.as_mut().right = new_node;
            }
        } else {
            self.root = new_node;
        }
    }

    fn drop_all(&mut self) {
        Tree::drop_node(self.root);
        self.root = None;
    }

    fn drop_node(root: Option<NonNull<Node<K, V>>>) {
        match root {
            Some(ptr) => {
                Tree::drop_node(unsafe { ptr.as_ref() }.left);
                Tree::drop_node(unsafe { ptr.as_ref() }.right);
                unsafe { Box::from_raw(ptr.as_ptr()); }
            }
            None => {}
        }
    }

    fn avl_bst_check(&self)-> bool{
        unsafe {
            let mut node = self.avl_node_first();
            if node.is_none() {
                return true;
            }
            let mut value = &((*node.unwrap().as_ptr()).key);
            node = Node::node_next(node);
            while node.is_some() {
                let x = &((*node.unwrap().as_ptr()).key);
                if x <= value {
                    return false;
                }
                value = x;
                node = Node::node_next(node);
            }
            return true;
        }
    }
}

impl<K, V> Drop for Tree<K, V> where K: Ord {
    fn drop(&mut self) {
        self.drop_all();
    }
}

mod test {
    extern crate rand;
    use avl::Tree;
    use std::cmp::Ordering;
    type DefaultType = Tree<i32, Option<char>>;

    #[test]
    fn test_avl_basic() {
        let mut t = DefaultType::new();
        unsafe {
            assert!(t.root.is_none());
            t.avl_add_element(3, None);
            assert_eq!(t.root.unwrap().as_ref().key, 3);
            assert_eq!(t.root.unwrap().as_ref().height, 1);
            assert!(t.root.unwrap().as_ref().left.is_none());
            assert!(t.root.unwrap().as_ref().right.is_none());

            t.avl_add_element(2, None);
            assert_eq!(t.root.unwrap().as_ref().key, 3);
            assert_eq!(t.root.unwrap().as_ref().height, 2);
            assert!(t.root.unwrap().as_ref().left.is_some());
            assert!(t.root.unwrap().as_ref().right.is_none());
            assert_eq!(t.root.unwrap().as_ref().left.unwrap().as_ref().parent.unwrap().as_ptr(), t.root.unwrap().as_ptr());
        }
    }


    #[test]
    fn test_avl_rotate_right() {
        let mut t = DefaultType::new();
        unsafe {
            t.avl_add_element(3, None);
            assert_eq!(t.root.unwrap().as_ref().key, 3);
            assert_eq!(t.root.unwrap().as_ref().height, 1);
            t.avl_add_element(2, None);
            assert_eq!(t.root.unwrap().as_ref().key, 3);
            assert_eq!(t.root.unwrap().as_ref().height, 2);
            t.avl_add_element(1, None);
            assert_eq!(t.root.unwrap().as_ref().key, 2);
            assert_eq!(t.root.unwrap().as_ref().height, 2);
        }
    }

    #[test]
    fn test_avl_rotate_left() {
        let mut t = DefaultType::new();
        unsafe {
            t.avl_add_element(1, None);
            assert_eq!(t.root.unwrap().as_ref().key, 1);
            assert_eq!(t.root.unwrap().as_ref().height, 1);
            t.avl_add_element(2, None);
            assert_eq!(t.root.unwrap().as_ref().key, 1);
            assert_eq!(t.root.unwrap().as_ref().height, 2);
            t.avl_add_element(3, None);
            assert_eq!(t.root.unwrap().as_ref().key, 2);
            assert_eq!(t.root.unwrap().as_ref().height, 2);
        }
    }

    #[test]
    fn test_avl_element_cmp() {
        #[derive(Eq)]
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

        let mut t = Tree::<MyData, Option<i32>>::new();
        unsafe {
            t.avl_add_element(MyData { a: 1 }, None);
            assert!(t.root.unwrap().as_ref().key == MyData { a: 1 });
            assert_eq!(t.root.unwrap().as_ref().height, 1);
            t.avl_add_element(MyData { a: 2 }, None);
            assert!(t.root.unwrap().as_ref().key == MyData { a: 1 });
            assert_eq!(t.root.unwrap().as_ref().height, 2);
        }
    }

    #[test]
    fn test_avl_duplicate() {
        let mut t = DefaultType::new();
        let mut is_duplicate = t.avl_add_element(1, None);
        assert!(!is_duplicate);
        assert!(unsafe { t.avl_node_find(&1) }.is_some());
        is_duplicate = t.avl_add_element(1, None);
        assert!(is_duplicate);
    }

    #[test]
    fn test_avl_find() {
        let mut t = DefaultType::new();
        assert!(unsafe { t.avl_node_find(&1) }.is_none());
        t.avl_add_element(1, None);
        assert_eq!(unsafe { t.avl_node_find(&1) }.unwrap().as_ptr(), t.root.unwrap().as_ptr());
    }

    #[test]
    fn test_avl_validate() {
        let mut v = vec![0; 1000];
        for idx in 0..v.len() {
            v[idx] = idx as i32;
            let pos = rand::random::<usize>() % (idx + 1);
            assert!(pos <= idx);
            assert!(pos >= 0);
            v.swap(idx, pos);
        }
        unsafe {
            let mut t = Tree::<i32, i32>::new();
            for d in v {
                t.avl_add_element(d, -d);
            }
            assert_eq!(t.root.unwrap().as_ref().height, 12);
            let left = t.root.unwrap().as_ref().left;
            assert!(left.unwrap().as_ref().height <= 11);
            assert!(left.unwrap().as_ref().height >= 10);
            let right = t.root.unwrap().as_ref().right;
            assert!(right.unwrap().as_ref().height <= 11);
            assert!(right.unwrap().as_ref().height >= 10);

            assert!(t.avl_bst_check());
        }
    }
}

