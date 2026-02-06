use std::cell::RefCell;
use std::rc::Rc;

/// Read-only handle to a reactive value.
#[derive(Debug)]
pub struct Signal<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal { inner: self.inner.clone() }
    }
}

impl<T: Clone> Signal<T> {
    /// Returns a clone of the current value.
    pub fn get(&self) -> T {
        self.inner.borrow().clone()
    }
}

impl<T> Signal<T> {
    /// Borrows the current value and passes it to the closure.
    /// Avoids cloning — preferred in hot paths like draw().
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&*self.inner.borrow())
    }
}

/// Write handle to a reactive value.
#[derive(Debug)]
pub struct SetSignal<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> Clone for SetSignal<T> {
    fn clone(&self) -> Self {
        SetSignal { inner: self.inner.clone() }
    }
}

impl<T> SetSignal<T> {
    /// Replaces the current value.
    pub fn set(&self, value: T) {
        *self.inner.borrow_mut() = value;
    }

    /// Mutates the current value via a closure.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut *self.inner.borrow_mut());
    }
}

/// Creates a signal pair: `(reader, writer)`.
///
/// ```
/// let (count, set_count) = create_signal(0);
/// set_count.set(42);
/// assert_eq!(count.get(), 42);
/// ```
pub fn create_signal<T>(initial: T) -> (Signal<T>, SetSignal<T>) {
    let inner = Rc::new(RefCell::new(initial));
    (
        Signal { inner: inner.clone() },
        SetSignal { inner },
    )
}

/// Convert various types into a `Signal<T>`.
pub trait IntoSignal<T> {
    fn into_signal(self) -> Signal<T>;
}

impl<T> IntoSignal<T> for Signal<T> {
    fn into_signal(self) -> Signal<T> {
        self
    }
}

impl<T> IntoSignal<T> for Rc<RefCell<T>> {
    fn into_signal(self) -> Signal<T> {
        Signal { inner: self }
    }
}
