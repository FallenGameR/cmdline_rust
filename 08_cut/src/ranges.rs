use std::ops::RangeInclusive;

struct RangeIndex {
    ext: usize,
    int: Option<usize>,
}

struct RangeIter<'a> {
    ranges: &'a [RangeInclusive<usize>],
    index: RangeIndex,
}

impl<'a> RangeIter<'a> {
    fn new(ranges: &'a [RangeInclusive<usize>]) -> Self {
        Self {
            ranges,
            index: RangeIndex { ext: 0, int: None },
        }
    }
}

impl<'a> Iterator for RangeIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let range = self.ranges.get(self.index.ext)?;

        match self.index.int {
            None => {
                // Init the internal index
                self.index.int = Some(*range.start());
                self.next()
            }
            Some(value) => {
                // What is the next internal index?
                let next = if range.start() <= range.end() {
                    value.checked_add(1)
                } else {
                    value.checked_sub(1)
                };

                // Check if we need to move to another range
                if value == *range.end() || next.is_none() {
                    self.index = RangeIndex { ext: self.index.ext + 1, int: None };
                } else {
                    self.index.int = next;
                }

                Some(value)
            }
        }
    }
}

fn ranges_iter(ranges: &[RangeInclusive<usize>]) -> RangeIter {
    RangeIter::new(ranges)
}

/*
fn ranges_iter(ranges: &[RangeInclusive<usize>]) -> Box<dyn Iterator<Item = usize>> {
    let mut indexes = Vec::<usize>::new();

    for range in ranges {
        if range.start() <= range.end() {
            for index in range.clone() {
                indexes.push(index);
            }
        } else {
            let range = *range.end()..=*range.start();
            for index in range.rev() {
                indexes.push(index);
            }
        }
    }

    Box::new(indexes.into_iter())
}
*/