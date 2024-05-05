use super::{extract_args, validate_command, CommandError, CommandExecutor, SAdd};
use crate::{RespArray, RespFrame};

impl CommandExecutor for SAdd {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        let response = self
            .members
            .into_iter()
            .map(|f| match backend.sadd(self.key.clone(), f) {
                true => RespFrame::Integer(1),
                false => RespFrame::Integer(0),
            })
            .collect();
        RespFrame::Array(RespArray(response))
    }
}

impl TryFrom<RespArray> for SAdd {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let len = value.len();
        match len {
            0 => {
                return Err(CommandError::InvalidCommand(
                    "sadd command does not accept null array".to_string(),
                ))
            }
            1..=2 => {
                return Err(CommandError::InvalidCommand(format!(
                    "sadd command needs at least 2 argument, got {len}",
                )))
            }
            _ => validate_command(&value, &["sadd"], len - 1)?,
        }

        let mut args = extract_args(value, 1)?.into_iter();
        let key = match args.next() {
            Some(RespFrame::BulkString(key)) => String::from_utf8(key.0)?,
            _ => return Err(CommandError::InvalidArgument("Invalid key".to_string())),
        };
        let mut members = vec![];
        loop {
            match args.next() {
                Some(RespFrame::BulkString(key)) => members.push(String::from_utf8(key.0)?),
                None => break,
                _ => return Err(CommandError::InvalidArgument("Invalid key".to_string())),
            };
        }
        Ok(SAdd { key, members })
    }
}

#[cfg(test)]
mod tests {
    use crate::RespDecoder;

    use super::*;
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_sadd_from_resp_array2() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$4\r\nSADD\r\n$5\r\nmyset\r\n$5\r\nHello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let result: SAdd = frame.try_into()?;
        assert_eq!(result.key, "myset");
        assert_eq!(result.members.len(), 1);
        assert_eq!(result.members[0], "Hello");

        Ok(())
    }

    #[test]
    fn test_sadd_from_resp_array3() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*4\r\n$4\r\nSADD\r\n$5\r\nmyset\r\n$5\r\nHello\r\n$5\r\nWorld\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let result: SAdd = frame.try_into()?;
        assert_eq!(result.key, "myset");
        assert_eq!(result.members.len(), 2);
        assert_eq!(result.members[0], "Hello");
        assert_eq!(result.members[1], "World");
        Ok(())
    }
}
