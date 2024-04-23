use crate::BindContext;

pub struct IdentFilterIter<'a> {
    bindings: &'a BindContext<'a>,
    iter: &'a mut dyn Iterator<Item = &'a str>,
}

impl<'a> IdentFilterIter<'a> {
    pub fn new(
        bindings: &'a BindContext,
        iterable: &'a mut dyn Iterator<Item = &'a str>,
    ) -> IdentFilterIter<'a> {
        IdentFilterIter {
            bindings,
            iter: iterable,
        }
    }
}

impl<'a> Iterator for IdentFilterIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(val) => {
                    if self.bindings.is_bound(val) {
                        continue;
                    }

                    return Some(val.to_owned());
                }
                None => return None,
            }
        }
    }
}
