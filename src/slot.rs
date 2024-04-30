use core::mem::MaybeUninit;
use core::sync::atomic::Ordering;
#[cfg(not(loom))]
use core::{cell::UnsafeCell, sync::atomic::AtomicBool};

#[cfg(loom)]
use loom::{cell::UnsafeCell, sync::atomic::AtomicBool};

pub struct Slot<T> {
    has_element: AtomicBool,
    buffer: UnsafeCell<MaybeUninit<T>>,
}

impl<T> Slot<T> {
    #[cfg(not(loom))]
    pub const fn new() -> Self {
        Slot {
            has_element: AtomicBool::new(false),
            buffer: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
    #[cfg(loom)]
    pub fn new() -> Self {
        Slot {
            has_element: AtomicBool::new(false),
            buffer: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Returns `true` if the slot is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        !self.has_element.load(Ordering::Relaxed)
    }

    /// Adds an `item` to the slot
    ///
    /// Returns back the `item` if the slot is full
    #[inline]
    pub fn enqueue(&mut self, val: T) -> Result<(), T> {
        unsafe { self.inner_enqueue(val) }
    }

    /// Returns the item in the slot, or `None` if the slot is empty
    #[inline]
    pub fn dequeue(&mut self) -> Option<T> {
        unsafe { self.inner_dequeue() }
    }

    unsafe fn inner_enqueue(&self, val: T) -> Result<(), T> {
        if !self.has_element.load(Ordering::Acquire) {
            #[cfg(not(loom))]
            self.buffer.get().write(MaybeUninit::new(val));
            #[cfg(loom)]
            self.buffer.with_mut(|p| p.write(MaybeUninit::new(val)));
            self.has_element.store(true, Ordering::Release);

            Ok(())
        } else {
            Err(val)
        }
    }

    unsafe fn inner_dequeue(&self) -> Option<T> {
        if self.has_element.load(Ordering::Acquire) {
            #[cfg(not(loom))]
            let v = (self.buffer.get() as *const T).read();
            #[cfg(loom)]
            let v = self.buffer.with(|p| (p as *const T).read());

            self.has_element.store(false, Ordering::Release);

            Some(v)
        } else {
            None
        }
    }

    /// Splits a slot into producer and consumer endpoints
    pub fn split(&mut self) -> (Producer<'_, T>, Consumer<'_, T>) {
        (Producer { rb: self }, Consumer { rb: self })
    }
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for Slot<T> {
    fn drop(&mut self) {
        self.dequeue();
    }
}

/// A slot "consumer"; it can dequeue an item from the slot
pub struct Consumer<'a, T> {
    rb: &'a Slot<T>,
}

unsafe impl<'a, T> Send for Consumer<'a, T> where T: Send {}

/// A queue "producer"; it can enqueue an item into the slot
pub struct Producer<'a, T> {
    rb: &'a Slot<T>,
}

unsafe impl<'a, T> Send for Producer<'a, T> where T: Send {}

impl<'a, T> Consumer<'a, T> {
    /// Returns the item in the slot, or `None` if the slot is empty
    #[inline]
    pub fn dequeue(&mut self) -> Option<T> {
        unsafe { self.rb.inner_dequeue() }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.rb.is_empty()
    }
}

impl<'a, T> Producer<'a, T> {
    /// Adds an `item` to the slot, returns back the `item` if the slot is full
    #[inline]
    pub fn enqueue(&mut self, val: T) -> Result<(), T> {
        unsafe { self.rb.inner_enqueue(val) }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.rb.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(loom))]
    #[test]
    fn test_slot() {
        let mut slot = Slot::new();
        let (mut producer, mut consumer) = slot.split();
        assert_eq!(consumer.dequeue(), None);

        assert_eq!(producer.enqueue(1), Ok(()));
        assert_eq!(producer.enqueue(0), Err(0));
        assert_eq!(consumer.dequeue(), Some(1));
        assert_eq!(consumer.dequeue(), None);

        assert_eq!(producer.enqueue(2), Ok(()));
        assert_eq!(producer.enqueue(0), Err(0));
        assert_eq!(consumer.dequeue(), Some(2));
        assert_eq!(consumer.dequeue(), None);

        assert_eq!(producer.enqueue(3), Ok(()));
        assert_eq!(producer.enqueue(0), Err(0));
        assert_eq!(consumer.dequeue(), Some(3));
        assert_eq!(consumer.dequeue(), None);
    }

    #[test]
    fn test_loom() {
        #[cfg(loom)]
        use loom::{
            model,
            thread::{spawn, yield_now},
        };
        #[cfg(not(loom))]
        use std::thread::{spawn, yield_now};
        #[cfg(not(loom))]
        fn model<F: Fn() + Sync + Send + 'static>(f: F) {
            f()
        }

        model(|| {
            let slot = Box::leak(Box::new(Slot::new()));
            let slot_raw = slot as *mut Slot<i32>;

            let (mut producer, mut consumer) = slot.split();

            let handle = spawn(move || {
                producer.enqueue(1).unwrap();
                yield_now();
                producer.enqueue(2).ok();
            });

            yield_now();

            if let Some(v) = consumer.dequeue() {
                assert_eq!(v, 1);
                yield_now();

                if let Some(v) = consumer.dequeue() {
                    assert_eq!(v, 2);
                }
            }
            handle.join().unwrap();
            unsafe { drop(Box::from_raw(slot_raw)) };
        })
    }
}
