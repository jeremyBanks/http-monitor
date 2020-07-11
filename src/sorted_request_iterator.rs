use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    collections::{BinaryHeap, VecDeque},
    iter::{Enumerate, FusedIterator},
    rc::Rc,
};

use crate::{Config, RequestRecord};

/// Wraps an iterator of RequestRecords with a buffer to allow the records to be sorted.
///
/// Panics if a request has a timestamp more than buffer_seconds less than the
/// greatest previous timestamp.
pub struct SortedRequestIterator<T: Iterator<Item = Rc<RequestRecord>>> {
    /// The iterator being wrapped.
    iterator: Enumerate<T>,
    /// Records whose order is now known, but haven't yet been read out of this iterator.
    /// (Timestamps will be more than buffer_seconds before largest_timestamp.)
    sorted: VecDeque<Rc<RequestRecord>>,
    /// Records whose order is still indeterminate.
    /// (Timestamps will be within buffer_seconds of largest_timestamp.)
    unsorted: BinaryHeap<ChronologicalRecord>,
    /// The number of seconds of logs to buffer.
    buffer_seconds: u32,
    /// The maximum timestamp from records we've read from the iterator so far.
    largest_timestamp: u32,
}

/// A request record wrapped to sort by its date then an index.
///
/// Since BinaryHeap is a max heap, this sorts the values we want first, last.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ChronologicalRecord {
    record: Rc<RequestRecord>,
    index: usize,
}

impl PartialOrd for ChronologicalRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChronologicalRecord {
    fn cmp(&self, other: &Self) -> Ordering {
        (other.record.date, other.index).cmp(&(self.record.date, self.index))
    }
}

impl<T: Iterator<Item = Rc<RequestRecord>>> SortedRequestIterator<T> {
    pub fn new(iterator: T, config: Config) -> Self {
        Self {
            iterator: iterator.fuse(),
            buffer_seconds: 2 * config.maximum_timestamp_error,
            largest_timestamp: 0,
            sorted: VecDeque::new(),
            unsorted: BinaryHeap::new(),
        }
    }
}

impl<T: Iterator<Item = Rc<RequestRecord>>> Iterator for SortedRequestIterator<T> {
    type Item = Rc<RequestRecord>;

    fn next(&mut self) -> Option<Rc<RequestRecord>> {
        if let Some(record) = self.sorted.pop_front() {
            Some(record)
        } else if let Some((index, record)) = self.iterator.next() {
            if record.date > self.largest_timestamp {
            } else {
                self.unsorted.push(ChronologicalRecord { record, index })
            }

            if record.date > self.largest_timestamp + self.buffer_seconds {}

            assert!(record.date > self.buffer_minimum_timestamp);

            if record.date == self.buffer_minimum_timestamp + 1 {
                // This record has the minimum possible
                return Some(record);
            }

            self.buffer_minimum_timestamp = record.date - self.buffer_seconds;
            Some(record)
        } else {
            self.buffer.pop().map(|x| x.record)
        }
    }
}

impl<T: Iterator<Item = Rc<RequestRecord>>> FusedIterator for SortedRequestIterator<T> {}
