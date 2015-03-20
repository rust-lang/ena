#![feature(std_misc)]
#![crate_name="rusty-cc"]

#[macro_use]
mod debug;

mod graph;
mod snapshot_vec;
mod cc;
mod unify;
mod bitvec;

#[cfg(test)] mod test;
#[cfg(test)] mod cc_test;
