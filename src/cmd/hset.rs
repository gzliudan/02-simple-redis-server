use super::{extract_args, validate_command, CommandError, CommandExecutor, SAdd, SIsMember};
use crate::{RespArray, RespFrame};

impl CommandExecutor for SAdd {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        let response = self
            .members
            .into_iter()
            .map(|f| backend.sadd(self.key.clone(), f))
            .map(|b| RespFrame::Integer(b as i64))
            .collect();
        RespFrame::Array(RespArray(response))
    }
}

impl CommandExecutor for SIsMember {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        RespFrame::Integer(backend.sismember(&self.key, &self.member) as i64)
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

impl TryFrom<RespArray> for SIsMember {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sismember"], 2)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(member))) => {
                Ok(SIsMember {
                    key: String::from_utf8(key.0)?,
                    member: String::from_utf8(member.0)?,
                })
            }
            _ => Err(CommandError::InvalidArgument(
                "Invalid key or value".to_string(),
            )),
        }
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

    #[test]
    fn test_sismember_from_resp() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$9\r\nSISMEMBER\r\n$5\r\nmyset\r\n$3\r\none\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let result: SIsMember = frame.try_into()?;
        assert_eq!(result.key, "myset");
        assert_eq!(result.member, "one");
        Ok(())
    }
}
