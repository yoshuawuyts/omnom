use std::io::{self, BufRead, ErrorKind, Read};
use std::slice;

use crate::ReadBytes;

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
                }
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
            }
            if predicate(available[0]) {
                self.consume(1);
                read += 1;
            } else {
                break;
            }
        }

        Ok(read)
    }

    /// Skip bytes until the delimiter `byte` or EOF is reached.
    ///
    /// This function will read bytes from the underlying stream until the
    /// delimiter or EOF is found. Once found, all bytes up to, and including,
    /// the delimiter (if found) will be skipped.
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
    ///
    /// // skip up to and including '-'
    /// let num_bytes = cursor.skip_until(b'-').unwrap();
    /// assert_eq!(num_bytes, 6);
    ///
    /// // read the rest of the bytes
    /// let mut buf = [0; 5];
    /// cursor.fill_exact(&mut buf).unwrap();
    /// assert_eq!(&buf, b"ipsum");
    /// ```
    fn skip_until(&mut self, byte: u8) -> io::Result<usize> {
        let mut read = 0;
        loop {
            let available = match self.fill_buf() {
                Ok(b) => b,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };

            if available.len() == 0 {
                break;
            }

            if available[0] == byte {
                self.consume(1);
                read += 1;
                break;
            } else {
                self.consume(1);
                read += 1;
            }
        }

        Ok(read)
    }

    /// Fill bytes as big endian.
    fn fill_be<B: ReadBytes>(&mut self) -> io::Result<B> where Self: Sized {
        <B>::fill_be_bytes(self)
    }

    /// Fill bytes as little endian.
    fn fill_le<B: ReadBytes>(&mut self) -> io::Result<B> where Self: Sized {
        <B>::fill_le_bytes(self)
    }

    /// Fill bytes using native endianness.
    fn fill_ne<B: ReadBytes>(&mut self) -> io::Result<B> where Self: Sized {
        <B>::fill_ne_bytes(self)
    }
}

impl<T: BufRead> BufReadExt for T {}
