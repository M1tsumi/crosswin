use crosswin::windows::handles::Handle;

#[test]
fn construct_handle_from_raw() {
    unsafe {
        let handle = Handle::from_raw(0);
        assert_eq!(handle.raw().0, 0);
        assert!(!handle.is_valid());
    }
}

#[test]
fn test_invalid_handle_detection() {
    unsafe {
        let null_handle = Handle::from_raw(0);
        assert!(!null_handle.is_valid());
        
        let invalid_handle = Handle::from_raw(-1);
        assert!(!invalid_handle.is_valid());
        
        let valid_handle = Handle::from_raw(100);
        assert!(valid_handle.is_valid());
    }
}
