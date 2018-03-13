use std::heap::Alloc;
use std::heap::Heap;
use std::heap::Layout;
use std::mem;

pub type VoidPtr = *mut u8;

const VOID_PTR_NULL: VoidPtr = 0 as VoidPtr;

pub struct Fastbin {
    obj_size: usize,
    page_size: usize,
    align: usize,
    maximum: usize,
    start: VoidPtr,
    end: VoidPtr,
    next: VoidPtr,
    pages: VoidPtr,
}

impl Default for Fastbin {
    fn default() -> Self {
        Fastbin {
            obj_size: 0,
            page_size: 0,
            align: 0,
            maximum: 1usize << 16,
            start: VOID_PTR_NULL,
            end: VOID_PTR_NULL,
            next: VOID_PTR_NULL,
            pages: VOID_PTR_NULL,
        }
    }
}

impl Fastbin {
    #[inline]
    pub fn new(obj_size: usize) -> Self {
        let mut fastbin = Default::default();
        (&mut fastbin as FastbinPtr).fastbin_init(obj_size);
        fastbin
    }

    #[inline]
    pub fn del(&mut self, ptr: VoidPtr) {
        (self as FastbinPtr).fastbin_del(ptr);
    }

    #[inline]
    pub fn alloc(&mut self) -> VoidPtr {
        unsafe { (self as FastbinPtr).fastbin_new() }
    }

    #[inline]
    pub fn destroy(&mut self) {
        (self as FastbinPtr).fastbin_destroy();
    }
}

pub type FastbinPtr = *mut Fastbin;

#[inline]
fn get_page_next(ptr: VoidPtr) -> VoidPtr {
    unsafe { *(ptr as *mut VoidPtr) }
}

#[inline]
fn set_page_next(ptr: VoidPtr, data: VoidPtr) {
    unsafe { *(ptr as *mut VoidPtr) = data }
}

#[inline]
fn get_page_size(ptr: VoidPtr) -> usize {
    unsafe { *(ptr.offset(mem::size_of::<VoidPtr>() as isize) as *mut usize) }
}

#[inline]
fn set_page_size(ptr: VoidPtr, size: usize) {
    unsafe { *(ptr.offset(mem::size_of::<VoidPtr>() as isize) as *mut usize) = size; }
}

trait FastbinPtrBase {
    fn start(self) -> VoidPtr;
    fn set_start(self, start: VoidPtr);
    fn end(self) -> VoidPtr;
    fn set_end(self, end: VoidPtr);
    fn next(self) -> VoidPtr;
    fn set_next(self, next: VoidPtr);
    fn pages(self) -> VoidPtr;
    fn set_pages(self, pages: VoidPtr);
    fn obj_size(self) -> usize;
    fn set_obj_size(self, obj_size: usize);
    fn page_size(self) -> usize;
    fn set_page_size(self, page_size: usize);
    fn maximum(self) -> usize;
    fn set_maximum(self, maximum: usize);
    fn align(self) -> usize;
    fn set_align(self, align: usize);
}

pub trait FastbinPtrOperation {
    fn fastbin_init(self, obj_size: usize);
    fn fastbin_destroy(self);
    unsafe fn fastbin_new(self) -> VoidPtr;
    fn fastbin_del(self, ptr: VoidPtr);
}

impl FastbinPtrOperation for *mut Fastbin {
    #[inline]
    fn fastbin_init(self, obj_size: usize) {
        let align = mem::align_of::<VoidPtr>();
        self.set_start(VOID_PTR_NULL);
        self.set_end(VOID_PTR_NULL);
        self.set_next(VOID_PTR_NULL);
        self.set_pages(VOID_PTR_NULL);
        self.set_obj_size(round_up_to_next(obj_size, align));
        let need = self.obj_size() * 32 + mem::size_of::<VoidPtr>() + 16;
        self.set_page_size(1usize << 5);
        while self.page_size() < need {
            self.set_page_size(self.page_size() * 2);
        }
        self.set_align(align);
    }

    #[inline]
    fn fastbin_destroy(self) {
        while !self.pages().is_null() {
            let page = self.pages();
            let next = get_page_next(page);
            let page_size = get_page_size(page);
            self.set_pages(next);
            unsafe {
                Heap.dealloc(self.pages(), Layout::from_size_align_unchecked(page_size, self.align()));
            }
        }
        self.set_start(VOID_PTR_NULL);
        self.set_end(VOID_PTR_NULL);
        self.set_next(VOID_PTR_NULL);
        self.set_pages(VOID_PTR_NULL);
    }

    #[inline]
    unsafe fn fastbin_new(self) -> VoidPtr {
        let obj_size = self.obj_size() as isize;
        let mut obj = self.next();
        if !obj.is_null() {
            self.set_next(get_page_next(self.next()));
            return obj;
        }
        if self.start().offset(obj_size) > self.end() {
            let page = Heap.alloc(Layout::from_size_align_unchecked(self.page_size(), self.align())).unwrap_or_else(|e| Heap.oom(e));
            let mut line_ptr = page;
            set_page_next(page, self.pages());
            set_page_size(page, self.page_size());
            self.set_pages(page);
            line_ptr = round_up_to_next(
                line_ptr as usize + mem::size_of::<VoidPtr>() + mem::size_of::<usize>(),
                16,
            ) as VoidPtr;
            self.set_start(line_ptr);
            self.set_end(page.offset(self.page_size() as isize));
            if self.page_size() < self.maximum() {
                self.set_page_size(self.page_size() * 2);
            }
        }
        obj = self.start();
        self.set_start(self.start().offset(obj_size));
        debug_assert!(self.start() <= self.end());
        obj
    }

    #[inline]
    fn fastbin_del(self, ptr: VoidPtr) {
        set_page_next(ptr, self.next());
        self.set_next(ptr);
    }
}

impl FastbinPtrBase for *mut Fastbin {
    #[inline]
    fn start(self) -> VoidPtr {
        unsafe { (*self).start }
    }

    #[inline]
    fn set_start(self, start: VoidPtr) {
        unsafe { (*self).start = start }
    }

    #[inline]
    fn end(self) -> VoidPtr {
        unsafe { (*self).end }
    }

    #[inline]
    fn set_end(self, end: VoidPtr) {
        unsafe { (*self).end = end }
    }

    #[inline]
    fn next(self) -> VoidPtr {
        unsafe { (*self).next }
    }

    #[inline]
    fn set_next(self, next: VoidPtr) {
        unsafe { (*self).next = next }
    }

    #[inline]
    fn pages(self) -> VoidPtr {
        unsafe { (*self).pages }
    }

    #[inline]
    fn set_pages(self, pages: VoidPtr) {
        unsafe { (*self).pages = pages }
    }

    #[inline]
    fn obj_size(self) -> usize {
        unsafe { (*self).obj_size }
    }

    #[inline]
    fn set_obj_size(self, obj_size: usize) {
        unsafe { (*self).obj_size = obj_size }
    }

    #[inline]
    fn page_size(self) -> usize {
        unsafe { (*self).page_size }
    }

    #[inline]
    fn set_page_size(self, page_size: usize) {
        unsafe { (*self).page_size = page_size }
    }

    #[inline]
    fn maximum(self) -> usize {
        unsafe { (*self).maximum }
    }

    #[inline]
    fn set_maximum(self, maximum: usize) {
        unsafe { (*self).maximum = maximum }
    }

    #[inline]
    fn align(self) -> usize {
        unsafe { (*self).align }
    }

    #[inline]
    fn set_align(self, align: usize) {
        unsafe { (*self).align = align }
    }
}

#[inline]
fn round_up_to_next(unrounded: usize, target_alignment: usize) -> usize {
    (unrounded + target_alignment - 1) & !(target_alignment - 1)
}

mod test {
    use fastbin;
    use fastbin::Fastbin;
    use std::mem;
    use fastbin::VoidPtr;

    #[test]
    fn test_fastbin_init() {
        struct Node {
            a: u8,
            b: u64,
            c: u8,
            e: u64,
            d: u8,
        }
        let fb = Fastbin::new(mem::size_of::<Node>());
        assert_eq!(fb.align, mem::align_of::<VoidPtr>());
        assert_eq!(fb.obj_size, 24);
        assert_eq!(fb.page_size, 1024);
        assert_eq!(fb.maximum, (1usize << 16));
    }

    #[test]
    fn test_fastbin_new() {
        struct Node {
            a: u8,
        }
        let mut fb = Fastbin::new(mem::size_of::<Node>());
        fb.alloc() as *mut Node;
        assert!(!fb.pages.is_null());
        let page = fb.pages;
        for _ in 0..60 {
            fb.alloc() as *mut Node;
        }
        assert_eq!(fb.pages, page);
        for _ in 0..150 {
            fb.alloc() as *mut Node;
        }
        assert_ne!(fb.pages, page);
    }

    #[test]
    fn test_fastbin_del() {
        struct Node {
            a: u8,
        }
        let mut fb = Fastbin::new(mem::size_of::<Node>());
        for _ in 0..3 { fb.alloc() as *mut Node; }
        let a = fb.alloc();
        for _ in 0..3 { fb.alloc() as *mut Node; }
        let b = fb.alloc();
        for _ in 0..3 { fb.alloc() as *mut Node; }
        let c = fb.alloc();
        assert!(fb.next.is_null());
        fb.del(a);
        assert!(fastbin::get_page_next(a).is_null());
        assert_eq!(fb.next, a);
        fb.del(b);
        assert_eq!(fastbin::get_page_next(b), a);
        assert_eq!(fb.next, b);
        fb.del(c);
        assert_eq!(fastbin::get_page_next(c), b);
        assert_eq!(fb.next, c);
    }

    #[test]
    fn test_fastbin_destroy() {
        struct Node { a: u8 }
        let mut fb = Fastbin::new(mem::size_of::<Node>());
        for _ in 0..150 {
            fb.alloc();
        }
        let mut v = Vec::new();
        let mut page = fb.pages;
        while !page.is_null() {
            let next = fastbin::get_page_next(page);
            let page_size = fastbin::get_page_size(page);
            v.push(page_size);
            page = next;
        }
        assert_eq!(v[0], v[1] * 2);

    }
}