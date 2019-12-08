use std::io::{self, Write};

/// Trait to enable writing bytes to a writer.
pub trait WriteBytes {
    /// Write bytes to a writer as big endian.
    fn write_be_bytes<W: Write>(&self, writer: &mut W) -> io::Result<usize>;

    /// Write bytes to a writer as little endian.
    fn write_le_bytes<W: Write>(&self, writer: &mut W) -> io::Result<usize>;

    /// Write bytes to a writer using native endianness.
    fn write_ne_bytes<W: Write>(&self, writer: &mut W) -> io::Result<usize>;
}

impl WriteBytes for u8 {
    fn write_be_bytes<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        writer.write_all([self])
    }

    fn write_le_bytes<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        writer.write_all([self])
    }

    fn write_ne_bytes<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        writer.write_all([self])
    }
}

pub trait WriteExt: Write {
    /// Write bytes as big endian.
    fn write_be_bytes<B: WriteBytes>(&mut self, num: B) -> io::Result<usize> {
        num.write_be_bytes(self)
    }

    /// Write bytes as little endian.
    fn write_le_bytes<B: WriteBytes>(&mut self, num: B) -> io::Result<usize> {
        num.write_le_bytes(self)
    }

    /// Write bytes using native endianness.
    fn write_ne_bytes<B: WriteBytes>(&mut self, num: B) -> io::Result<usize> {
        num.write_ne_bytes(self)
    }
}
