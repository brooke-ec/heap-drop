use heap_drop::HeapDrop;

#[derive(HeapDrop, Default)]
struct Tree {
    left: Option<Box<Tree>>,
    right: Option<Box<Tree>>,
    value: i32,
}

#[test]
fn is_implementing() {
    assert!(::heap_drop::maybe_as_heap_drop!(Tree::default()).is_some());
}

#[test]
fn not_implementing() {
    Box::new(Option::<Tree>::None).into_children();
    assert!(::heap_drop::maybe_as_heap_drop!(42).is_none());
}

#[test]
fn non_implementing_fields() {
    let tree = Tree {
        left: None,
        right: None,
        value: 42,
    };

    assert_eq!(Box::new(tree).into_children().len(), 2);
}
