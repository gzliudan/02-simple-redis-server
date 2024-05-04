use bytes::{Buf, BytesMut};
use std::ops::Deref;

use super::{
    calc_total_length, parse_length, RespDecoder, RespEncoder, RespError, RespFrame, BUFFER_CAP,
    CRLF_LEN,
};

const NULL_ARRAY: &[u8] = b"*-1\r\n";

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
// - null array: "*-1\r\n"
// - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
impl RespEncoder for RespArray {
    fn encode(self) -> Vec<u8> {
        if self.is_empty() {
            return NULL_ARRAY.to_vec();
        }

        let mut buf = Vec::with_capacity(BUFFER_CAP);
        buf.extend_from_slice(&format!("*{}\r\n", self.0.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
// - null array: "*-1\r\n"
// - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
// FIXME: need to handle incomplete
impl RespDecoder for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.starts_with(NULL_ARRAY) {
            buf.advance(NULL_ARRAY.len());
            return Ok(RespArray::new(vec![]));
        }

        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            frames.push(RespFrame::decode(buf)?);
        }

        Ok(RespArray::new(frames))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BulkString, RespFrame};
    use anyhow::Result;

    #[test]
    fn test_encode_null_array_encode() {
        let frame: RespFrame = RespArray::new(vec![]).into();
        assert_eq!(frame.encode(), NULL_ARRAY);
    }

    #[test]
    fn test_decode_null_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(NULL_ARRAY);
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new(vec![]));
        Ok(())
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
    fn test_decode_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
        let ret = RespArray::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        Ok(())
    }

    #[test]
    fn test_calc_array_length() -> Result<()> {
        let buf = b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n";
        let (end, len) = parse_length(buf, "*")?;
        let total_len = calc_total_length(buf, end, len, "*")?;
        assert_eq!(total_len, buf.len());

        let buf = b"*2\r\n$3\r\nset\r\n";
        let (end, len) = parse_length(buf, "*")?;
        let ret = calc_total_length(buf, end, len, "*");
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        Ok(())
    }
}
