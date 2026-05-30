use std::collections::HashMap;

#[derive(Debug)]
pub struct ParsedValue {
    pub value: Bencode,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Bencode {
    Integer(i64),
    String(Vec<u8>),
    List(Vec<ParsedValue>),
    Dictionary(HashMap<String, ParsedValue>),
}

impl Bencode {
    pub fn as_dictionary(&self) -> Option<&HashMap<String, ParsedValue>> {
        match self {
            Bencode::Dictionary(dict) => Some(dict),
            _ => None,
        }
    }
}

pub fn parse(data: &[u8]) -> Result<ParsedValue, String> {
    let mut pos = 0;
    parse_value(data, &mut pos)
}

fn parse_value(data: &[u8], pos: &mut usize) -> Result<ParsedValue, String> {
    let start = *pos;

    let value = match data[*pos] {
        b'i' => parse_integer(data, pos)?,
        b'l' => parse_list(data, pos)?,
        b'd' => parse_dictionary(data, pos)?,
        b'0'..=b'9' => parse_string(data, pos)?,
        _ => return Err(format!("unexpected byte {}", data[*pos])),
    };

    let end = *pos;

    Ok(ParsedValue { value, start, end })
}

fn parse_string(data: &[u8], pos: &mut usize) -> Result<Bencode, String> {
    let start = *pos;

    while *pos < data.len() && data[*pos] != b':' {
        *pos += 1;
    }

    if *pos >= data.len() {
        return Err("unterminated string length".into());
    }

    let len_str = std::str::from_utf8(&data[start..*pos]).map_err(|_| "invalid string length")?;

    let len: usize = len_str.parse().map_err(|_| "invalid string length")?;

    *pos += 1; // skip ':'

    let end = *pos + len;

    if end > data.len() {
        return Err("string extends beyond buffer".into());
    }

    let value = data[*pos..end].to_vec();
    *pos = end;

    Ok(Bencode::String(value))
}

fn parse_integer(data: &[u8], pos: &mut usize) -> Result<Bencode, String> {
    *pos += 1; // skip 'i'

    let start = *pos;

    while *pos < data.len() && data[*pos] != b'e' {
        *pos += 1;
    }

    let value = std::str::from_utf8(&data[start..*pos])
        .map_err(|_| "invalid integer")?
        .parse::<i64>()
        .map_err(|_| "invalid integer")?;

    *pos += 1; // skip 'e'

    Ok(Bencode::Integer(value))
}

fn parse_dictionary(data: &[u8], pos: &mut usize) -> Result<Bencode, String> {
    *pos += 1; // skip 'd'

    let mut map = HashMap::new();

    while *pos < data.len() && data[*pos] != b'e' {
        let key = match parse_string(data, pos)? {
            Bencode::String(bytes) => {
                String::from_utf8(bytes).map_err(|_| "dictionary key is not utf8")?
            }
            _ => unreachable!(),
        };

        let value = parse_value(data, pos)?;

        map.insert(key, value);
    }

    *pos += 1; // skip 'e'

    Ok(Bencode::Dictionary(map))
}

fn parse_list(data: &[u8], pos: &mut usize) -> Result<Bencode, String> {
    *pos += 1; // skip 'l'

    let mut values = Vec::new();

    while *pos < data.len() && data[*pos] != b'e' {
        values.push(parse_value(data, pos)?);
    }

    *pos += 1; // skip 'e'

    Ok(Bencode::List(values))
}
