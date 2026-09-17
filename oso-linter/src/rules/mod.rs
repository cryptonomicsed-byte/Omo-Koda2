//! Rule registry — each module implements a `check` function that
//! appends to the shared warnings/errors vecs.

pub mod semantic;
pub mod security;
pub mod resource;
