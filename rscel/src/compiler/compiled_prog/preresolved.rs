use std::{collections::HashMap, ops::Index};

use crate::{interp::JmpWhen, types::CelByteCode, ByteCode};

#[derive(Debug, Clone, PartialEq)]
pub enum PreResolvedCodePoint {
    Bytecode(ByteCode),
    Jmp { label: u32 },
    JmpCond { when: JmpWhen, label: u32 },
    Label(u32),
}

#[derive(Debug, Clone)]
pub struct PreResolvedByteCode {
    inner: Vec<PreResolvedCodePoint>,
    len: usize,
}

impl From<ByteCode> for PreResolvedCodePoint {
    fn from(value: ByteCode) -> Self {
        PreResolvedCodePoint::Bytecode(value)
    }
}

impl From<CelByteCode> for Vec<PreResolvedCodePoint> {
    fn from(value: CelByteCode) -> Self {
        value.into_iter().map(|b| b.into()).collect()
    }
}

impl PreResolvedByteCode {
    pub fn new() -> Self {
        PreResolvedByteCode {
            inner: Vec::new(),
            len: 0,
        }
    }

    pub fn push(&mut self, val: impl Into<PreResolvedCodePoint>) {
        let v = val.into();
        self.inner.push(v);
    }

    pub fn extend(&mut self, byte_codes: impl IntoIterator<Item = PreResolvedCodePoint>) {
        for b in byte_codes.into_iter() {
            match &b {
                PreResolvedCodePoint::Label(_) => {}
                _ => self.len += 1,
            }

            self.inner.push(b)
        }
    }

    pub fn into_iter(self) -> impl Iterator<Item = PreResolvedCodePoint> {
        self.inner.into_iter()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn resolve(self) -> CelByteCode {
        let mut curr_loc: usize = 0;
        let mut locations = HashMap::<u32, usize>::new();
        let mut ret = CelByteCode::new();

        // determine label locations
        for c in self.inner.iter() {
            match c {
                PreResolvedCodePoint::Label(i) => {
                    if locations.contains_key(i) {
                        panic!("Duplicate label found!");
                    }
                    locations.insert(*i, curr_loc);
                }
                _ => {
                    curr_loc += 1;
                }
            }
        }

        curr_loc = 0;

        // resolve the label locations
        for c in self.inner.into_iter() {
            match c {
                PreResolvedCodePoint::Bytecode(byte_code) => {
                    curr_loc += 1;
                    ret.push(byte_code);
                }
                PreResolvedCodePoint::Jmp { label } => {
                    curr_loc += 1;
                    let jmp_loc = locations[&label];
                    let offset = (jmp_loc as isize) - (curr_loc as isize);
                    ret.push(ByteCode::Jmp(
                        i32::try_from(offset).expect("Attempt to jump farther than possible"),
                    ));
                }
                PreResolvedCodePoint::JmpCond { when, label } => {
                    curr_loc += 1;
                    let jmp_loc = locations[&label];
                    let offset = (jmp_loc as isize) - (curr_loc as isize);
                    ret.push(ByteCode::JmpCond {
                        when,
                        dist: offset as i32,
                    });
                }
                PreResolvedCodePoint::Label(_) => {}
            }
        }

        ret
    }
}

impl Index<usize> for PreResolvedByteCode {
    type Output = PreResolvedCodePoint;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl From<CelByteCode> for PreResolvedByteCode {
    fn from(value: CelByteCode) -> Self {
        value.into_iter().collect()
    }
}

impl FromIterator<ByteCode> for PreResolvedByteCode {
    fn from_iter<T: IntoIterator<Item = ByteCode>>(iter: T) -> Self {
        let v: Vec<_> = iter.into_iter().map(|b| b.into()).collect();
        let l = v.len();

        PreResolvedByteCode { inner: v, len: l }
    }
}

impl FromIterator<PreResolvedCodePoint> for PreResolvedByteCode {
    fn from_iter<T: IntoIterator<Item = PreResolvedCodePoint>>(iter: T) -> Self {
        let mut code_points = Vec::new();
        let mut size = 0;

        for code_point in iter.into_iter() {
            match &code_point {
                PreResolvedCodePoint::Label(_) => {}
                _ => {
                    size += 1;
                }
            }

            code_points.push(code_point);
        }

        PreResolvedByteCode {
            inner: code_points,
            len: size,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{types::CelByteCode, ByteCode, CelValue};

    use super::{PreResolvedByteCode, PreResolvedCodePoint};

    #[test]
    fn test_basic() {
        let mut code = PreResolvedByteCode::new();

        code.extend([
            PreResolvedCodePoint::Label(0),
            PreResolvedCodePoint::Bytecode(ByteCode::Push(2.into())),
            PreResolvedCodePoint::Jmp { label: 0 },
        ]);

        let resolved = code.resolve();
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0], ByteCode::Push(CelValue::Int(2)));
        assert_eq!(resolved[1], ByteCode::Jmp(-2));
    }

    #[test]
    fn test_from_bytecode() {
        let r: Vec<PreResolvedCodePoint> =
            [ByteCode::Test].into_iter().collect::<CelByteCode>().into();

        assert_eq!(r.len(), 1)
    }

    #[test]
    #[should_panic]
    fn test_dup_label_panics() {
        let mut code = PreResolvedByteCode::new();

        code.extend([
            PreResolvedCodePoint::Label(0),
            PreResolvedCodePoint::Bytecode(ByteCode::Push(2.into())),
            PreResolvedCodePoint::Jmp { label: 0 },
            PreResolvedCodePoint::Label(0),
        ]);

        code.resolve();
    }
}
