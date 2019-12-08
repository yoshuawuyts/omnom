use std::io::{self, BufRead, Read};
use std::mem;

use crate::BufReadExt;

/// Trait to enable writing bytes to a reader.
pub trait ReadBytes: Sized {
    /// Read bytes from a reader as big endian.
    ///
    /// Returns the amount of bytes read.
    fn read_be_bytes<R: Read>(reader: &mut R) -> io::Result<Self>;

    /// Read bytes from a reader as little endian.
    ///
    /// Returns the amount of bytes read.
    fn read_le_bytes<R: Read>(reader: &mut R) -> io::Result<Self>;

    /// Read bytes from a reader using native endianness.
    ///
    /// Returns the amount of bytes read.
    fn read_ne_bytes<R: Read>(reader: &mut R) -> io::Result<Self>;

    /// Fill bytes from a reader as big endian.
    ///
    /// Returns the amount of bytes read.
    fn fill_be_bytes<R: BufRead>(reader: &mut R) -> io::Result<Self>;

    /// Fill bytes from a reader as little endian.
    ///
    /// Returns the amount of bytes read.
    fn fill_le_bytes<R: BufRead>(reader: &mut R) -> io::Result<Self>;

    /// Fill bytes from a reader using native endianness.
    ///
    /// Returns the amount of bytes read.
    fn fill_ne_bytes<R: BufRead>(reader: &mut R) -> io::Result<Self>;
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
use std::io::{Cursor, Seek, SeekFrom};
use omnom::prelude::*;

let mut buf = Cursor::new(vec![0; 15]);

let num = 12_", stringify!($SelfT), ";
buf.write_be(num).unwrap();

buf.seek(SeekFrom::Start(0)).unwrap();
let num: ", stringify!($SelfT), " = buf.read_be().unwrap();
assert_eq!(num, 12);
```"),
            fn read_be_bytes<R: Read>(reader: &mut R) -> io::Result<Self> {
                let mut buf = [0; mem::size_of::<$SelfT>()];
                reader.read_exact(&mut buf)?;
                Ok(<$SelfT>::from_be_bytes(buf))
            }
        }

        doc_comment! {
            concat!("Read bytes from a reader as little endian.

# Examples

```
use std::io::{Cursor, Seek, SeekFrom};
use omnom::prelude::*;

let mut buf = Cursor::new(vec![0; 15]);

let num = 12_", stringify!($SelfT), ";
buf.write_le(num).unwrap();

buf.seek(SeekFrom::Start(0)).unwrap();
let num: ", stringify!($SelfT), " = buf.read_le().unwrap();
assert_eq!(num, 12);
```"),
            fn read_le_bytes<R: Read>(reader: &mut R) -> io::Result<Self> {
                let mut buf = [0; mem::size_of::<$SelfT>()];
                reader.read_exact(&mut buf)?;
                Ok(<$SelfT>::from_le_bytes(buf))
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
use std::io::{Cursor, Seek, SeekFrom};
use omnom::prelude::*;

let mut buf = Cursor::new(vec![0; 15]);

let num = 12_", stringify!($SelfT), ";
buf.write_ne(num).unwrap();

buf.seek(SeekFrom::Start(0)).unwrap();
let num: ", stringify!($SelfT), " = buf.read_ne().unwrap();
assert_eq!(num, 12);
```"),
            fn read_ne_bytes<R: Read>(reader: &mut R) -> io::Result<Self> {
                let mut buf = [0; mem::size_of::<$SelfT>()];
                reader.read_exact(&mut buf)?;
                Ok(<$SelfT>::from_ne_bytes(buf))
            }
        }

        doc_comment! {
            concat!("Read bytes from a reader as big endian.

# Examples

```
use std::io::{Cursor, Seek, SeekFrom};
use omnom::prelude::*;

let mut buf = Cursor::new(vec![0; 15]);

let num = 12_", stringify!($SelfT), ";
buf.write_be(num).unwrap();

buf.seek(SeekFrom::Start(0)).unwrap();
let num: ", stringify!($SelfT), " = buf.read_be().unwrap();
assert_eq!(num, 12);
```"),
            fn fill_be_bytes<R: BufRead>(reader: &mut R) -> io::Result<Self> {
                let mut buf = [0; mem::size_of::<$SelfT>()];
                reader.fill_exact(&mut buf)?;
                Ok(<$SelfT>::from_be_bytes(buf))
            }
        }

        doc_comment! {
            concat!("Read bytes from a reader as little endian.

# Examples

```
use std::io::{Cursor, Seek, SeekFrom};
use omnom::prelude::*;

let mut buf = Cursor::new(vec![0; 15]);

let num = 12_", stringify!($SelfT), ";
buf.write_le(num).unwrap();

buf.seek(SeekFrom::Start(0)).unwrap();
let num: ", stringify!($SelfT), " = buf.read_le().unwrap();
assert_eq!(num, 12);
```"),
            fn fill_le_bytes<R: BufRead>(reader: &mut R) -> io::Result<Self> {
                let mut buf = [0; mem::size_of::<$SelfT>()];
                reader.fill_exact(&mut buf)?;
                Ok(<$SelfT>::from_le_bytes(buf))
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
use std::io::{Cursor, Seek, SeekFrom};
use omnom::prelude::*;

let mut buf = Cursor::new(vec![0; 15]);

let num = 12_", stringify!($SelfT), ";
buf.write_ne(num).unwrap();

buf.seek(SeekFrom::Start(0)).unwrap();
let num: ", stringify!($SelfT), " = buf.read_ne().unwrap();
assert_eq!(num, 12);
```"),
            fn fill_ne_bytes<R: BufRead>(reader: &mut R) -> io::Result<Self> {
                let mut buf = [0; mem::size_of::<$SelfT>()];
                reader.fill_exact(&mut buf)?;
                Ok(<$SelfT>::from_ne_bytes(buf))
            }
        }
    }
)*}}

read_bytes_impl!(u8, u16, u32, u64, u128, usize);
read_bytes_impl!(i8, i16, i32, i64, i128, isize);
