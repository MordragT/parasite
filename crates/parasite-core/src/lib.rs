#![feature(slice_group_by)]
#![feature(let_chains)]
#![feature(associated_type_defaults)]

pub mod builder;
pub mod combinators;
pub mod first;
pub mod follow;
pub mod grammar;
pub mod parser;
pub mod table;

#[cfg(feature = "default")]
pub mod chumsky;
