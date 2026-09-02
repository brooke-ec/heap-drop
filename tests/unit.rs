use heap_drop::HeapDrop;

#[derive(HeapDrop, Default, Hash, PartialEq, Eq)]
struct Tree {
    left: Option<Box<Tree>>,
    right: Option<Box<Tree>>,
    value: i32,
}

// check derive functionality

#[test]
fn derive_test() {
    let tree = Tree {
        right: None,
        left: None,
        value: 42,
    };

    assert_eq!(Box::new(tree).into_children().len(), 2);
}

// check macro functionality

#[test]
fn is_implementing() {
    assert!(::heap_drop::maybe_as_heap_drop!(Tree::default()).is_some());
}

#[test]
fn not_implementing() {
    assert!(::heap_drop::maybe_as_heap_drop!(42).is_none());
}

// check option implementation

#[test]
fn option_none() {
    assert!(::heap_drop::maybe_as_heap_drop!(Option::<Tree>::None).is_some());
}

#[test]
fn option_some() {
    assert!(::heap_drop::maybe_as_heap_drop!(Some(Tree::default())).is_some());
}

#[test]
fn option_some_non_implementing() {
    assert!(::heap_drop::maybe_as_heap_drop!(Some(42)).is_none());
}

// check box implementation

#[test]
fn box_implementing() {
    assert!(::heap_drop::maybe_as_heap_drop!(Box::new(Tree::default())).is_some());
}

#[test]
fn box_non_implementing() {
    assert!(::heap_drop::maybe_as_heap_drop!(Box::new(42)).is_none());
}

#[test]
fn box_children() {
    assert_eq!(Box::new(Box::new(Tree::default())).into_children().len(), 1);
}

// check vec implementation

#[test]
fn vec_implementing() {
    assert!(::heap_drop::maybe_as_heap_drop!(vec![Tree::default()]).is_some());
}

#[test]
fn vec_non_implementing() {
    assert!(::heap_drop::maybe_as_heap_drop!(vec![42]).is_none());
}

#[test]
fn vec_children() {
    assert_eq!(Box::new(vec![Tree::default()]).into_children().len(), 1);
}

// check hashmap implementation

#[test]
fn hashmap_implementing() {
    assert!(::heap_drop::maybe_as_heap_drop!(std::collections::HashMap::<String, Tree>::new()).is_some());
}

#[test]
fn hashmap_non_implementing() {
    assert!(::heap_drop::maybe_as_heap_drop!(std::collections::HashMap::<String, i32>::new()).is_none());
}

#[test]
fn hashmap_children() {
    let mut map = std::collections::HashMap::<Tree, Tree>::new();
    map.insert(Tree::default(), Tree::default());
    assert_eq!(Box::new(map).into_children().len(), 1);
}
