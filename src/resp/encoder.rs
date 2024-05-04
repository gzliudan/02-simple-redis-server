use crate::{
    BulkString, NullBulkString, RespArray, RespEncoder, RespMap, RespNull, RespNullArray, RespSet,
    SimpleError, SimpleString,
};

const BUFFER_CAP: usize = 4096;

// - simple string: "+OK\r\n"
impl RespEncoder for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}

// - error: "-Error message\r\n"
impl RespEncoder for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

// - integer: ":[<+|->]<value>\r\n"
impl RespEncoder for i64 {
    fn encode(self) -> Vec<u8> {
        format!(":{}\r\n", self).into_bytes()
    }
}

// - bulk string: "$<length>\r\n<data>\r\n"
impl RespEncoder for BulkString {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self.0);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

// - null bulk string: "$-1\r\n"
impl RespEncoder for NullBulkString {
    fn encode(self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
// - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
impl RespEncoder for RespArray {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUFFER_CAP);
        buf.extend_from_slice(&format!("*{}\r\n", self.0.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

// - null array: "*-1\r\n"
impl RespEncoder for RespNullArray {
    fn encode(self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

// - null: "_\r\n"
impl RespEncoder for RespNull {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

// - boolean: "#<t|f>\r\n"
impl RespEncoder for bool {
    fn encode(self) -> Vec<u8> {
        match self {
            true => b"#t\r\n".to_vec(),
            false => b"#f\r\n".to_vec(),
        }
    }
}

// - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
impl RespEncoder for f64 {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);
        let ret = if self.abs() > 1e+8 || self.abs() < 1e-8 {
            format!(",{:+e}\r\n", self)
        } else {
            format!(",{}\r\n", self)
        };

        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}

// - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespEncoder for RespMap {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUFFER_CAP);
        buf.extend_from_slice(&format!("%{}\r\n", self.len()).into_bytes());
        for (key, value) in self.0 {
            buf.extend_from_slice(&SimpleString::new(key).encode());
            buf.extend_from_slice(&value.encode());
        }
        buf
    }
}

// - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
impl RespEncoder for RespSet {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUFFER_CAP);
        buf.extend_from_slice(&format!("~{}\r\n", self.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RespFrame;

    #[test]
    fn test_encode_simple_string() {
        let frame: RespFrame = SimpleString::new("OK".to_string()).into();
        assert_eq!(frame.encode(), b"+OK\r\n");
    }

    #[test]
    fn test_encode_simple_error() {
        let frame: RespFrame = SimpleError::new("Error message".to_string()).into();
        assert_eq!(frame.encode(), b"-Error message\r\n")
    }

    #[test]
    fn test_encode_positive_integer() {
        let frame: RespFrame = 123.into();
        assert_eq!(frame.encode(), b":123\r\n");
    }

    #[test]
    fn test_encode_negative_integer() {
        let frame: RespFrame = (-123).into();
        assert_eq!(frame.encode(), b":-123\r\n");
    }

    #[test]
    fn test_encode_bulk_string() {
        let frame: RespFrame = BulkString::new("hello".to_string()).into();
        assert_eq!(frame.encode(), b"$5\r\nhello\r\n".to_vec());
    }

    #[test]
    fn test_encode_null_bulk_string() {
        let frame: RespFrame = NullBulkString.into();
        assert_eq!(frame.encode(), b"$-1\r\n".to_vec());
    }

    #[test]
    fn test_encode_array2() {
        let frame: RespFrame = RespArray::new(vec![
            BulkString::new("get".to_string()).into(),
            BulkString::new("hello".to_string()).into(),
        ])
        .into();
        assert_eq!(frame.encode(), b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n")
    }

    #[test]
    fn test_encode_array3() {
        let frame: RespFrame = RespArray::new(vec![
            BulkString::new("set".to_string()).into(),
            BulkString::new("hello".to_string()).into(),
            BulkString::new("world".to_string()).into(),
        ])
        .into();
        assert_eq!(
            &frame.encode(),
            b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
        );
    }

    #[test]
    fn test_encode_null_array_encode() {
        let frame: RespFrame = RespNullArray.into();
        assert_eq!(frame.encode(), b"*-1\r\n");
    }

    #[test]
    fn test_encode_null_encode() {
        let frame: RespFrame = RespNull.into();
        assert_eq!(frame.encode(), b"_\r\n");
    }

    #[test]
    fn test_encode_true_boolean() {
        let frame: RespFrame = true.into();
        assert_eq!(frame.encode(), b"#t\r\n");
    }

    #[test]
    fn test_encode_false_boolean() {
        let frame: RespFrame = false.into();
        assert_eq!(frame.encode(), b"#f\r\n");
    }

    #[test]
    fn test_encode_positive_double() {
        let frame: RespFrame = 123.456.into();
        assert_eq!(frame.encode(), b",123.456\r\n");

        let frame: RespFrame = 1.23456e+8.into();
        assert_eq!(frame.encode(), b",+1.23456e8\r\n");
    }

    #[test]
    fn test_encode_negative_double() {
        let frame: RespFrame = (-123.456).into();
        assert_eq!(frame.encode(), b",-123.456\r\n");

        let frame: RespFrame = (-1.23456e-9).into();
        assert_eq!(&frame.encode(), b",-1.23456e-9\r\n");
    }

    #[test]
    fn test_encode_map() {
        let mut map = RespMap::new();
        map.insert(
            "hello".to_string(),
            BulkString::new("world".to_string()).into(),
        );
        map.insert("foo".to_string(), (-123456.789).into());

        let frame: RespFrame = map.into();
        assert_eq!(
            String::from_utf8_lossy(&frame.encode()),
            "%2\r\n+foo\r\n,-123456.789\r\n+hello\r\n$5\r\nworld\r\n"
        );
    }

    #[test]
    fn test_encode_set() {
        let frame: RespFrame = RespSet::new([
            RespArray::new([1234.into(), true.into()]).into(),
            BulkString::new("world".to_string()).into(),
        ])
        .into();
        assert_eq!(
            frame.encode(),
            b"~2\r\n*2\r\n:1234\r\n#t\r\n$5\r\nworld\r\n"
        );
    }
}
