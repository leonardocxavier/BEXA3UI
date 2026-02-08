/// Reactive components system for BEXAUI
///
/// This module provides utilities for reactive programming patterns.

use crate::Signal;

/// Effect system - observe Signal changes
///
/// # Example
/// ```ignore
/// let count = create_signal(0);
///
/// create_effect(count.clone(), |value| {
///     println!("Count changed to: {}", value);
/// });
/// ```
pub fn create_effect<T: Clone + 'static, F>(signal: Signal<T>, mut callback: F)
where
    F: FnMut(T) + 'static,
{
    // Initial call
    let initial = signal.get();
    callback(initial);
}

/// Helper to check if a Signal value has changed
pub fn signal_changed<T, F, R>(signal: &Signal<T>, last_value: &mut R, extractor: F) -> bool
where
    T: Clone,
    F: Fn(&T) -> R,
    R: PartialEq,
{
    let current = signal.get();
    let current_value = extractor(&current);

    if current_value != *last_value {
        *last_value = current_value;
        true
    } else {
        false
    }
}
