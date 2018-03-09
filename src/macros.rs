macro_rules! avl_offset {($TYPE: ty, $MEMBER: ident) => {&(*(0 as *const $TYPE)).$MEMBER as *const _ as isize}}
macro_rules! avl_entry {($PTR: expr, $TYPE: ty, $MEMBER: ident) => {($PTR as *const _ as isize - unsafe { avl_offset!($TYPE, $MEMBER)}) as *mut $TYPE}}
macro_rules! list_offset_of {($TYPE: ty, $MEMBER: ident) => {&(*(0 as *const $TYPE)).$MEMBER as *const _ as isize}}
macro_rules! list_container_of {($PTR: expr, $TYPE: ty, $MEMBER: ident) => {($PTR as *const _ as isize - unsafe {list_offset_of!($TYPE, $MEMBER)}) as *mut $TYPE}}
macro_rules! list_entry {($PTR: expr, $TYPE: ty, $MEMBER: ident) => {list_container_of!($PTR, $TYPE, $MEMBER)}}