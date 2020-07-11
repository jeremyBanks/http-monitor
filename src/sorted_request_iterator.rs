use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    collections::{BinaryHeap, VecDeque},
    iter::{Enumerate, FusedIterator},
    rc::Rc,
};

use itertools::Itertools;

use crate::{Config, RequestRecord};

/// Wraps an iterator of RequestRecords with a buffer to allow the records to be sorted.
///
/// Panics if a request has a timestamp more than buffer_seconds less than the
/// greatest previous timestamp.
pub struct SortedRequestIterator<T: Iterator<Item = RequestRecord>> {
    /// The iterator being wrapped.
    iterator: Enumerate<T>,
    /// Records whose order is now known, but haven't yet been read out of this iterator.
    /// (Timestamps will be more than buffer_seconds before largest_timestamp.)
    sorted: VecDeque<RequestRecord>,
    /// Records whose order is still indeterminate.
    /// (Timestamps will be within buffer_seconds of largest_timestamp.)
    unsorted: BinaryHeap<ChronologicalRecord>,
    /// The number of seconds of logs to buffer.
    buffer_seconds: u32,
    /// The maximum timestamp from records we've read from the iterator so far.
    largest_timestamp: u32,
}

/// A request record wrapped to sort by its date then (for stability) an index.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ChronologicalRecord {
    record: RequestRecord,
    index: usize,
}

impl PartialOrd for ChronologicalRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChronologicalRecord {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.record.date, self.index).cmp(&(other.record.date, other.index))
    }
}

impl<T: Iterator<Item = RequestRecord>> SortedRequestIterator<T> {
    pub fn new(iterator: T, config: &Config) -> Self {
        Self {
            iterator: iterator.enumerate(),
            buffer_seconds: 2 * config.maximum_timestamp_error,
            largest_timestamp: 0,
            sorted: VecDeque::new(),
            unsorted: BinaryHeap::new(),
        }
    }
}

impl<T: Iterator<Item = RequestRecord>> Iterator for SortedRequestIterator<T> {
    type Item = RequestRecord;

    fn next(&mut self) -> Option<RequestRecord> {
        while self.sorted.is_empty() {
            let next = self.iterator.next();

            if next.is_none() {
                // If there are no more items in the source iterator,
                if self.unsorted.is_empty() {
                    // then if there are also no unsorted items, this iterator is also exhausted.
                    return None;
                } else {
                    // otherwise we can now take the remaining unsorted items and sort them.
                    for record in self.unsorted.drain().sorted() {
                        self.sorted.push_back(record.record);
                    }
                    break;
                }
            }

            let (index, record) = next.unwrap();

            let buffer_advanced = record.date > self.largest_timestamp;
            if buffer_advanced {
                self.largest_timestamp = record.date;
            }

            self.unsorted.push(ChronologicalRecord { record, index });

            if buffer_advanced {
                while self.unsorted.peek().unwrap().record.date
                    < self.largest_timestamp - self.buffer_seconds
                {
                    self.sorted.push_back(self.unsorted.pop().unwrap().record);
                }
            }
        }

        self.sorted.pop_front()
    }
}

impl<T: Iterator<Item = RequestRecord>> FusedIterator for SortedRequestIterator<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_iter() {
        let results: Vec<_> =
            SortedRequestIterator::new(Vec::new().into_iter(), &Config::default()).collect();
        assert_eq!(results, vec![]);
    }
}
