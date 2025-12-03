use crosswin::windows::handles::Handle;

#[test]
fn construct_placeholder_handle() {
    let handle = Handle::from_raw(0);
    assert_eq!(handle.raw().0, 0);
}
