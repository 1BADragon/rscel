use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct CelBytes {
    inner: Vec<u8>,
}

impl CelBytes {
    pub fn new() -> Self {
        CelBytes { inner: Vec::new() }
    }

    pub fn from_vec(bytes: Vec<u8>) -> Self {
        CelBytes { inner: bytes }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.inner
    }

    pub fn extend<T>(&mut self, bytes: T)
    where
        T: IntoIterator<Item = u8>,
    {
        self.inner.extend(bytes.into_iter());
    }

    pub fn as_slice(&self) -> &[u8] {
        self.inner.as_ref()
    }
}

impl From<Vec<u8>> for CelBytes {
    fn from(value: Vec<u8>) -> Self {
        CelBytes::from_vec(value)
    }
}

impl Into<Vec<u8>> for CelBytes {
    fn into(self) -> Vec<u8> {
        self.inner
    }
}

impl IntoIterator for CelBytes {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
