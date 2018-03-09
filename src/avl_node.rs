use std::ptr;
use std::cmp::{max};

pub struct AVLNode {
    left: AVLNodePtr,
    right: AVLNodePtr,
    parent: AVLNodePtr,
    height: i32,
}

impl AVLNode {
    #[inline]
    pub fn new() -> AVLNode {
        AVLNode {
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            parent: ptr::null_mut(),
            height: 0,
        }
    }
}

pub type AVLNodePtr = *mut AVLNode;

pub trait AVLNodePtrBase {
    fn isomorphic(self, node: AVLNodePtr) -> bool;
    fn height_update(self);
    fn height(self) -> i32;
    fn next(self) -> AVLNodePtr;
    fn prev(self) -> AVLNodePtr;
    fn set_parent(self, parent: AVLNodePtr);
    fn set_left(self, left: AVLNodePtr);
    fn set_right(self, right: AVLNodePtr);
    fn parent(self) -> AVLNodePtr;
    fn left(self) -> AVLNodePtr;
    fn right(self) -> AVLNodePtr;
    fn left_mut(self) -> *mut AVLNodePtr;
    fn right_mut(self) -> *mut AVLNodePtr;
    fn null() -> AVLNodePtr;
    fn not_null(self) -> bool;
    fn set_height(self, height: i32);
    fn left_height(self) -> i32;
    fn right_height(self) -> i32;
    fn first_node(self) -> AVLNodePtr;
    fn last_node(self) -> AVLNodePtr;
}

impl AVLNodePtrBase for AVLNodePtr {
    fn isomorphic(self, node: AVLNodePtr) -> bool {
        if self.is_null() && node.is_null() {
            return true;
        }
        if self.is_null() || node.is_null() {
            return false;
        }
        if self.height() != node.height() {
            return false;
        }
        self.left().isomorphic(node.left()) && self.right().isomorphic(node.right())
    }

    #[inline]
    fn height_update(self) {
        self.set_height(max(self.left_height(), self.right_height()) + 1);
    }

    #[inline]
    fn height(self) -> i32 {
        if self.is_null() {
            return 0;
        }
        unsafe { (*self).height }
    }

    #[inline]
    fn next(self) -> AVLNodePtr {
        if self.is_null() {
            return AVLNodePtr::null();
        }
        let mut node = self;
        if self.right().not_null() {
            node = node.right();
            while node.left().not_null() {
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
    fn prev(self) -> AVLNodePtr {
        if self.is_null() {
            return AVLNodePtr::null();
        }
        let mut node = self;
        if node.left().not_null() {
            node = node.left();
            while node.right().not_null() {
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
    fn set_parent(self, parent: AVLNodePtr) {
        unsafe { (*self).parent = parent }
    }

    #[inline]
    fn set_left(self, left: AVLNodePtr) {
        unsafe { (*self).left = left }
    }

    #[inline]
    fn set_right(self, right: AVLNodePtr) {
        unsafe { (*self).right = right }
    }

    #[inline]
    fn parent(self) -> AVLNodePtr {
        unsafe { (*self).parent }
    }

    #[inline]
    fn left(self) -> AVLNodePtr {
        unsafe { (*self).left }
    }

    #[inline]
    fn right(self) -> AVLNodePtr {
        unsafe { (*self).right }
    }

    #[inline]
    fn left_mut(self) -> *mut AVLNodePtr {
        unsafe { &mut (*self).left }
    }

    #[inline]
    fn right_mut(self) -> *mut AVLNodePtr {
        unsafe { &mut (*self).right }
    }

    #[inline]
    fn null() -> AVLNodePtr {
        ptr::null_mut()
    }

    #[inline]
    fn not_null(self) -> bool {
        !self.is_null()
    }

    #[inline]
    fn set_height(self, height: i32) {
        unsafe { (*self).height = height; }
    }

    #[inline]
    fn left_height(self) -> i32 {
        self.left().height()
    }

    #[inline]
    fn right_height(self) -> i32 {
        self.right().height()
    }

    #[inline]
    fn first_node(self) -> AVLNodePtr {
        let mut ptr = self;
        if ptr.is_null() {
            return AVLNodePtr::null();
        }
        while ptr.left().not_null() {
            ptr = ptr.left();
        }
        ptr
    }

    #[inline]
    fn last_node(self) -> AVLNodePtr {
        let mut ptr = self;
        if ptr.is_null() {
            return AVLNodePtr::null();
        }
        while ptr.right().not_null() {
            ptr = ptr.right();
        }
        ptr
    }
}
