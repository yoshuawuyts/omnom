use omnom::prelude::*;
use std::io::{self, BufRead};

fn main() {
    let mut cursor = io::Cursor::new(b"lorem-ipsum");
    let mut buf = vec![];

    dbg!("hi");

    // cursor is at 'l'
    let num_bytes = cursor
        .try_read_until(b'-', &mut buf)
        .expect("reading from cursor won't fail");
    cursor.consume(6);
    assert_eq!(buf, b"lorem-");
    assert_eq!(num_bytes, 6);
    buf.clear();

    // cursor is at 'i'
    let num_bytes = cursor
        .try_read_until(b'-', &mut buf)
        .expect("reading from cursor won't fail");
    cursor.consume(5);
    assert_eq!(buf, b"ipsum");
    assert_eq!(num_bytes, 5);
    buf.clear();

    // cursor is at EOF
    let num_bytes = cursor
        .try_read_until(b'-', &mut buf)
        .expect("reading from cursor won't fail");
    assert_eq!(num_bytes, 0);
    assert_eq!(buf, b"");
}
