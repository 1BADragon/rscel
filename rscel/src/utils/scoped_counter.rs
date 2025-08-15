use std::cell::Cell;

pub struct ScopedCounter {
    count: Cell<usize>,
}

pub struct ScopedCounterRef<'a> {
    count: &'a Cell<usize>,
}

impl ScopedCounter {
    pub fn new() -> ScopedCounter {
        ScopedCounter {
            count: Cell::new(0),
        }
    }

    pub fn count(&self) -> usize {
        self.count.get()
    }

    pub fn inc<'a>(&'a self) -> ScopedCounterRef<'a> {
        let count = self.count.get();
        self.count.set(count + 1);
        ScopedCounterRef { count: &self.count }
    }
}

impl<'a> ScopedCounterRef<'a> {
    pub fn count(&self) -> usize {
        self.count.get()
    }
}

impl<'a> Drop for ScopedCounterRef<'a> {
    fn drop(&mut self) {
        let count = self.count.get();
        self.count.set(count - 1);
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
