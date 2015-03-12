#![feature(std_misc)]

#[macro_use]
extern crate log;

mod graph;
mod snapshot_vec;
mod cc;
mod unify;
mod bitvec;

#[cfg(test)] mod test;
#[cfg(test)] mod cc_test;
