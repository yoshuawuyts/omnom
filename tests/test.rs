use std::error::Error;

#[test]
fn should_work() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    Ok(())
}

#[test]
fn read_while_cursor() {
    use omnom::prelude::*;
    use std::io::BufRead;

    let mut s = std::io::Cursor::new("aaaaa");
    let mut buf = Vec::new();
    assert_eq!(s.read_while(&mut buf, |b| b == b'a').unwrap(), 5);
    assert_eq!(&buf[..], &b"aaaaa"[..]);
    assert_eq!(s.fill_buf().unwrap().len(), 0);

    let mut s = std::io::Cursor::new("ab");
    let mut buf = Vec::new();
    assert_eq!(s.read_while(&mut buf, |b| b == b'a').unwrap(), 1);
    assert_eq!(&buf[..], &b"a"[..]);
    assert_eq!(s.fill_buf().unwrap().len(), 1);

    let mut s = std::io::Cursor::new("ab");
    let mut buf = Vec::new();
    assert_eq!(s.read_while(&mut buf, |b| b == b'b').unwrap(), 0);
    assert_eq!(&buf[..], &b""[..]);
    assert_eq!(s.fill_buf().unwrap().len(), 2);
}
