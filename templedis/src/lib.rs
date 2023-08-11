use regex::Regex;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum RESPDataType {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Option<Vec<RESPDataType>>),
}

pub struct RESPData {
    data: RESPDataType,
    size: usize,
}

impl RESPData {
    pub fn new(data: RESPDataType, size: usize) -> Self {
        RESPData { data, size }
    }
}

pub trait RESPDeserializer {
    fn deserialize(&self) -> String;
}

impl RESPDeserializer for RESPDataType {
    fn deserialize(&self) -> String {
        match self {
            RESPDataType::SimpleString(s) => format!("+{}\r\n", s),
            RESPDataType::Error(s) => format!("-{}\r\n", s),
            RESPDataType::Integer(i) => format!(":{}\r\n", i),
            RESPDataType::BulkString(s) => {
                if let Some(s) = s.clone() {
                    return format!("${}\r\n{}\r\n", s.len(), s);
                }
                return format!("None, 0");
            }
            RESPDataType::Array(v) => {
                if let Some(v) = v.clone() {
                    let mut result = String::new();
                    result.push_str(&format!("*{}\r\n", v.len()));
                    for item in v {
                        result.push_str(&item.deserialize());
                    }
                    return result;
                } else {
                    return format!("None, 0");
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

fn formatter(buffer: &str) -> Vec<&str> {
    let re = Regex::new(r"\r\n").unwrap();
    let split = re.split(buffer).collect::<Vec<&str>>();
    split
}

fn serialize(split: Vec<&str>) -> Option<RESPData> {
    if split.len() == 2 {
        let (first, second) = split[0].split_at(1);
        if first == "+" {
            return Some(RESPData::new(RESPDataType::SimpleString(second.to_string()), second.len() + 3));
        }
        if first == "-" {
            return Some(RESPData::new(RESPDataType::Error(second.to_string()), second.len() + 3));
        }
        if first == ":" {
            return Some(RESPData::new(RESPDataType::Integer(second.parse().unwrap()), second.len() + 3));
        }
        if first == "$" {
            let num: i32 = second.parse().unwrap();
            if num == -1 {
                return Some(RESPData::new(RESPDataType::BulkString(None), 0));
            }
        }

        if first == "*" {
            let num: i64 = second.parse().unwrap();
            if num == -1 {
                return Some(RESPData::new(RESPDataType::Array(None), 0));
            } else if num == 0 {
                return Some(RESPData::new(RESPDataType::Array(Some(vec![])), 0));
            }
        }
    }

    if split.len() > 2 {
        let (first, second) = split[0].split_at(1);
        if first == "$" {
            let num: i64 = second.parse().unwrap();
            if num == 0 {
                return Some(RESPData::new(RESPDataType::BulkString(Some("".to_string())), 6));
            }
            if split.len() == 3 {
                return Some(RESPData::new(RESPDataType::BulkString(Some(split[1].to_string())), split[1].len() + 6));
            }
        }
        if first == "*" {
            let num: i64 = second.parse().unwrap();
            if num == 0 {
                return Some(RESPData::new(RESPDataType::Array(Some(vec![])), 0));
            }

            let mut results = Vec::new();
            let mut size = 0;
            for i in (1..split.len() - 1).step_by(2) {
                let result = serialize(vec![split[i], split[i + 1], ""]).unwrap();
                size += result.size;
                results.push(result.data);
            }
            return Some(RESPData::new(RESPDataType::Array(Some(results)), size + 4));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("+OK\r\n", RESPDataType::SimpleString("OK".to_string()), 5; "simple string with OK")]
    #[test_case("+OK\r\n+Next", RESPDataType::SimpleString("OK".to_string()), 5; "simple string with OK and cut next")]
    #[test_case("+\r\n", RESPDataType::SimpleString("".to_string()), 3; "simple string with empty string")]
    #[test_case("-Error message\r\n", RESPDataType::Error("Error message".to_string()), 16; "error with data")]
    #[test_case(":42\r\n", RESPDataType::Integer(42), 5; "integer with 42")]
    #[test_case("$4\r\nTest\r\n", RESPDataType::BulkString(Some("Test".to_string())), 10; "bulk string with Test")]
    #[test_case("$0\r\n\r\n", RESPDataType::BulkString(Some("".to_string())), 6; "bulk string with empty string")]
    #[test_case("$-1\r\n", RESPDataType::BulkString(None), 0; "bulk string with None")]
    #[test_case("*-1\r\n", RESPDataType::Array(None), 0; "array with None")]
    #[test_case("*0\r\n", RESPDataType::Array(Some(vec ! [])), 0; "array empty")]
    #[test_case("*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n", RESPDataType::Array(Some(vec ! [
    RESPDataType::BulkString(Some("hello".to_string())),
    RESPDataType::BulkString(Some("world".to_string()))])), 26; "array with 2 bulk strings")]
    fn serialization_tests(buffer: &str, expected: RESPDataType, size: usize) {
        if let Some(data) = serialize(formatter(buffer)) {
            assert_eq!(expected, data.data);
            assert_eq!(size, data.size);
        } else {
            assert!(false);
        }
    }
}
