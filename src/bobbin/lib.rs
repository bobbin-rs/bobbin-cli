#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! { 
    foreign_links {
    }
}