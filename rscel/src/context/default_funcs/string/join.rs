use crate::{CelError, CelValue};

pub fn join(this: CelValue, args: Vec<CelValue>) -> CelValue {
    let items = match this {
        CelValue::List(ref l) => l.as_slice(),
        _ => return CelValue::from_err(CelError::value("join() only available on list")),
    };

    let separator = match args.first() {
        Some(CelValue::String(s)) => s.as_str(),
        None => "",
        _ => {
            return CelValue::from_err(CelError::argument(
                "join() separator must be a string",
            ))
        }
    };

    let parts: Result<Vec<&str>, _> = items
        .iter()
        .map(|v| match v {
            CelValue::String(s) => Ok(s.as_str()),
            _ => Err(CelError::value("join() requires all list elements to be strings")),
        })
        .collect();

    match parts {
        Ok(p) => CelValue::String(p.join(separator)),
        Err(e) => CelValue::from_err(e),
    }
}
