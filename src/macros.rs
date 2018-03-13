macro_rules! offset_of {($TYPE: ty, $MEMBER: ident) => {&(*(0 as *const $TYPE)).$MEMBER as *const _ as isize}}
macro_rules! container_of {($PTR: expr, $TYPE: ty, $MEMBER: ident) => {($PTR as *const _ as isize - unsafe {offset_of!($TYPE, $MEMBER)}) as *mut $TYPE}}
macro_rules! entry_of {($PTR: expr, $TYPE: ty, $MEMBER: ident) => {container_of!($PTR, $TYPE, $MEMBER)}}