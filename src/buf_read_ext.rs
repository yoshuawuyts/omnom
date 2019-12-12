use std::io::{self, BufRead, ErrorKind, Read};
use std::slice;

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
        let mut inner_read = 0;

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
                    inner_read += 1;
                    read += 1;
                } else {
                    break 'outer;
                }
            }
            self.consume(inner_read);
            inner_read = 0;
        }
        self.consume(inner_read);

        Ok(read)
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
    /// use std::io::{self, BufRead, Read};
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
    /// cursor.read_exact(&mut buf).unwrap();
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
}

impl<T: BufRead> BufReadExt for T {}
