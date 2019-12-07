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
    if !validate_code_points(&base_type) {
        return None;
    }

    // parse the "subtype"
    //
    // ```txt
    // text/html; charset=utf-8;
    //      ^^^^^
    // ```
    let mut sub_type = vec![];
    let read = s.read_until(b';', &mut sub_type).unwrap();
    if read == 0 {
        return None;
    }
    if sub_type[sub_type.len() - 1] == b';' {
        sub_type.pop();
    }
    if !validate_code_points(&sub_type) {
        return None;
    }

    let mut mime = Mime {
        base_type: String::from_utf8(base_type).unwrap(),
        sub_type: String::from_utf8(sub_type).unwrap(),
        parameters: None,
    };

    // Parse parameters into a hashmap.
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
        s.skip_while(is_http_whitespace_char).unwrap();

        // Get the param name.
        //
        // ```txt
        // text/html; charset=utf-8;
        //            ^^^^^^^
        // ```
        let mut param_name = vec![];
        s.read_while(&mut param_name, |b| b != b';' && b != b'=')
            .unwrap();
        let mut param_name = String::from_utf8(param_name).unwrap();
        param_name.make_ascii_lowercase();
        if !validate_code_points(&param_name.as_bytes()) {
            return None;
        }

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

        match s.read_until(b';', &mut param_value).unwrap() {
            0 => return None,
            _ => param_value.pop(),
        };
        let mut param_value = String::from_utf8(param_value).unwrap();
        param_value.make_ascii_lowercase();
        if !validate_code_points(&param_value.as_bytes()) {
            return None;
        }

        // Insert attribute pair into hashmap.
        if let None = mime.parameters {
            mime.parameters = Some(HashMap::new());
        }

        mime.parameters
            .as_mut()
            .unwrap()
            .insert(param_name, param_value);
    }

    Some(mime)
}

fn validate_code_points(buf: &[u8]) -> bool {
    buf.iter().all(|b| match b {
        b'-'
        | b'!'
        | b'#'
        | b'$'
        | b'%'
        | b'&'
        | b'\''
        | b'*'
        | b'+'
        | b'.'
        | b'^'
        | b'_'
        | b'`'
        | b'|'
        | b'~'
        | b'A'..=b'Z'
        | b'a'..=b'z'
        | b'0'..=b'9' => true,
        _ => false,
    })
}

fn is_http_whitespace_char(b: u8) -> bool {
    match b {
        b' ' | b'\t' | b'\n' | b'\r' => true,
        _ => false,
    }
}
