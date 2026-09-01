use std::collections::VecDeque;

pub use heap_drop_macros::HeapDrop;

pub trait HeapDrop {
    fn into_children(self: Box<Self>) -> Vec<Box<dyn HeapDrop>>;
}

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

#[doc(hidden)]
#[macro_export]
macro_rules! maybe_as_heap_drop {
    ($val:expr) => {{
        #[allow(unused_imports)]
        use $crate::HeapDropDetectFallback;
        $crate::HeapDropDetect(Box::new($val)).detect()
    }};
}

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
/// let v = Node(Some(Box::new(Node(None))));
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
