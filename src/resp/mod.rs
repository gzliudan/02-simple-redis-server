mod decoder;
mod encoder;

use enum_dispatch::enum_dispatch;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

#[enum_dispatch]
pub trait RespEncoder {
    fn encode(self) -> Vec<u8>;
}

pub trait RespDecoder {
    fn decode(data: &[u8]) -> Result<RespFrame, String>;
}

#[enum_dispatch(RespEncoder)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(BulkString),
    NullBulkString(NullBulkString),
    Array(RespArray),
    NullArray(RespNullArray),
    Null(RespNull),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SimpleString(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SimpleError(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct BulkString(Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNull;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespArray(Vec<RespFrame>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNullArray;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct NullBulkString;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespMap(BTreeMap<String, RespFrame>);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespSet(Vec<RespFrame>);

impl Deref for SimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SimpleError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for RespSet {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleString {
    pub fn new(value: impl Into<String>) -> Self {
        SimpleString(value.into())
    }
}

impl SimpleError {
    pub fn new(value: impl Into<String>) -> Self {
        SimpleError(value.into())
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}

impl RespMap {
    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }
}

impl Default for RespMap {
    fn default() -> Self {
        Self::new()
    }
}

impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(s.into())
    }
}
