use crate::macros::dispatch;

pub use methods::dispatch as size;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn size(this: String) -> u64 {
        this.len() as u64
    }

    fn size(this: Vec<u8>) -> u64 {
        this.len() as u64
    }

    fn size(this: Vec<CelValue>) -> u64 {
        this.len() as u64
    }

    fn size(arg: String) -> u64 {
        arg.len() as u64
    }

    fn size(arg: Vec<u8>) -> u64 {
        arg.len() as u64
    }

    fn size(arg: Vec<CelValue>) -> u64 {
        arg.len() as u64
    }
}
