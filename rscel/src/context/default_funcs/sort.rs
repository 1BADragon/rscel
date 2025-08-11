use rscel_macro::dispatch;

pub use methods::dispatch as sort;

#[dispatch]
mod methods {
    use crate::CelValue;

    fn sort(mut this: Vec<CelValue>) -> Vec<CelValue> {
        this.sort_by(|a, b| {
            a.clone()
                .ord(b.clone())
                .unwrap_or(Some(std::cmp::Ordering::Less))
                .unwrap_or(std::cmp::Ordering::Less)
        });
        this
    }

    mod internal {}
}
