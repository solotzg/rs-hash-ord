#![feature(allocator_api)]
#![allow(dead_code)]
#![feature(collections_range)]
#![feature(try_reserve)]

#[macro_use]
mod macros;
pub mod ord_map;
mod hash_table;
pub mod hash_map;
mod avl_node;
mod list;
mod fastbin;
