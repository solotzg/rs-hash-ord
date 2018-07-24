use std::ptr;
use std::cmp::max;

pub struct AVLNode {
    pub left: AVLNodePtr,
    pub right: AVLNodePtr,
    pub parent: AVLNodePtr,
    pub height: i32,
}

#[derive(Copy, Clone)]
pub struct AVLRoot {
    pub node: AVLNodePtr,
}

impl Default for AVLRoot {
    fn default() -> Self {
        AVLRoot {
            node: ptr::null_mut(),
        }
    }
}

pub type AVLRootPtr = *mut AVLRoot;

impl Default for AVLNode {
    fn default() -> Self {
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
    fn not_null(self) -> bool;
    fn set_height(self, height: i32);
    fn left_height(self) -> i32;
    fn right_height(self) -> i32;
    fn first_node(self) -> AVLNodePtr;
    fn last_node(self) -> AVLNodePtr;
    fn init(self);
    fn empty(self) -> bool;
    fn reset(self, left: AVLNodePtr, right: AVLNodePtr, parent: AVLNodePtr, height: i32);
    fn check_valid(self) -> bool;
    fn get_node_num(self) -> i32;
}

impl AVLNodePtrBase for *mut AVLNode {
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
            return ptr::null_mut();
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
            return ptr::null_mut();
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
    fn not_null(self) -> bool {
        !self.is_null()
    }

    #[inline]
    fn set_height(self, height: i32) {
        unsafe {
            (*self).height = height;
        }
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
            return ptr::null_mut();
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
            return ptr::null_mut();
        }
        while ptr.right().not_null() {
            ptr = ptr.right();
        }
        ptr
    }

    #[inline]
    fn init(self) {
        self.set_parent(self)
    }

    #[inline]
    fn empty(self) -> bool {
        self.parent() == self
    }

    #[inline]
    fn reset(self, left: AVLNodePtr, right: AVLNodePtr, parent: AVLNodePtr, height: i32) {
        self.set_left(left);
        self.set_right(right);
        self.set_parent(parent);
        self.set_height(height);
    }

    fn check_valid(self) -> bool {
        use std::cmp;
        if self.is_null() {
            return true;
        }
        let h0 = self.left_height();
        let h1 = self.right_height();
        let diff = h0 - h1;
        if self.height() != cmp::max(h0, h1) + 1 {
            return false;
        }
        if diff < -1 || diff > 1 {
            return false;
        }
        self.left().check_valid() && self.right().check_valid()
    }

    fn get_node_num(self) -> i32 {
        if self.is_null() {
            return 0;
        }
        self.left().get_node_num() + self.right().get_node_num() + 1
    }
}

#[inline]
pub unsafe fn erase_node(mut node: AVLNodePtr, root: AVLRootPtr) {
    if node.is_null() {
        return;
    }
    let parent = if node.left().not_null() && node.right().not_null() {
        let old = node;
        node = node.right();
        while node.left().not_null() {
            node = node.left();
        }
        let child = node.right();
        let mut parent = node.parent();
        if child.not_null() {
            child.set_parent(parent);
        }
        child_replace(node, child, parent, root);
        if node.parent() == old {
            parent = node;
        }
        node.set_left(old.left());
        node.set_right(old.right());
        node.set_parent(old.parent());
        node.set_height(old.height());
        child_replace(old, node, old.parent(), root);
        old.left().set_parent(node);
        if old.right().not_null() {
            old.right().set_parent(node);
        }
        parent
    } else {
        let child = if node.left().is_null() {
            node.right()
        } else {
            node.left()
        };
        let parent = node.parent();
        child_replace(node, child, parent, root);
        if child.not_null() {
            child.set_parent(parent);
        }
        parent
    };
    if parent.not_null() {
        rebalance_node(parent, root);
    }
}

#[inline]
unsafe fn child_replace(
    old_node: AVLNodePtr,
    new_node: AVLNodePtr,
    parent: AVLNodePtr,
    root: AVLRootPtr,
) {
    if parent.is_null() {
        (*root).node = new_node;
    } else {
        if parent.left() == old_node {
            parent.set_left(new_node);
        } else {
            parent.set_right(new_node);
        }
    }
}

#[inline]
unsafe fn rebalance_node(mut node: AVLNodePtr, root: AVLRootPtr) {
    while node.not_null() {
        let h0 = node.left_height();
        let h1 = node.right_height();
        let diff = h0 - h1;
        let height = max(h0, h1) + 1;
        if node.height() != height {
            node.set_height(height);
        } else if diff >= -1 && diff <= 1 {
            break;
        }
        if diff <= -2 {
            node = node_fix_l(node, root);
        } else if diff >= 2 {
            node = node_fix_r(node, root);
        }
        node = node.parent();
    }
}

#[inline]
unsafe fn node_fix_l(mut node: AVLNodePtr, root: AVLRootPtr) -> AVLNodePtr {
    let right = node.right();
    let rh0 = right.left_height();
    let rh1 = right.right_height();
    if rh0 > rh1 {
        let right = node_rotate_right(right, root);
        right.right().height_update();
        right.height_update();
    }
    node = node_rotate_left(node, root);
    node.left().height_update();
    node.height_update();
    node
}

#[inline]
pub unsafe fn node_fix_r(mut node: AVLNodePtr, root: AVLRootPtr) -> AVLNodePtr {
    let left = node.left();
    let rh0 = left.left_height();
    let rh1 = left.right_height();
    if rh0 < rh1 {
        let left = node_rotate_left(left, root);
        left.left().height_update();
        left.height_update();
    }
    node = node_rotate_right(node, root);
    node.right().height_update();
    node.height_update();
    node
}

#[inline]
pub unsafe fn node_rotate_right(node: AVLNodePtr, root: AVLRootPtr) -> AVLNodePtr {
    let left = node.left();
    let parent = node.parent();
    node.set_left(left.right());
    if left.right().not_null() {
        left.right().set_parent(node);
    }
    left.set_right(node);
    left.set_parent(parent);
    child_replace(node, left, parent, root);
    node.set_parent(left);
    left
}

#[inline]
pub unsafe fn node_rotate_left(node: AVLNodePtr, root: AVLRootPtr) -> AVLNodePtr {
    let right = node.right();
    let parent = node.parent();
    node.set_right(right.left());
    if right.left().not_null() {
        right.left().set_parent(node);
    }
    right.set_left(node);
    right.set_parent(parent);
    child_replace(node, right, parent, root);
    node.set_parent(right);
    right
}

#[inline]
pub unsafe fn link_node(new_node: AVLNodePtr, parent: AVLNodePtr, link_node: *mut AVLNodePtr) {
    new_node.set_parent(parent);
    new_node.set_height(0);
    new_node.set_left(ptr::null_mut());
    new_node.set_right(ptr::null_mut());
    *link_node = new_node;
}

#[inline]
pub unsafe fn node_post_insert(mut node: AVLNodePtr, root: AVLRootPtr) {
    node.set_height(1);
    node = node.parent();
    while node.not_null() {
        let h0 = node.left_height();
        let h1 = node.right_height();
        let height = max(h1, h0) + 1;
        let diff = h0 - h1;
        if node.height() == height {
            break;
        }
        node.set_height(height);
        if diff <= -2 {
            node = node_fix_l(node, root);
        } else if diff >= 2 {
            node = node_fix_r(node, root);
        }
        node = node.parent();
    }
}

#[inline]
pub unsafe fn avl_node_replace(tar: AVLNodePtr, new_node: AVLNodePtr, root: AVLRootPtr) {
    let parent = tar.parent();
    child_replace(tar, new_node, parent, root);
    if tar.left().not_null() {
        tar.left().set_parent(new_node);
    }
    if tar.right().not_null() {
        tar.right().set_parent(new_node);
    }
    new_node.set_left(tar.left());
    new_node.set_right(tar.right());
    new_node.set_parent(tar.parent());
    new_node.set_height(tar.height());
}

#[inline]
pub unsafe fn avl_node_tear(root: &mut AVLRoot, next: *mut AVLNodePtr) -> AVLNodePtr {
    let mut node = *next;
    if node.is_null() {
        if root.node.is_null() {
            return ptr::null_mut();
        }
        node = root.node;
    }
    loop {
        if node.left().not_null() {
            node = node.left();
        } else if node.right().not_null() {
            node = node.right();
        } else {
            break;
        }
    }
    let parent = node.parent();
    *next = parent;
    if parent.not_null() {
        if parent.left() == node {
            parent.set_left(ptr::null_mut());
        } else {
            parent.set_right(ptr::null_mut())
        }
        node.set_height(0);
    } else {
        root.node = ptr::null_mut();
    }
    node
}

/// convert AVL to list
/// left become prev
/// right become next
pub unsafe fn avl_tree_convert_to_list(root: &mut AVLRoot) -> AVLNodePtr {
    if root.node.is_null() {
        return ptr::null_mut();
    }
    let mut node = root.node.first_node();
    let head = node;
    root.node = ptr::null_mut();
    while node.not_null() {
        let next = node.next();
        node.set_right(next);
        node = next;
    }
    head
}
