extern crate num;
#[macro_use]
extern crate num_derive;

pub mod cpu;
pub mod mem;

pub use cpu::{tb_mem_read, tb_mem_write};
