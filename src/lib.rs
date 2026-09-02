//! Provides a way of deallocating a value using a heap-allocated queue to traverse fields instead of stack recursion.
//!
//! This effectively does nothing for types which implement `Copy`, e.g.
//! integers. Such values are copied and _then_ moved into the function, so the
//! value persists after this function call.
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```
//! use heap_drop::{HeapDrop, heap_drop};
//!
//! #[derive(HeapDrop)]
//! struct Node(Option<Box<Node>>);
//!
//! let v = Node(Some(Box::new(Node(None /* could be a very deep structure */))));
//! heap_drop(v);
//! ```
//!
use std::collections::VecDeque;

pub use heap_drop_macros::HeapDrop;

/// A trait that allows a type to be disposed of using a heap-allocated queue instead of the stack.
///
/// See the [heap_drop] function.
pub trait HeapDrop {
    /// Returns a vector of children of the current value which implement `HeapDrop`.
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>>;
}

// standard library implementations

impl<T: HeapDrop + 'static> HeapDrop for Option<T> {
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>> {
        match *self {
            Some(value) => vec![Box::new(value) as Box<dyn HeapDrop>],
            None => vec![],
        }
    }
}

impl<T: HeapDrop + 'static> HeapDrop for Box<T> {
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>> {
        vec![*self as Box<dyn HeapDrop>]
    }
}

impl<T: HeapDrop + 'static> HeapDrop for Vec<T> {
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>> {
        self.into_iter().map(|item| Box::new(item) as Box<dyn HeapDrop>).collect()
    }
}

impl<T: HeapDrop + 'static> HeapDrop for VecDeque<T> {
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>> {
        self.into_iter().map(|item| Box::new(item) as Box<dyn HeapDrop>).collect()
    }
}

impl<T: HeapDrop + 'static> HeapDrop for std::collections::LinkedList<T> {
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>> {
        self.into_iter().map(|item| Box::new(item) as Box<dyn HeapDrop>).collect()
    }
}

impl<T: HeapDrop + 'static> HeapDrop for std::collections::HashSet<T> {
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>> {
        self.into_iter().map(|item| Box::new(item) as Box<dyn HeapDrop>).collect()
    }
}

impl<K, V: HeapDrop + 'static> HeapDrop for std::collections::HashMap<K, V> {
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>> {
        self.into_values().map(|value| Box::new(value) as Box<dyn HeapDrop>).collect()
    }
}

impl<T: HeapDrop + 'static> HeapDrop for std::cell::Cell<T> {
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>> {
        vec![Box::new(self.into_inner()) as Box<dyn HeapDrop>]
    }
}

impl<T: HeapDrop + 'static> HeapDrop for core::cell::RefCell<T> {
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>> {
        vec![Box::new(self.into_inner()) as Box<dyn HeapDrop>]
    }
}

impl<T: HeapDrop + 'static> HeapDrop for std::rc::Rc<T> {
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>> {
        vec![Box::new((*self).clone()) as Box<dyn HeapDrop>]
    }
}

impl<T: HeapDrop + 'static> HeapDrop for std::sync::Arc<T> {
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>> {
        vec![Box::new((*self).clone()) as Box<dyn HeapDrop>]
    }
}

impl<T: HeapDrop + 'static> HeapDrop for std::sync::Mutex<T> {
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>> {
        vec![Box::new(self.into_inner().unwrap()) as Box<dyn HeapDrop>]
    }
}

impl<T: HeapDrop + 'static> HeapDrop for std::sync::RwLock<T> {
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>> {
        vec![Box::new(self.into_inner().unwrap()) as Box<dyn HeapDrop>]
    }
}

// implementation detection logic

#[doc(hidden)]
pub struct HeapDropDetect<T>(pub Box<T>);

#[doc(hidden)]
pub trait HeapDropDetectFallback {
    fn detect(self) -> Option<Box<dyn HeapDrop>>;
}

impl<T> HeapDropDetectFallback for HeapDropDetect<T> {
    fn detect(self) -> Option<Box<dyn HeapDrop>> {
        None
    }
}

impl<T: HeapDrop + 'static> HeapDropDetect<T> {
    pub fn detect(self) -> Option<Box<dyn HeapDrop>> {
        Some(self.0 as Box<dyn HeapDrop>)
    }
}

/// A macro that attempts to cast a value to a `Box<dyn HeapDrop>`, returning `None` if the value does not implement `HeapDrop`.
///
/// This macro requires that the value has a concrete type in the caller scope, and will not work with generic type parameters.
/// It is not part of the public API and should not be used outside of the crate.
#[doc(hidden)]
#[macro_export]
macro_rules! maybe_as_heap_drop {
    ($val:expr) => {{
        #[allow(unused_imports)]
        use $crate::HeapDropDetectFallback;
        $crate::HeapDropDetect(Box::new($val)).detect()
    }};
}

// public api functions

/// Disposes of a value using a heap-allocated queue to traverse fields instead of the stack.
///
/// This effectively does nothing for types which implement `Copy`, e.g.
/// integers. Such values are copied and _then_ moved into the function, so the
/// value persists after this function call.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use heap_drop::{HeapDrop, heap_drop};
///
/// #[derive(HeapDrop)]
/// struct Node(Option<Box<Node>>);
///
/// let v = Node(Some(Box::new(Node(None /* could be a very deep structure */))));
/// heap_drop(v);
/// ```
pub fn heap_drop<T: HeapDrop>(value: T) {
    let mut queue = VecDeque::new();

    // add all anscestors to the queue and drop them one by one
    queue.push_back(Box::new(value) as Box<dyn HeapDrop>);
    while let Some(item) = queue.pop_front() {
        let children = item.into_children();
        queue.extend(children);
    }
}
