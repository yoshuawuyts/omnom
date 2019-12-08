use omnom::prelude::*;
use std::collections::HashMap;
use std::io::{BufRead, Cursor, Read};

fn main() {
    assert_eq!(
        parse_mime("text/html").unwrap(),
        Mime {
            base_type: "text".to_string(),
            sub_type: "html".to_string(),
            parameters: None,
        }
    );

    let mut parameters = HashMap::new();
    parameters.insert("charset".to_string(), "utf-8".to_string());
    assert_eq!(
        parse_mime("text/html; charset=utf-8;").unwrap(),
        Mime {
            base_type: "text".to_string(),
            sub_type: "html".to_string(),
            parameters: Some(parameters),
        }
    );
}

#[derive(Eq, PartialEq, Debug)]
pub struct Mime {
    base_type: String,
    sub_type: String,
    parameters: Option<HashMap<String, String>>,
}

fn parse_mime(s: &str) -> Option<Mime> {
    // parse the "type"
    //
    // ```txt
    // text/html; charset=utf-8;
    // ^^^^^
    // ```
    let mut s = Cursor::new(s);
    let mut base_type = vec![];
    match s.read_until(b'/', &mut base_type).unwrap() {
        0 => return None,
        _ => base_type.pop(),
    };
    validate_code_points(&base_type)?;

    // parse the "subtype"
    //
    // ```txt
    // text/html; charset=utf-8;
    //      ^^^^^
    // ```
    let mut sub_type = vec![];
    s.read_until(b';', &mut sub_type).unwrap();
    if let Some(b';') = sub_type.last() {
        sub_type.pop();
    }
    validate_code_points(&sub_type)?;

    // instantiate our mime struct
    let mut mime = Mime {
        base_type: String::from_utf8(base_type).unwrap(),
        sub_type: String::from_utf8(sub_type).unwrap(),
        parameters: None,
    };

    // parse parameters into a hashmap
    //
    // ```txt
    // text/html; charset=utf-8;
    //           ^^^^^^^^^^^^^^^
    // ```
    loop {
        // Stop parsing if there's no more bytes to consume.
        if s.fill_buf().unwrap().len() == 0 {
            break;
        }

        // Trim any whitespace.
        //
        // ```txt
        // text/html; charset=utf-8;
        //           ^
        // ```
        s.skip_while(is_http_whitespace_char).ok()?;

        // Get the param name.
        //
        // ```txt
        // text/html; charset=utf-8;
        //            ^^^^^^^
        // ```
        let mut param_name = vec![];
        s.read_while(&mut param_name, |b| b != b';' && b != b'=')
            .ok()?;
        validate_code_points(&param_name)?;
        let mut param_name = String::from_utf8(param_name).ok()?;
        param_name.make_ascii_lowercase();

        // Ignore param names without values.
        //
        // ```txt
        // text/html; charset=utf-8;
        //                   ^
        // ```
        let mut token = vec![0; 1];
        s.read_exact(&mut token).unwrap();
        if token[0] == b';' {
            continue;
        }

        // Get the param value.
        //
        // ```txt
        // text/html; charset=utf-8;
        //                    ^^^^^^
        // ```
        let mut param_value = vec![];
        s.read_until(b';', &mut param_value).ok()?;
        if let Some(b';') = param_value.last() {
            param_value.pop();
        }
        validate_code_points(&param_value)?;
        let mut param_value = String::from_utf8(param_value).ok()?;
        param_value.make_ascii_lowercase();

        // Insert attribute pair into hashmap.
        if let None = mime.parameters {
            mime.parameters = Some(HashMap::new());
        }
        mime.parameters.as_mut()?.insert(param_name, param_value);
    }

    Some(mime)
}

fn validate_code_points(buf: &[u8]) -> Option<()> {
    let all = buf.iter().all(|b| match b {
        b'-' | b'!' | b'#' | b'$' | b'%' => true,
        b'&' | b'\'' | b'*' | b'+' | b'.' => true,
        b'^' | b'_' | b'`' | b'|' | b'~' => true,
        b'A'..=b'Z' => true,
        b'a'..=b'z' => true,
        b'0'..=b'9' => true,
        _ => false,
    });

    if all {
        Some(())
    } else {
        None
    }
}

fn is_http_whitespace_char(b: u8) -> bool {
    match b {
        b' ' | b'\t' | b'\n' | b'\r' => true,
        _ => false,
    }
}
