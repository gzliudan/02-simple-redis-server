use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use thiserror::Error;

use crate::{Backend, RespArray, RespError, RespFrame, SimpleString};

mod hmap;
mod hset;
mod map;

lazy_static! {
    static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("{0}")]
    RespError(#[from] RespError),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

#[enum_dispatch]
pub trait CommandExecutor {
    fn execute(self, backend: &Backend) -> RespFrame;
}

#[enum_dispatch(CommandExecutor)]
#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    Echo(Echo),
    HGet(HGet),
    HSet(HSet),
    HGetAll(HGetAll),
    HMGet(HMGet),
    SAdd(SAdd),
    SIsMember(SIsMember),

    // unrecognized command
    Unrecognized(Unrecognized),
}

#[derive(Debug)]
pub struct Get {
    key: String,
}

#[derive(Debug)]
pub struct Set {
    key: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct Echo {
    message: String,
}

#[derive(Debug)]
pub struct HGet {
    key: String,
    field: String,
}

#[derive(Debug)]
pub struct HSet {
    key: String,
    field: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct HGetAll {
    key: String,
    sort: bool,
}

// HSET myhash field1 "Hello"
// HGET myhash field1
// HSET myhash field2 "World"
// HGET myhash field2
// HMGET myhash field1 field2
// HMGET myhash field1 field2 nofield
// "*5\r\n$5\r\nHMGET\r\n$6\r\nmyhash\r\n$6\r\nfield1\r\n$6\r\nfield2\r\n$7\r\nnofield\r\n"
#[derive(Debug)]
pub struct HMGet {
    hash: String,
    fields: Vec<String>,
}

// SADD key member [member ...]
// SADD myset "Hello": "*3\r\n$4\r\nSADD\r\n$5\r\nmyset\r\n$5\r\nHello\r\n"
// SADD myset "World": "*3\r\n$4\r\nSADD\r\n$5\r\nmyset\r\n$5\r\nWorld\r\n"
// SADD myset "Hello" "World": "*4\r\n$4\r\nSADD\r\n$5\r\nmyset\r\n$5\r\nHello\r\n$5\r\nWorld\r\n"
// redis> SADD myset "Hello"
// (integer) 1
// redis> SADD myset "Hello"
// (integer) 0
#[derive(Debug)]
pub struct SAdd {
    key: String,
    members: Vec<String>,
}

// SISMEMBER key member
// SISMEMBER myset "one": "*3\r\n$9\r\nSISMEMBER\r\n$5\r\nmyset\r\n$3\r\none\r\n"
// SISMEMBER myset "two": "*3\r\n$9\r\nSISMEMBER\r\n$5\r\nmyset\r\n$3\r\ntwo\r\n"
// redis> SADD myset "one"
// (integer) 1
// redis> SISMEMBER myset "one"
// (integer) 1
// redis> SISMEMBER myset "two"
// (integer) 0
#[derive(Debug)]
pub struct SIsMember {
    key: String,
    member: String,
}

#[derive(Debug)]
pub struct Unrecognized;

impl TryFrom<RespFrame> for Command {
    type Error = CommandError;
    fn try_from(v: RespFrame) -> Result<Self, Self::Error> {
        match v {
            RespFrame::Array(array) => array.try_into(),
            _ => Err(CommandError::InvalidCommand(
                "Command must be an Array".to_string(),
            )),
        }
    }
}

impl CommandExecutor for Unrecognized {
    fn execute(self, _: &Backend) -> RespFrame {
        RESP_OK.clone()
    }
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;
    fn try_from(v: RespArray) -> Result<Self, Self::Error> {
        match v.first() {
            Some(RespFrame::BulkString(ref cmd)) => {
                match cmd.as_ref().to_ascii_lowercase().as_slice() {
                    b"get" => Ok(Get::try_from(v)?.into()),
                    b"set" => Ok(Set::try_from(v)?.into()),
                    b"echo" => Ok(Echo::try_from(v)?.into()),
                    b"hget" => Ok(HGet::try_from(v)?.into()),
                    b"hset" => Ok(HSet::try_from(v)?.into()),
                    b"hmget" => Ok(HMGet::try_from(v)?.into()),
                    b"hgetall" => Ok(HGetAll::try_from(v)?.into()),
                    b"sadd" => Ok(SAdd::try_from(v)?.into()),
                    b"sismember" => Ok(SIsMember::try_from(v)?.into()),
                    _ => Ok(Unrecognized.into()),
                }
            }
            _ => Err(CommandError::InvalidCommand(
                "Command must have a BulkString as the first argument".to_string(),
            )),
        }
    }
}

fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    if value.len() != n_args + names.len() {
        return Err(CommandError::InvalidArgument(format!(
            "{} command must have exactly {} argument",
            names.join(" "),
            n_args
        )));
    }

    for (i, name) in names.iter().enumerate() {
        match value[i] {
            RespFrame::BulkString(ref cmd) => {
                if cmd.as_ref().to_ascii_lowercase() != name.as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "Invalid command: expected {}, got {}",
                        name,
                        String::from_utf8_lossy(cmd.as_ref())
                    )));
                }
            }
            _ => {
                return Err(CommandError::InvalidCommand(
                    "Command must have a BulkString as the first argument".to_string(),
                ))
            }
        }
    }
    Ok(())
}

fn extract_args(value: RespArray, start: usize) -> Result<Vec<RespFrame>, CommandError> {
    Ok(value.0.into_iter().skip(start).collect::<Vec<RespFrame>>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RespDecoder, RespNull};
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_lowercase_get() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let cmd: Command = frame.try_into()?;
        let backend = Backend::new();
        let ret = cmd.execute(&backend);
        assert_eq!(ret, RespFrame::Null(RespNull));
        Ok(())
    }

    #[test]
    fn test_uppercase_get() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nGET\r\n$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let cmd: Command = frame.try_into()?;
        let backend = Backend::new();
        let ret = cmd.execute(&backend);
        assert_eq!(ret, RespFrame::Null(RespNull));
        Ok(())
    }
}
