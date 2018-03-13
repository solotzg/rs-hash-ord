use std::ptr;

pub type ListHeadPtr = *mut ListHead;

#[derive(Copy, Clone)]
pub struct ListHead {
    pub next: ListHeadPtr,
    pub prev: ListHeadPtr,
}

impl Default for ListHead {
    fn default() -> Self {
        ListHead{ next: ptr::null_mut(), prev: ptr::null_mut()}
    }
}

impl ListHead {
    #[inline]
    pub fn is_eq_ptr(&self, ptr: ListHeadPtr) -> bool {
        (self as *const ListHead as isize) == (ptr as isize)
    }
}

pub trait ListHeadPtrFn {
    fn list_init(self);
    fn next(self) -> ListHeadPtr;
    fn prev(self) -> ListHeadPtr;
    fn set_next(self, next: ListHeadPtr);
    fn set_prev(self, prev: ListHeadPtr);
    fn list_add(self, node: ListHeadPtr);
    fn list_add_tail(self, node: ListHeadPtr);
    fn list_del(self);
    fn list_del_init(self);
    fn list_is_empty(self) -> bool;
    fn list_replace(old_node: ListHeadPtr, new_node: ListHeadPtr);
}

impl ListHeadPtrFn for *mut ListHead {
    #[inline]
    fn list_init(self) {
        self.set_next(self);
        self.set_prev(self);
    }

    #[inline]
    fn next(self) -> ListHeadPtr {
        unsafe {(*self).next}
    }

    #[inline]
    fn prev(self) -> ListHeadPtr {
        unsafe {(*self).prev}
    }

    #[inline]
    fn set_next(self, next: ListHeadPtr) {
        unsafe {(*self).next = next;}
    }

    #[inline]
    fn set_prev(self, prev: ListHeadPtr) {
        unsafe {(*self).prev = prev;}
    }

    #[inline]
    fn list_add(self, node: ListHeadPtr) {
        node.set_prev(self);
        node.set_next(self.next());
        self.next().set_prev(node);
        self.set_next(node);
    }

    #[inline]
    fn list_add_tail(self, node: ListHeadPtr) {
        node.set_prev(self.prev());
        node.set_next(self);
        self.prev().set_next(node);
        self.set_prev(node);
    }

    #[inline]
    fn list_del(self) {
        self.next().set_prev(self.prev());
        self.prev().set_next(self.next());
        self.set_next(ptr::null_mut());
        self.set_prev(ptr::null_mut());
    }

    #[inline]
    fn list_del_init(self) {
        self.list_del();
        self.list_init();
    }

    #[inline]
    fn list_is_empty(self) -> bool {
        self.next() == self
    }
    fn list_replace(old_node: ListHeadPtr, new_node: ListHeadPtr) {
        new_node.set_next(old_node.next());
        new_node.next().set_prev(new_node);
        new_node.set_prev(old_node.prev());
        new_node.prev().set_next(new_node);
    }
}
