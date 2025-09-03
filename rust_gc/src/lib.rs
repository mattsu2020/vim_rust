use std::cell::RefCell;
use std::rc::Rc;

/// A very small garbage-collected heap using Rust ownership.
///
/// Values allocated through [`GcHeap::alloc`] are reference counted and
/// automatically cleaned up when the heap is dropped.
pub struct GcHeap<T> {
    objects: RefCell<Vec<Rc<T>>>,
}

impl<T> GcHeap<T> {
    /// Create a new empty heap.
    pub fn new() -> Self {
        Self { objects: RefCell::new(Vec::new()) }
    }

    /// Allocate a value on the heap and return a [`Gc`] handle.
    pub fn alloc(&self, value: T) -> Gc<T> {
        let rc = Rc::new(value);
        self.objects.borrow_mut().push(rc.clone());
        Gc { inner: rc }
    }
}

/// Handle to a value allocated on a [`GcHeap`].
#[derive(Clone)]
pub struct Gc<T> {
    inner: Rc<T>,
}

impl<T> std::ops::Deref for Gc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_drop() {
        struct DropTracker<'a>(&'a RefCell<bool>);
        impl<'a> Drop for DropTracker<'a> {
            fn drop(&mut self) {
                *self.0.borrow_mut() = true;
            }
        }

        let dropped = RefCell::new(false);
        {
            let heap = GcHeap::new();
            let _v = heap.alloc(DropTracker(&dropped));
            assert_eq!(*dropped.borrow(), false);
        }
        // Heap dropped, value should be dropped as well.
        assert_eq!(*dropped.borrow(), true);
    }
}
