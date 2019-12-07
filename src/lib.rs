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
pub mod prelude {
    pub use crate::BufReadExt;
}

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

    /// Read all bytes into `buf` until the delimiter `byte` or EOF is reached.
    ///
    /// This function will read bytes from the underlying stream until the
    /// delimiter or EOF is found. Once found, all bytes up to, and including,
    /// the delimiter (if found) will be appended to `buf`.
    ///
    /// Unlike `read_until` after consuming bytes through this method you'll
    /// have to manually call [`BufRead::consume`].
    ///
    /// If successful, this function will return the total number of bytes read.
    ///
    /// # Errors
    ///
    /// This function will ignore all instances of [`ErrorKind::Interrupted`] and
    /// will otherwise return any errors returned by [`BufRead::fill_buf`].
    ///
    /// If an I/O error is encountered then all bytes read so far will be
    /// present in `buf` and its length will have been adjusted appropriately.
    ///
    /// [`BufRead::consume`]: https://doc.rust-lang.org/std/io/trait.BufRead.html#tymethod.consume
    /// [`BufRead::fill_buf`]: https://doc.rust-lang.org/std/io/trait.BufRead.html#tymethod.consume
    /// [`ErrorKind::Interrupted`]: https://doc.rust-lang.org/std/io/enum.ErrorKind.html#variant.Interrupted
    ///
    /// # Examples
    ///
    /// [`std::io::Cursor`][`Cursor`] is a type that implements `BufRead`. In
    /// this example, we use [`Cursor`] to read all the bytes in a byte slice
    /// in hyphen delimited segments:
    ///
    /// [`Cursor`]: https://doc.rust-lang.org/std/io/struct.Cursor.html
    ///
    /// ```
    /// use std::io::{self, BufRead};
    /// use omnom::prelude::*;
    ///
    /// let mut cursor = io::Cursor::new(b"lorem-ipsum");
    /// let mut buf = vec![];
    ///
    /// // cursor is at 'l'
    /// let num_bytes = cursor.try_read_until(b'-', &mut buf)
    ///     .expect("reading from cursor won't fail");
    /// assert_eq!(buf, b"lorem-");
    /// assert_eq!(num_bytes, 6);
    /// cursor.consume(num_bytes);
    /// buf.clear();
    ///
    /// // cursor is at 'i'
    /// let num_bytes = cursor.try_read_until(b'-', &mut buf)
    ///     .expect("reading from cursor won't fail");
    /// assert_eq!(buf, b"ipsum");
    /// assert_eq!(num_bytes, 5);
    /// cursor.consume(num_bytes);
    /// buf.clear();
    ///
    /// // cursor is at EOF
    /// let num_bytes = cursor.try_read_until(b'-', &mut buf)
    ///     .expect("reading from cursor won't fail");
    /// assert_eq!(num_bytes, 0);
    /// assert_eq!(buf, b"");
    /// ```
    fn try_read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut read = 0;
        loop {
            let available = match self.fill_buf() {
                Ok(n) => n,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };

            let available = &available[read..];

            let (done, used) = match memchr::memchr(byte, available) {
                Some(i) => {
                    buf.extend_from_slice(&available[..=i]);
                    (true, i + 1)
                }
                None => {
                    buf.extend_from_slice(available);
                    (false, available.len())
                }
            };

            read += used;
            if done || used == 0 {
                return Ok(read);
            }
        }
    }
}

impl<T: BufRead> BufReadExt for T {}
