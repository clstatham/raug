use std::sync::{Arc, OnceLock};

/// A sender for the [`Eventual`] synchronization primitive.
#[must_use = "Promise must be used to send a value to the corresponding Eventual"]
pub struct Promise<T> {
    value: Arc<OnceLock<T>>,
}

impl<T> Promise<T> {
    /// Sends a value to the corresponding [`Eventual`].
    ///
    /// This function consumes the [`Promise`], so it can only be used once,
    /// which ensures that the value can be moved out of the receiving [`Eventual`].
    pub fn fulfill(self, value: T) -> Result<(), T> {
        self.value.set(value)
    }
}

/// A synchronization primitive that can be used to eventually get a value from another thread.
/// The value can be set exactly once, and can be retrieved multiple times, or moved out of the [`Eventual`] exactly once.
///
/// If the value is not yet set, the `get` method will return `None`.
///
/// The `is_set` method can be used to check if the value is set.
///
/// None of the operations, besides [`new()`](Eventual::new), will allocate memory.
///
/// In addition, this type is `!Clone`, to ensure the value can be moved out of the [`Eventual`] once set.
#[derive(Debug, PartialEq)]
#[must_use = "Eventual must be used to receive a value from the corresponding Promise"]
pub struct Eventual<T> {
    value: Arc<OnceLock<T>>,
}

impl<T> Eventual<T> {
    /// Creates a new [`Eventual`] and its corresponding [`Promise`].
    ///
    /// The promise can be used to set the value exactly once.
    #[inline]
    pub fn new() -> (Self, Promise<T>) {
        let inner = Arc::new(OnceLock::new());
        (
            Self {
                value: inner.clone(),
            },
            Promise { value: inner },
        )
    }

    /// Checks if the value is set.
    #[inline]
    pub fn is_set(&self) -> bool {
        self.get().is_some()
    }

    /// Tries to get the value.
    ///
    /// If the value is not yet sent, returns `None`.
    ///
    /// This function will not block the current thread.
    #[inline]
    pub fn get(&self) -> Option<&T> {
        self.value.get()
    }

    /// Waits until the value is set and returns a reference to it.
    ///
    /// This function WILL block the current thread until the value is available.
    #[inline]
    pub fn wait(&self) -> &T {
        self.value.wait()
    }

    /// Consumes the [`Eventual`] and returns the value if it is set, or returns the [`Eventual`] back if it is not set.
    #[inline]
    #[cfg_attr(debug_assertions, allow(unused))]
    pub fn consume(self) -> Result<T, Self> {
        // pre-check if value is set
        if self.value.get().is_none() {
            return Err(self);
        }

        match Arc::try_unwrap(self.value) {
            Ok(value) => match value.into_inner() {
                Some(value) => Ok(value),
                None => {
                    unreachable!("OnceLock was checked to be set, but is not set now");
                }
            },
            Err(value) => {
                #[cfg(debug_assertions)]
                unreachable!(
                    "Arc has multiple strong references, despite sender being consumed and receiver not being cloned"
                );
                #[cfg(not(debug_assertions))]
                Err(Self { value })
            }
        }
    }
}

impl<T> std::ops::Deref for Eventual<T> {
    type Target = OnceLock<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use super::Eventual;

    #[test]
    fn test_eventual() {
        let (eventual, sender) = Eventual::new();
        assert!(!eventual.is_set());
        assert!(eventual.get().is_none());

        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            sender.fulfill(42).unwrap();
        });
        thread::sleep(Duration::from_millis(200));
        assert!(eventual.is_set());
        assert_eq!(eventual.get(), Some(&42));
        assert_eq!(eventual.get(), Some(&42));
        assert_eq!(eventual.consume(), Ok(42));
        handle.join().unwrap();
    }
}
