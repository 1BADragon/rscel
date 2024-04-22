use std::cell::RefCell;

pub struct ScopedCounter {
    count: RefCell<usize>,
}

pub struct ScopedCounterRef<'a> {
    count: &'a RefCell<usize>,
}

impl ScopedCounter {
    pub fn new() -> ScopedCounter {
        ScopedCounter {
            count: RefCell::new(0),
        }
    }

    pub fn count(&self) -> usize {
        *self.count.borrow()
    }

    pub fn inc<'a>(&'a self) -> ScopedCounterRef<'a> {
        {
            let mut count = self.count.borrow_mut();

            *count += 1;
        }
        ScopedCounterRef { count: &self.count }
    }
}

impl<'a> ScopedCounterRef<'a> {
    pub fn count(&self) -> usize {
        *self.count.borrow()
    }
}

impl<'a> Drop for ScopedCounterRef<'a> {
    fn drop(&mut self) {
        let mut count = self.count.borrow_mut();

        *count -= 1;
    }
}

#[cfg(test)]
mod test {
    use super::ScopedCounter;

    #[test]
    fn test_scoped_counter() {
        let counter = ScopedCounter::new();

        assert_eq!(counter.count(), 0);

        {
            let c = counter.inc();

            assert_eq!(c.count(), 1);
            assert_eq!(counter.count(), 1);
        }

        assert_eq!(counter.count(), 0);
    }
}
