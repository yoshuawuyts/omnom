use std::io::{self, Read};

/// Trait to enable writing bytes to a writer.
pub trait ReadBytes {
    /// Read bytes from a reader as big endian.
    ///
    /// Returns the amount of bytes read.
    fn read_be_bytes<R: Read>(&self, reader: &mut R) -> io::Result<usize>;

    /// Read bytes from a reader as little endian.
    ///
    /// Returns the amount of bytes read.
    fn read_le_bytes<R: Read>(&self, reader: &mut R) -> io::Result<usize>;

    /// Read bytes from a reader using native endianness.
    ///
    /// Returns the amount of bytes read.
    fn read_ne_bytes<R: Read>(&self, reader: &mut R) -> io::Result<usize>;
}

macro_rules! doc_comment {
    ($x:expr, $($tt:tt)*) => {
        #[doc = $x]
        $($tt)*
    };
}

macro_rules! read_bytes_impl {
    ($($SelfT:ty),* $(,)?) => { $(
        impl ReadBytes for $SelfT {
        doc_comment! {
            concat!("Read bytes from a reader as big endian.

# Examples

```
use std::io::Cursor;
use omnom::prelude::*;

let mut buf = Cursor::new(vec![0; 15]);

let num = 12_", stringify!($SelfT), ";
buf.read_be_bytes(num).unwrap();
```"),
            fn read_be_bytes<R: Read>(&self, reader: &mut R) -> io::Result<usize> {
                let b = &self.to_be_bytes();
                let len = b.len();
                writer.read_all(b)?;
                Ok(len)
            }
        }

        doc_comment! {
            concat!("Read bytes from a reader as little endian.

# Examples

```
use std::io::Cursor;
use omnom::prelude::*;

let mut buf = Cursor::new(vec![0; 15]);

let num = 12_", stringify!($SelfT), ";
buf.read_le_bytes(num).unwrap();
```"),
            fn read_le_bytes<R: Read>(&self, reader: &mut R) -> io::Result<usize> {
                let b = &self.to_le_bytes();
                let len = b.len();
                writer.read_all(b)?;
                Ok(len)
            }
        }

        doc_comment! {
            concat!("Read bytes from a reader using native endianness.

As the target platform's native endianness is used, portable code
likely wants to use [`read_be_bytes`] or [`read_le_bytes`], as
appropriate instead.

[`read_be_bytes`]: #method.read_be_bytes
[`read_le_bytes`]: #method.read_le_bytes

# Examples

```
use std::io::Cursor;
use omnom::prelude::*;

let mut buf = Cursor::new(vec![0; 15]);

let num = 12_", stringify!($SelfT), ";
buf.read_ne_bytes(num).unwrap();
```"),
            fn read_ne_bytes<R: Read>(&self, reader: &mut R) -> io::Result<usize> {
                let b = &self.to_ne_bytes();
                let len = b.len();
                writer.read_all(b)?;
                Ok(len)
            }
        }
    }
)*}}

read_bytes_impl!(u8, u16, u32, u64, u128, usize);
read_bytes_impl!(i8, i16, i32, i64, i128, isize);
