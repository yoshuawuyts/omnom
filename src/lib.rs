//! Smaller, sillier version of the Nom parser.
//!
//! This is a one-off experiment to see if we can extend the `Read` and `Write`
//! traits with better parsing capabilities. The name is a riff on the
//! [`nom`](https://docs.rs/nom) parser, which you should probably check out.
//!
//! # Examples
//!
//! ```
//! use omnom::prelude::*;
//! ```

#![forbid(unsafe_code, future_incompatible, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]

use std::io::{self, BufRead, ErrorKind, Read};
use std::slice;

/// The `omnom` prelude.
pub mod prelude {}

/// Extend `BufRead` with methods for streaming parsing.
pub trait BufReadExt: BufRead {
    /// Read bytes based on a predicate.
    ///
    /// `read_while` takes a predicate as an argument.
    /// It will call this on each byte, and copy it to the slice if the
    /// predicate evaluates to `true`. Returns the amount of bytes read.
    ///
    /// # Errors
    ///
    /// If this function encounters an error of the kind
    /// `ErrorKind::Interrupted` then the error is ignored and the operation
    /// will continue.
    ///
    /// If any other read error is encountered then this function immediately
    /// returns. Any bytes which have already been read will be appended to
    /// `buf`.
    fn read_while<P>(&mut self, buf: &mut [u8], mut predicate: P) -> io::Result<usize>
    where
        P: FnMut(u8) -> bool,
    {
        let mut read = 0;

        while read < buf.len() {
            let mut byte = 0;

            match self.read(slice::from_mut(&mut byte)) {
                Ok(0) => break,
                Ok(_) => {
                    read += 1;
                    if predicate(byte) {
                        buf[read] = byte;
                    } else {
                        break;
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => {
                    self.consume(read);
                    return Err(e);
                }
            };
        }

        self.consume(read);
        Ok(read)
    }

    /// Try reading based on a predicate.
    ///
    /// `read_while` takes a predicate as an argument.
    /// It will call this on each byte, and copy it to the slice if the
    /// predicate evaluates to `true`. Returns the amount of bytes read.
    ///
    /// Unlike `read_while` after consuming bytes through this method you'll
    /// have to manually call [`BufRead::consume`](https://doc.rust-lang.org/std/io/trait.BufRead.html#tymethod.consume).
    ///
    /// # Errors
    ///
    /// If this function encounters an error of the kind
    /// `ErrorKind::Interrupted` then the error is ignored and the operation
    /// will continue.
    ///
    /// If any other read error is encountered then this function immediately
    /// returns. Any bytes which have already been read will be appended to
    /// `buf`.
    fn try_read_while<P>(&mut self, buf: &mut [u8], mut predicate: P) -> io::Result<usize>
    where
        Self: Read,
        P: FnMut(u8) -> bool,
    {
        let mut read = 0;

        while read < buf.len() {
            let mut byte = 0;

            match self.read(slice::from_mut(&mut byte)) {
                Ok(0) => break,
                Ok(_) => {
                    read += 1;
                    if predicate(byte) {
                        buf[read] = byte;
                    } else {
                        break;
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
        }
        Ok(read)
    }
}

impl<T: BufRead> BufReadExt for T {}
