//! Streaming parser extensions for `BufRead`.
//!
//! This is a one-off experiment to see if we can extend the `std::io::BufRead`
//! traits with better parsing capabilities. The name is a riff on the
//! [`nom`](https://docs.rs/nom) parser, which you should probably check out.
//!
//! # Why?
//!
//! The purpose of this crate is to make authoring streaming parsers easier. And
//! the way we do this is by providing more operations that decouple
//! "looking at bytes" from "consuming bytes". So hence we introduce `fill_`
//! counterparts for `read_until` and `read_exact`. And two new methods:
//! `read_while` and `fill_while` that read bytes into a buffer based on a
//! predicate.
//!
//! Together this should make it easier to parse bytes from streams.
//!
//! # Methods
//!
//! Methods prefixed with `fill_` don't [`consume`] bytes. This means that in order to move the
//! `BufRead` cursor forward, the `consume` method needs to be called. This means the same bytes
//! can be read multiple times.
//!
//! Methods prefixed with `read_` *do* `consume` bytes. This means when this method is called, the
//! cursor moves forward, which means the same bytes *cannot* be read multiple times.
//!
//! - [`BufReadExt::fill_exact`] reads bytes until a buffer has been filled, doesn't consume bytes.
//! - [`BufReadExt::fill_until`] reads bytes until a byte has been encountered, doesn't consume bytes.
//! - [`BufReadExt::fill_while`] reads bytes based on a predicate, doesn't consume bytes.
//! - [`BufReadExt::read_while`] reads bytes based on a predicate, consumes bytes.
//!
//! [`consume`]: https://doc.rust-lang.org/std/io/trait.BufRead.html#tymethod.consume
//! [`BufReadExt::fill_exact`]: trait.BufReadExt.html#method.fill_exact
//! [`BufReadExt::fill_until`]: trait.BufReadExt.html#method.fill_until
//! [`BufReadExt::fill_while`]: trait.BufReadExt.html#method.fill_while
//! [`BufReadExt::read_while`]: trait.BufReadExt.html#method.read_while
//!
//! # Todos
//!
//! - `AsyncRead` support
//! - [byte-ordered reads/writes](https://github.com/async-rs/async-std/issues/578)
//!
//! # Examples
//!
//! ```
//! use std::io::{self, BufRead};
//! use omnom::prelude::*;
//!
//! let mut cursor = io::Cursor::new(b"lorem-ipsum");
//! let mut buf = vec![];
//!
//! // cursor is at 'l'
//! let num_bytes = cursor.fill_until(b'-', &mut buf)
//!     .expect("reading from cursor won't fail");
//! assert_eq!(buf, b"lorem-");
//! assert_eq!(num_bytes, 6);
//! cursor.consume(num_bytes);
//! buf.clear();
//!
//! // cursor is at 'i'
//! let num_bytes = cursor.fill_until(b'-', &mut buf)
//!     .expect("reading from cursor won't fail");
//! assert_eq!(buf, b"ipsum");
//! assert_eq!(num_bytes, 5);
//! cursor.consume(num_bytes);
//! buf.clear();
//!
//! // cursor is at EOF
//! let num_bytes = cursor.fill_until(b'-', &mut buf)
//!     .expect("reading from cursor won't fail");
//! assert_eq!(num_bytes, 0);
//! assert_eq!(buf, b"");
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
    ///
    /// # Examples
    ///
    /// [`std::io::Cursor`][`Cursor`] is a type that implements `BufRead`. In
    /// this example, we use [`Cursor`] to read bytes in a byte slice until
    /// we encounter a hyphen:
    ///
    /// ```
    /// use std::io::{self, BufRead};
    /// use omnom::prelude::*;
    ///
    /// let mut cursor = io::Cursor::new(b"lorem-ipsum");
    /// let mut buf = vec![];
    ///
    /// let num_bytes = cursor.read_while(&mut buf, |b| b != b'-')
    ///     .expect("reading from cursor won't fail");
    /// assert_eq!(buf, b"lorem");
    /// ```
    fn read_while<P>(&mut self, buf: &mut Vec<u8>, mut predicate: P) -> io::Result<usize>
    where
        P: FnMut(u8) -> bool,
    {
        let mut read = 0;

        'outer: loop {
            let available = match self.fill_buf() {
                Ok(n) => n,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => {
                    self.consume(read);
                    return Err(e);
                },
            };

            if available.len() == 0 {
                break;
            }

            for byte in available.bytes() {
                let byte = byte?;
                if predicate(byte) {
                    buf.extend_from_slice(&[byte]);
                    read += 1;
                } else {
                    break 'outer;
                }
            }
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
    ///
    /// # Examples
    ///
    /// [`std::io::Cursor`][`Cursor`] is a type that implements `BufRead`. In
    /// this example, we use [`Cursor`] to read bytes in a byte slice until
    /// we encounter a hyphen:
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
    /// let num_bytes = cursor.fill_while(&mut buf, |b| b != b'-')
    ///     .expect("reading from cursor won't fail");
    /// assert_eq!(buf, b"lorem");
    /// cursor.consume(num_bytes);
    /// ```
    fn fill_while<P>(&mut self, buf: &mut Vec<u8>, mut predicate: P) -> io::Result<usize>
    where
        Self: Read,
        P: FnMut(u8) -> bool,
    {
        let mut read = 0;

        loop {
            let mut byte = 0;

            match self.read(slice::from_mut(&mut byte)) {
                Ok(0) => break,
                Ok(_) => {
                    if predicate(byte) {
                        buf.extend_from_slice(&[byte]);
                        read += 1;
                    } else {
                        read += 1;
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
    /// let num_bytes = cursor.fill_until(b'-', &mut buf)
    ///     .expect("reading from cursor won't fail");
    /// assert_eq!(buf, b"lorem-");
    /// assert_eq!(num_bytes, 6);
    /// cursor.consume(num_bytes);
    /// buf.clear();
    ///
    /// // cursor is at 'i'
    /// let num_bytes = cursor.fill_until(b'-', &mut buf)
    ///     .expect("reading from cursor won't fail");
    /// assert_eq!(buf, b"ipsum");
    /// assert_eq!(num_bytes, 5);
    /// cursor.consume(num_bytes);
    /// buf.clear();
    ///
    /// // cursor is at EOF
    /// let num_bytes = cursor.fill_until(b'-', &mut buf)
    ///     .expect("reading from cursor won't fail");
    /// assert_eq!(num_bytes, 0);
    /// assert_eq!(buf, b"");
    /// ```
    fn fill_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
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

    /// Read the exact number of bytes required to fill `buf`.
    ///
    /// This function reads as many bytes as necessary to completely fill the
    /// specified buffer `buf`.
    ///
    /// Unlike `read_exact`, after reading bytes through this method you'll
    /// have to manually call [`BufRead::consume`].
    ///
    /// No guarantees are provided about the contents of `buf` when this
    /// function is called, implementations cannot rely on any property of the
    /// contents of `buf` being true. It is recommended that implementations
    /// only write data to `buf` instead of reading its contents.
    ///
    /// # Errors
    ///
    /// If this function encounters an error of the kind
    /// [`ErrorKind::Interrupted`] then the error is ignored and the operation
    /// will continue.
    ///
    /// If this function encounters an "end of file" before completely filling
    /// the buffer, it returns an error of the kind [`ErrorKind::UnexpectedEof`].
    /// The contents of `buf` are unspecified in this case.
    ///
    /// If any other read error is encountered then this function immediately
    /// returns. The contents of `buf` are unspecified in this case.
    ///
    /// If this function returns an error, it is unspecified how many bytes it
    /// has read, but it will never read more than would be necessary to
    /// completely fill the buffer.
    ///
    /// # Examples
    ///
    /// [`File`]s implement `Read`:
    ///
    /// [`File`]: https://doc.rust-lang.org/std/fs/struct.File.html
    /// [`ErrorKind::Interrupted`]: https://doc.rust-lang.org/std/io/enum.ErrorKind.html#variant.Interrupted
    /// [`ErrorKind::UnexpectedEof`]: https://doc.rust-lang.org/std/io/enum.ErrorKind.html#variant.UnexpectedEof
    /// [`ErrorKind::UnexpectedEof`]: ../../std/io/enum.ErrorKind.html#variant.UnexpectedEof
    ///
    /// ```no_run
    /// use std::io::{self, BufReader};
    /// use std::io::prelude::*;
    /// use std::fs::File;
    /// use omnom::prelude::*;
    ///
    /// let mut cursor = io::Cursor::new(b"lorem-ipsum");
    /// let mut buf = vec![0; 5];
    ///
    /// // read exactly 5 bytes
    /// cursor.fill_exact(&mut buf).unwrap();
    /// assert_eq!(buf, b"lorem");
    /// buf.clear();
    ///
    /// // the same bytes can be read again
    /// cursor.fill_exact(&mut buf).unwrap();
    /// assert_eq!(buf, b"lorem");
    /// buf.clear();
    /// cursor.consume(5);
    ///
    /// // after consuming bytes we read new bytes
    /// cursor.fill_exact(&mut buf).unwrap();
    /// assert_eq!(buf, b"-ipsu");
    /// ```
    fn fill_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        loop {
            let available = match self.fill_buf() {
                Ok(n) => n,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };

            if available.len() >= buf.len() {
                buf.copy_from_slice(&available[..buf.len()]);
                break;
            }
        }

        Ok(())
    }

    /// Skip the first `n` bytes.
    fn skip(&mut self, n: usize) -> io::Result<()> {
        let mut read = 0;

        while read < n {
            let available = match self.fill_buf() {
                Ok(b) => b,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };

            if available.len() == 0 {
                break;
            } else {
                self.consume(1);
                read += 1;
            }

            let mut byte = 0;
            match self.read(slice::from_mut(&mut byte)) {
                Ok(0) => break,
                Ok(_) => {
                    read += 1;
                    continue;
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
        }

        self.consume(read);
        Ok(())
    }

    /// Skip bytes while the predicate is true.
    fn skip_while<P>(&mut self, mut predicate: P) -> io::Result<usize>
    where
        P: FnMut(u8) -> bool,
    {
        let mut read = 0;
        loop {
            let available = match self.fill_buf() {
                Ok(b) => b,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };

            if available.len() == 0 {
                break;
            } if predicate(available[0]) {
                self.consume(1);
                read += 1;
            } else {
                break;
            }
        }

        Ok(read)
    }
}

impl<T: BufRead> BufReadExt for T {}
