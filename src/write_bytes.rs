use std::io::{self, Write};

/// Trait to enable writing bytes to a writer.
pub trait WriteBytes {
    /// Write bytes to a writer as big endian.
    ///
    /// Returns the amount of bytes written.
    fn write_be_bytes<W: Write>(&self, writer: &mut W) -> io::Result<usize>;

    /// Write bytes to a writer as little endian.
    ///
    /// Returns the amount of bytes written.
    fn write_le_bytes<W: Write>(&self, writer: &mut W) -> io::Result<usize>;

    /// Write bytes to a writer using native endianness.
    ///
    /// Returns the amount of bytes written.
    fn write_ne_bytes<W: Write>(&self, writer: &mut W) -> io::Result<usize>;
}

macro_rules! doc_comment {
    ($x:expr, $($tt:tt)*) => {
        #[doc = $x]
        $($tt)*
    };
}

macro_rules! write_bytes_impl {
    ($($SelfT:ty),* $(,)?) => { $(
        impl WriteBytes for $SelfT {
        doc_comment! {
            concat!("Write bytes to a writer as big endian.

# Examples

```
use std::io::Cursor;
use omnom::prelude::*;

let mut buf = Cursor::new(vec![0; 15]);

let num = 12_", stringify!($SelfT), ";
buf.write_be_bytes(num).unwrap();
```"),
            fn write_be_bytes<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
                let b = &self.to_be_bytes();
                let len = b.len();
                writer.write_all(b)?;
                Ok(len)
            }
        }

        doc_comment! {
            concat!("Write bytes to a writer as little endian.

# Examples

```
use std::io::Cursor;
use omnom::prelude::*;

let mut buf = Cursor::new(vec![0; 15]);

let num = 12_", stringify!($SelfT), ";
buf.write_le_bytes(num).unwrap();
```"),
            fn write_le_bytes<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
                let b = &self.to_le_bytes();
                let len = b.len();
                writer.write_all(b)?;
                Ok(len)
            }
        }

        doc_comment! {
            concat!("Write bytes to a writer using native endianness.

As the target platform's native endianness is used, portable code
likely wants to use [`from_be_bytes`] or [`from_le_bytes`], as
appropriate instead.

[`write_be_bytes`]: #method.write_be_bytes
[`write_le_bytes`]: #method.write_le_bytes

# Examples

```
use std::io::Cursor;
use omnom::prelude::*;

let mut buf = Cursor::new(vec![0; 15]);

let num = 12_", stringify!($SelfT), ";
buf.write_ne_bytes(num).unwrap();
```"),
            fn write_ne_bytes<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
                let b = &self.to_ne_bytes();
                let len = b.len();
                writer.write_all(b)?;
                Ok(len)
            }
        }
    }
)*}}

write_bytes_impl!(u8, u16, u32, u64, u128, usize);
write_bytes_impl!(i8, i16, i32, i64, i128, isize);
