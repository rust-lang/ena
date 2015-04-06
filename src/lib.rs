#![feature(std_misc)]
#![crate_name="rusty-cc"]

#[macro_use]
mod debug;

mod constraint;
mod graph;
mod snapshot_vec;
mod cc;
mod unify;
mod bitvec;
