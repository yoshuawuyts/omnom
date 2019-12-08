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

mod buf_read_ext;

pub use buf_read_ext::BufReadExt;

/// The `omnom` prelude.
pub mod prelude {
    pub use crate::BufReadExt;
}
