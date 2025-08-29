use core::hint::unlikely;
use crossbeam_utils::CachePadded;
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A lock-free, single-producer, single-consumer queue.
#[allow(dead_code)]
pub struct Spsc<T, const N: usize> {
    head: CachePadded<AtomicUsize>,
    tail: CachePadded<AtomicUsize>,
    buffer: [UnsafeCell<MaybeUninit<T>>; N],
}

#[allow(dead_code)]
impl<T, const N: usize> Spsc<T, N> {
    pub fn new() -> Self {
        Self {
            head: CachePadded::new(AtomicUsize::new(0)),
            tail: CachePadded::new(AtomicUsize::new(0)),
            buffer: std::array::from_fn(|_| UnsafeCell::new(MaybeUninit::uninit())),
        }
    }

    #[inline]
    fn next_index(&self, index: usize) -> usize {
        // TODO: Check if unlikely work in the elf
        if unlikely(index == N - 1) {
            0
        } else {
            index + 1
        }
    }

    /// Pushes a value onto the queue.
    ///
    /// This operation is lock-free and only safe to be called from the single producer.
    pub fn produce(&self, val: T) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = self.next_index(tail);

        if next_tail == self.head.load(Ordering::Acquire) {
            return false;
        }

        // Safety
        // This is safe because:
        // 1. We've checked that the queue is not full, so `tail` is a valid slot.
        // 2. We are the single producer, so no other thread is writing to this slot.
        unsafe {
            let slot = self.buffer.get_unchecked(tail);
            (*slot.get()).write(val);
        }

        self.tail.store(next_tail, Ordering::Release);

        true
    }

    /// Pops a value from the queue.
    ///
    /// This operation is lock-free and only safe to be called from the single consumer.
    pub fn consume(&self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);
        let next_head = self.next_index(head);

        if head == self.tail.load(Ordering::Acquire) {
            return None;
        }

        // Safety
        // This is safe because:
        // 1. We've checked that the queue is not empty.
        // 2. We are the single consumer, so no other thread is reading from this slot.
        let value = unsafe {
            let slot = self.buffer.get_unchecked(head);
            (*slot.get()).assume_init_read()
        };

        self.head.store(next_head, Ordering::Release);

        Some(value)
    }
}

// Safety
// This is safe because the SPSC queue is designed to be used by a single producer and a single consumer.
// The producer only ever writes to the `tail` and the consumer only ever reads from the `head`.
// The atomic operations on `head` and `tail` ensure that the producer and consumer never access the same slot at the same time.
unsafe impl<T: Send, const N: usize> Sync for Spsc<T, N> {}
