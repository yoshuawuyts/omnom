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
//! - [`BufReadExt::fill_be`] reads bytes as big-endian from a reader, doesn't consume bytes.
//! - [`BufReadExt::fill_exact`] reads bytes until a buffer has been filled, doesn't consume bytes.
//! - [`BufReadExt::fill_le`] reads bytes as little-endian from a reader, doesn't consume bytes.
//! - [`BufReadExt::fill_ne`] reads bytes using native endianness from a reader, doesn't consume bytes.
//! - [`BufReadExt::fill_until`] reads bytes until a byte has been encountered, doesn't consume bytes.
//! - [`BufReadExt::fill_while`] reads bytes based on a predicate, doesn't consume bytes.
//! - [`BufReadExt::read_while`] reads bytes based on a predicate, consumes bytes.
//! - [`BufReadExt::skip`] Skip the first `n` bytes.
//! - [`BufReadExt::skip_until`] Skip bytes until the delimiter `byte` or EOF is reached.
//! - [`BufReadExt::skip_while`] Skip bytes while a predicate is true.
//! - [`ReadExt::read_be`] reads bytes as big-endian from a reader, consumes bytes.
//! - [`ReadExt::read_le`] reads bytes as little-endian from a reader, consumes bytes.
//! - [`ReadExt::read_ne`] reads bytes using native endianness from a reader, consumes bytes.
//! - [`WriteExt::write_be`] write bytes as big-endian to a writer.
//! - [`WriteExt::write_le`] write bytes as little-endian to a writer.
//! - [`WriteExt::write_ne`] write bytes using native endianness to a writer.
//!
//! [`BufReadExt::fill_be`]: trait.BufReadExt.html#method.fill_be
//! [`BufReadExt::fill_exact`]: trait.BufReadExt.html#method.fill_exact
//! [`BufReadExt::fill_le`]: trait.BufReadExt.html#method.fill_le
//! [`BufReadExt::fill_ne`]: trait.BufReadExt.html#method.fill_ne
//! [`BufReadExt::fill_until`]: trait.BufReadExt.html#method.fill_until
//! [`BufReadExt::fill_while`]: trait.BufReadExt.html#method.fill_while
//! [`BufReadExt::read_while`]: trait.BufReadExt.html#method.read_while
//! [`BufReadExt::skip`]: trait.BufReadExt.html#method.skip
//! [`BufReadExt::skip_until`]: trait.BufReadExt.html#method.skip_until
//! [`BufReadExt::skip_while`]: trait.BufReadExt.html#method.skip_while
//! [`ReadExt::read_be`]: trait.ReadExt.html#method.read_be
//! [`ReadExt::read_le`]: trait.ReadExt.html#method.read_le
//! [`ReadExt::read_ne`]: trait.ReadExt.html#method.read_ne
//! [`WriteExt::write_be`]: trait.WriteExt.html#method.write_be
//! [`WriteExt::write_le`]: trait.WriteExt.html#method.write_le
//! [`WriteExt::write_ne`]: trait.WriteExt.html#method.write_ne
//! [`consume`]: https://doc.rust-lang.org/std/io/trait.BufRead.html#tymethod.consume
//!
//! # Todos
//!
//! - `AsyncRead` support.
//!
//! # Examples
//!
//! Read and write integers from IO streams with a chosen endianness:
//!
//! ```
//! use std::io::{Cursor, Seek, SeekFrom};
//! use omnom::prelude::*;
//!
//! let mut buf = Cursor::new(vec![0; 15]);
//!
//! let num = 12_u16;
//! buf.write_le(num).unwrap();
//!
//! buf.seek(SeekFrom::Start(0)).unwrap();
//! let num: u16 = buf.read_le().unwrap();
//! assert_eq!(num, 12);
//! ```
//!
//! Fill a buffer without immediately consuming the bytes:
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

mod buf_read_ext;
mod read_bytes;
mod read_ext;
mod write_bytes;
mod write_ext;

pub use buf_read_ext::BufReadExt;
pub use read_bytes::ReadBytes;
pub use read_ext::ReadExt;
pub use write_bytes::WriteBytes;
pub use write_ext::WriteExt;

/// The `omnom` prelude.
pub mod prelude {
    pub use crate::BufReadExt;
    pub use crate::ReadBytes;
    pub use crate::ReadExt;
    pub use crate::WriteBytes;
    pub use crate::WriteExt;
}
