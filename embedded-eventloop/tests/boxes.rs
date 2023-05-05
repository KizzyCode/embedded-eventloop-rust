//! A box

use eventloop::boxes::{Box, CopyBox};
use std::rc::Rc;

#[test]
fn box_simple() {
    // Box and unbox value
    let value = (17u8, 4u64);
    let boxed = Box::<128>::new(value).expect("failed to box simple value");
    let unboxed = boxed.into_inner().expect("failed to unbox simple value");

    // Compare values
    assert_eq!(value, unboxed, "invalid unboxed value");
}

#[test]
fn copybox_simple() {
    /// Squares a number
    fn square(num: usize) -> usize {
        num * num
    }

    // Box and unbox value
    let value: fn(usize) -> usize = square;
    let boxed = CopyBox::<128>::new(value).expect("failed to box simple value");
    let unboxed = boxed.inner().expect("failed to unbox simple value");

    // Compare values
    assert_eq!(value, unboxed, "invalid unboxed value");
    assert_eq!(unboxed(7), 49, "invalid function result");
}

#[test]
#[cfg(target_family = "unix")]
fn box_complex() {
    use std::os::unix::net::UnixDatagram;

    // Box and unbox value
    let (socket, value) = UnixDatagram::pair().expect("failed to create coupled datagram socket");
    let boxed = Box::<128>::new(value).expect("failed to box complex value");
    let unboxed: UnixDatagram = boxed.into_inner().expect("failed to unbox complex value");

    // Send some data and receive them again
    let mut contents = [0; 9];
    socket.send(b"Testolope").expect("failed to send datagram");
    unboxed.recv(&mut contents).expect("failed to receive data");

    // Test the data
    assert_eq!(contents, *b"Testolope", "empty read from boxed file");
}

#[test]
fn box_drop() {
    // Box the value and validate the reference count
    let rc = Rc::new(7);
    let boxed = Box::<128>::new(Rc::clone(&rc)).expect("failed to box reference counted value");
    assert_eq!(Rc::strong_count(&rc), 2, "invalid reference count");

    // Drop the box and validate the reference count
    drop(boxed);
    assert_eq!(Rc::strong_count(&rc), 1, "invalid reference count");
}

#[test]
fn box_constraints_size() {
    // Create a value that is too large
    let value = 7u64;
    assert!(Box::<7>::new(value).is_err(), "unexpected success when boxing too large value");
}

#[test]
fn copybox_constraints_size() {
    // Create a value that is too large
    let value = 7u64;
    assert!(CopyBox::<7>::new(value).is_none(), "unexpected success when boxing too large value");
}

#[test]
fn box_constraints_type() {
    // Box an u64
    let value = 7u64;
    let boxed = Box::<128>::new(value).expect("failed to box u64-typed value");

    // Try to unbox it as i64
    assert!(boxed.into_inner::<i64>().is_err(), "unexpected success when unboxing u64-typed value as i64");
}

#[test]
fn copybox_constraints_type() {
    // Box an u64
    let value = 7u64;
    let boxed = CopyBox::<128>::new(value).expect("failed to box u64-typed value");

    // Try to unbox it as i64
    assert!(boxed.inner::<i64>().is_none(), "unexpected success when unboxing u64-typed value as i64");
}
