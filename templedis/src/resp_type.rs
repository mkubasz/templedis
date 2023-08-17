use regex::Regex;
#[derive(Debug)]
#[derive(PartialEq)]
#[allow(dead_code)]
pub enum RESPType {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Option<Vec<RESPType>>),
}

#[allow(dead_code)]
pub struct RESPData {
    data: RESPType,
    size: usize,
}

#[allow(dead_code)]
impl RESPData {
    pub fn new(data: RESPType, size: usize) -> Self {
        RESPData { data, size }
    }
}

pub trait RESPDeserializer {
    fn deserialize(&self) -> String;
}

impl RESPDeserializer for RESPType {
    fn deserialize(&self) -> String {
        match self {
            RESPType::SimpleString(s) => format!("+{}\r\n", s),
            RESPType::Error(s) => format!("-{}\r\n", s),
            RESPType::Integer(i) => format!(":{}\r\n", i),
            RESPType::BulkString(s) => {
                if let Some(s) = s.clone() {
                    return format!("${}\r\n{}\r\n", s.len(), s);
                }
                return format!("None, 0");
            }
            RESPType::Array(v) => {
                return if let Some(v) = v.clone() {
                    let mut result = String::new();
                    result.push_str(&format!("*{}\r\n", v.len()));
                    for item in v {
                        result.push_str(&item.deserialize());
                    }
                    result
                } else {
                    format!("None, 0")
                }
            }
        }
    }
}

impl RESPDeserializer for RESPData {
    fn deserialize(&self) -> String {
        self.data.deserialize()
    }
}

#[allow(dead_code)]
fn formatter(buffer: &str) -> Vec<&str> {
    let re = Regex::new(r"\r\n").unwrap();
    let split = re.split(buffer).collect::<Vec<&str>>();
    split
}

#[allow(dead_code)]
fn serialize(split: Vec<&str>) -> Option<RESPData> {
    let (first, second) = split[0].split_at(1);
    match first {
        "+" => Some(RESPData::new(RESPType::SimpleString(second.to_string()), second.len() + 3)),
        "-" => Some(RESPData::new(RESPType::Error(second.to_string()), second.len() + 3)),
        ":" => Some(RESPData::new(RESPType::Integer(second.parse().unwrap()), second.len() + 3)),
        "$" => {
            let num: i64 = second.parse().unwrap();
            if num == -1 {
                return Some(RESPData::new(RESPType::BulkString(None), 0));
            }
            if num == 0 {
                return Some(RESPData::new(RESPType::BulkString(Some("".to_string())), 6));
            }
            if split.len() == 3 {
                return Some(RESPData::new(RESPType::BulkString(Some(split[1].to_string())), split[1].len() + 6));
            }
            None
        }
        "*" => {
            let num: i64 = second.parse().unwrap();
            if num == -1 {
                return Some(RESPData::new(RESPType::Array(None), 0));
            }
            if num == 0 {
                return Some(RESPData::new(RESPType::Array(Some(vec![])), 0));
            }

            let mut results = Vec::new();
            let mut size = 0;
            for i in (1..split.len() - 1).step_by(2) {
                let result = serialize(vec![split[i], split[i + 1], ""]).unwrap();
                size += result.size;
                results.push(result.data);
            }
            return Some(RESPData::new(RESPType::Array(Some(results)), size + 4));
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    #[rstest(
    buffer, expected, size,
    case("+OK\r\n", RESPType::SimpleString("OK".to_string()), 5),
    case("+OK\r\n+Next", RESPType::SimpleString("OK".to_string()), 5),

    case("+\r\n", RESPType::SimpleString("".to_string()), 3),
    case("-Error message\r\n", RESPType::Error("Error message".to_string()), 16),
    case("-Error message\r\n+Partial", RESPType::Error("Error message".to_string()), 16),

    case(":42\r\n", RESPType::Integer(42), 5),
    case("$4\r\nTest\r\n", RESPType::BulkString(Some("Test".to_string())), 10),
    case("$0\r\n\r\n", RESPType::BulkString(Some("".to_string())), 6),
    case("$-1\r\n", RESPType::BulkString(None), 0),
    case("*-1\r\n", RESPType::Array(None), 0),
    case("*0\r\n", RESPType::Array(Some(vec![])), 0),

    case("*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n", RESPType::Array(Some(vec![
        RESPType::BulkString(Some("hello".to_string())),
        RESPType::BulkString(Some("world".to_string()))])), 26),

    case("*3\r\n$3\r\nfoo\r\n$3\r\nbar\r\n$5\r\nHello\r\n", RESPType::Array(Some(vec![
        RESPType::BulkString(Some("foo".to_string())),
        RESPType::BulkString(Some("bar".to_string())),
        RESPType::BulkString(Some("Hello".to_string()))])), 33),
    )]

    fn serialization_tests(buffer: &str, expected: RESPType, size: usize) {
        if let Some(data) = serialize(formatter(buffer)) {
            assert_eq!(expected, data.data);
            assert_eq!(size, data.size);
        } else {
            assert!(false);
        }
    }
    #[rstest(
    buffer, input,
    case("+OK\r\n", RESPType::SimpleString("OK".to_string())),
    case("+\r\n", RESPType::SimpleString("".to_string())),
    case("-Error message\r\n", RESPType::Error("Error message".to_string())),
    case(":42\r\n", RESPType::Integer(42)),
    case("$4\r\nTest\r\n", RESPType::BulkString(Some("Test".to_string()))),
    case("$0\r\n\r\n", RESPType::BulkString(Some("".to_string()))),

    case("*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n", RESPType::Array(Some(vec![
    RESPType::BulkString(Some("hello".to_string())),
    RESPType::BulkString(Some("world".to_string()))]))),
    case("*3\r\n$3\r\nfoo\r\n$3\r\nbar\r\n$5\r\nHello\r\n", RESPType::Array(Some(vec![
    RESPType::BulkString(Some("foo".to_string())),
    RESPType::BulkString(Some("bar".to_string())),
    RESPType::BulkString(Some("Hello".to_string()))]))),
    )]

    fn deserialization_tests(buffer: &str, input: RESPType) {
        let result = input.deserialize();
        assert_eq!(buffer.to_string(), result);
    }
}
