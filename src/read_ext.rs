use std::io::{self, Read};

use crate::ReadBytes;

/// Extension trait to `Read` to read bytes using endianness.
pub trait ReadExt: Read + Sized {
    /// Read bytes as big endian.
    fn read_be_bytes<B: ReadBytes>(&mut self) -> io::Result<B> {
        <B>::read_be_bytes(self)
    }

    /// Read bytes as little endian.
    fn read_le_bytes<B: ReadBytes>(&mut self) -> io::Result<B> {
        <B>::read_le_bytes(self)
    }

    /// Read bytes using native endianness.
    fn read_ne_bytes<B: ReadBytes>(&mut self) -> io::Result<B> {
        <B>::read_ne_bytes(self)
    }
}

impl<T: Read> ReadExt for T {}
