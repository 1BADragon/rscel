use std::ops::Index;

use serde::{Deserialize, Serialize};

use crate::interp::ByteCode;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CelByteCode {
    inner: Vec<ByteCode>,
}

impl CelByteCode {
    pub fn new() -> Self {
        CelByteCode { inner: Vec::new() }
    }

    pub fn from_code_point(code_point: ByteCode) -> Self {
        CelByteCode {
            inner: vec![code_point],
        }
    }

    pub fn from_vec(code_points: Vec<ByteCode>) -> Self {
        CelByteCode { inner: code_points }
    }

    pub fn extend<T>(&mut self, items: T)
    where
        T: IntoIterator<Item = ByteCode>,
    {
        self.inner.extend(items);
    }

    pub fn push(&mut self, code_point: ByteCode) {
        self.inner.push(code_point);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ByteCode> {
        self.inner.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = ByteCode> {
        self.inner.into_iter()
    }

    pub fn as_slice(&self) -> &[ByteCode] {
        self.inner.as_slice()
    }
}

impl From<Vec<ByteCode>> for CelByteCode {
    fn from(value: Vec<ByteCode>) -> Self {
        CelByteCode { inner: value }
    }
}

impl Into<Vec<ByteCode>> for CelByteCode {
    fn into(self) -> Vec<ByteCode> {
        self.inner
    }
}

impl Index<usize> for CelByteCode {
    type Output = ByteCode;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl FromIterator<ByteCode> for CelByteCode {
    fn from_iter<T: IntoIterator<Item = ByteCode>>(iter: T) -> Self {
        CelByteCode {
            inner: iter.into_iter().collect(),
        }
    }
}
