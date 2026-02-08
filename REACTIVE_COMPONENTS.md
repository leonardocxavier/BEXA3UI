# Reactive Components System - BEXAUI

## Overview

This document describes the reactive programming utilities added to BEXAUI to enable better Signal-driven UI updates.

## Current Features

### 1. `for` Loop Support in `ui!` Macro

**Syntax:**
```rust
ui! {
    Container => {
        Flex::column(8.0, 0.0) => {
            for item in (items) {
                Label::new(item.name)
            }
        }
    }
}
```

**Example:**
See `examples/for_loop_rendering.rs` for a complete working example.

### 2. Reactive Utilities

**Module:** `bexa_ui::reactive`

#### `signal_changed<T, F, R>`

Helper to detect if a Signal's value has changed.

```rust
use bexa_ui::prelude::*;

let (items, set_items) = create_signal(vec![1, 2, 3]);
let mut last_len = items.get().len();

// Later, check if changed
if signal_changed(&items, &mut last_len, |v| v.len()) {
    println!("Items list changed!");
    // Rebuild UI or update state
}
```

#### `create_effect<T, F>`

Observe Signal changes (foundation for future reactive system).

```rust
let (count, set_count) = create_signal(0);

create_effect(count.clone(), |value| {
    println!("Count is now: {}", value);
});
```

**Note:** Current implementation calls callback once on creation. Full reactivity (automatic re-execution when Signal changes) requires deeper framework integration.

## Limitations & Future Work

### Current Limitations

1. **No Automatic Re-render**: Changing a Signal does NOT automatically rebuild the widget tree.
   - Widgets using `Rc<RefCell<T>>` update their content ✅
   - Widget tree structure does NOT auto-update ❌

2. **WidgetNode is not Clone**: Cannot store and reuse WidgetNodes, limiting reactive patterns.

3. **No Virtual DOM**: No diffing or efficient partial updates.

### Future Roadmap

#### Phase 1: Foundation (✅ **COMPLETED**)
- [x] Add `for` loop support to `ui!` macro
- [x] Add `reactive` module with helper utilities
- [x] Document reactive patterns

#### Phase 2: Reactive Components (🚧 Planned)
- [ ] Implement `Clone` for `WidgetNode`
- [ ] Add `ReactiveList<T>` component
- [ ] Global reactive context for automatic updates
- [ ] `create_effect` with auto-subscription

#### Phase 3: Advanced Features (📋 Future)
- [ ] Virtual DOM with diffing
- [ ] Batch updates
- [ ] Memoization (`create_memo`)
- [ ] Derived signals (`create_derived`)
- [ ] Async effects

## Design Philosophy

BEXAUI's reactive system is designed to be:

1. **Opt-in**: Apps don't need reactivity if they don't want it
2. **Lightweight**: Minimal overhead when not used
3. **Explicit**: Clear when reactivity happens
4. **Rust-idiomatic**: Leverages Rust's type system

## Contributing

To contribute to the reactive system:

1. **Understand the constraints**: `WidgetNode` ownership model is complex
2. **Start small**: Add utilities before changing core framework
3. **Test thoroughly**: Reactivity bugs are hard to debug
4. **Document patterns**: Show users how to use reactive features

## Examples

### Pattern 1: Manual Rebuild on Signal Change

```rust
// Current best practice for dynamic lists
let (items, set_items) = create_signal(vec![]);
let mut last_count = 0;

// In render loop or update callback
if signal_changed(&items, &mut last_count, |v| v.len()) {
    // Rebuild only the affected panel
    rebuild_server_list();
}
```

### Pattern 2: Reactive Labels

```rust
// Text content updates automatically
let status = Rc::new(RefCell::new("Idle".to_string()));

let label = Label::new(
    status.clone(),
    metrics,
    [255, 255, 255],
);

// Later: update status
*status.borrow_mut() = "Connecting...".to_string();
// Label text updates on next frame ✅
```

### Pattern 3: Conditional Rendering

```rust
let show_panel = true; // or from Signal

ui! {
    Container => {
        if (show_panel) {
            Panel::new() => {
                Label::new("Panel content")
            }
        } else {
            Label::new("Panel hidden")
        }
    }
}
```

## See Also

- `examples/conditional_rendering.rs` - if/else in ui! macro
- `examples/for_loop_rendering.rs` - for loops in ui! macro
- `crates/bexa-ui-core/src/reactive.rs` - Reactive utilities source

---

**Status**: ✅ Foundation complete, 🚧 Full reactivity in progress

**Last Updated**: 2026-02-07
