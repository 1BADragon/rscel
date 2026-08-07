use base64::{engine::general_purpose::STANDARD, Engine};
use crate::{CelError, CelValue};

pub fn encode(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let bytes = match this {
        CelValue::Bytes(ref b) if args.is_empty() => b.as_slice().to_vec(),
        _ => match args.first() {
            Some(CelValue::Bytes(b)) => b.as_slice().to_vec(),
            _ => return CelValue::from_err(CelError::argument("encode() requires a bytes argument")),
        },
    };
    CelValue::String(STANDARD.encode(&bytes))
}

pub fn decode(this: CelValue, args: Vec<CelValue>) -> CelValue {
    use base64::engine::general_purpose::STANDARD_NO_PAD;

    let s = match this {
        CelValue::String(ref s) if args.is_empty() => s.clone(),
        _ => match args.first() {
            Some(CelValue::String(s)) => s.clone(),
            _ => return CelValue::from_err(CelError::argument("decode() requires a string argument")),
        },
    };
    match STANDARD.decode(&s) {
        Ok(bytes) => CelValue::from_bytes(bytes),
        Err(_) => match STANDARD_NO_PAD.decode(&s) {
            Ok(bytes) => CelValue::from_bytes(bytes),
            Err(e) => CelValue::from_err(CelError::value(&format!("base64 decode error: {e}"))),
        },
    }
}
