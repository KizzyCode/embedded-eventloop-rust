//! A stack-allocated type-opaque box

use core::{
    any::TypeId,
    mem::{self, MaybeUninit},
    ptr,
};

/// A stack-allocated type-opaque box
#[derive(Debug)]
pub struct Box<const SIZE: usize> {
    /// The type info
    type_id: TypeId,
    /// The opaque bytes of the value
    bytes: [u8; SIZE],
    /// A destructor to drop the value
    drop: Option<fn(TypeId, [u8; SIZE])>,
}
impl<const SIZE: usize> Box<SIZE> {
    /// Creates a new stackbox with the given `value`, returns `Err(value)` if the value is larger than `SIZE`
    pub fn new<T>(value: T) -> Result<Self, T>
    where
        T: 'static,
    {
        // Validate that `T` fits into the box
        if mem::size_of::<T>() > SIZE {
            return Err(value);
        };

        // Wrap the value
        let (type_id, bytes) = value_into_bytes(value);
        Ok(Self { type_id, bytes, drop: Some(Self::drop_impl::<T>) })
    }

    /// The type ID of the inner value
    pub fn inner_type_id(&self) -> TypeId {
        self.type_id
    }

    /// Unwraps the underlying wrapped value, return `Err(self)` if the value is not of type `T`
    pub fn into_inner<T>(mut self) -> Result<T, Self>
    where
        T: 'static,
    {
        // Validate that we have boxed a type `T`
        if TypeId::of::<T>() != self.type_id {
            return Err(self);
        }

        // Remove the destructor and get the value
        self.drop = None;
        let value = bytes_into_value(self.type_id, self.bytes);
        Ok(value)
    }

    /// Safely unwraps a value of type `T` and drops it
    fn drop_impl<T>(type_id: TypeId, bytes: [u8; SIZE])
    where
        T: 'static,
    {
        let value: T = bytes_into_value(type_id, bytes);
        drop(value);
    }
}
impl<const SIZE: usize> Drop for Box<SIZE> {
    fn drop(&mut self) {
        // Call the destructor if any
        if let Some(drop) = self.drop.take() {
            drop(self.type_id, self.bytes);
        }
    }
}

/// A stack-allocated type-opaque box for copyable values
#[derive(Debug, Clone, Copy)]
pub struct CopyBox<const SIZE: usize> {
    /// The type info
    type_id: TypeId,
    /// The opaque bytes of the value
    bytes: [u8; SIZE],
}
impl<const SIZE: usize> CopyBox<SIZE> {
    /// Creates a new stackbox with the given `value`, returns `None` if the value is larger than `SIZE`
    pub fn new<T>(value: T) -> Option<Self>
    where
        T: 'static,
    {
        // Validate that `T` fits into the box
        if mem::size_of::<T>() > SIZE {
            return None;
        };

        // Wrap the value
        let (type_id, bytes) = value_into_bytes(value);
        Some(Self { type_id, bytes })
    }

    /// The type ID of the inner value
    pub fn inner_type_id(&self) -> TypeId {
        self.type_id
    }

    /// Unwraps the underlying wrapped value, return `Err(self)` if the value is not of type `T`
    pub fn inner<T>(&self) -> Option<T>
    where
        T: 'static,
    {
        // Validate that we have boxed a type `T`
        if TypeId::of::<T>() != self.type_id {
            return None;
        }

        // Copy the value
        let value = bytes_into_value(self.type_id, self.bytes);
        Some(value)
    }
}

/// Safely transforms a value into a byte array
fn value_into_bytes<T, const SIZE: usize>(value: T) -> (TypeId, [u8; SIZE])
where
    T: 'static,
{
    // Validate constraints
    assert!(mem::size_of::<T>() <= SIZE, "type is too large for stackbox");

    // Copy the value
    let mut bytes = [0; SIZE];
    let value_ptr = ptr::addr_of!(value) as *const u8;
    unsafe { bytes.as_mut_ptr().copy_from_nonoverlapping(value_ptr, mem::size_of::<T>()) };

    // Forget the value and return the data
    mem::forget(value);
    (TypeId::of::<T>(), bytes)
}

/// Safely recovers a value from a byte array
fn bytes_into_value<T, const SIZE: usize>(type_id: TypeId, bytes: [u8; SIZE]) -> T
where
    T: 'static,
{
    // Validate constraints
    assert_eq!(type_id, TypeId::of::<T>(), "type mismatch");

    // Recover the value
    let mut value = MaybeUninit::uninit();
    let value_ptr = value.as_mut_ptr() as *mut u8;
    unsafe { bytes.as_ptr().copy_to_nonoverlapping(value_ptr, mem::size_of::<T>()) };

    // Unwrap the value
    unsafe { value.assume_init() }
}
