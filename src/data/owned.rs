// Copyright (c) 2023 Julia DeMille
// 
// Licensed under the EUPL, Version 1.2
// 
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

use std::{
    cmp,
    ffi::{CString, NulError},
    i32,
    marker::PhantomData,
    os::raw::{c_int, c_void},
    ptr,
};

use xplane_sys::{
    xplmType_Data, xplmType_Double, xplmType_Float, xplmType_FloatArray, xplmType_Int,
    xplmType_IntArray, XPLMDataRef, XPLMFindDataRef, XPLMGetDatab_f, XPLMGetDatad_f,
    XPLMGetDataf_f, XPLMGetDatai_f, XPLMGetDatavf_f, XPLMGetDatavi_f, XPLMRegisterDataAccessor,
    XPLMSetDatab_f, XPLMSetDatad_f, XPLMSetDataf_f, XPLMSetDatai_f, XPLMSetDatavf_f,
    XPLMSetDatavi_f, XPLMUnregisterDataAccessor,
};

use super::{Access, ArrayRead, ArrayReadWrite, DataRead, DataReadWrite, DataType, ReadOnly};

/// A dataref owned by this plugin
///
/// The access parameter of this type determines whether X-Plane and other plugins can write
/// this dataref. Owned datarefs can always be written by this plugin.
pub struct OwnedData<T: DataType + ?Sized, A = ReadOnly> {
    /// The dataref handle
    id: XPLMDataRef,
    /// The current value
    ///
    /// This is boxed so that it will have a constant memory location that is
    /// provided as a refcon to the callbacks.
    value: Box<T::Storage>,
    /// Data access phantom data
    access_phantom: PhantomData<A>,
}

impl<T: DataType + ?Sized, A: Access> OwnedData<T, A> {
    /// Creates a new dataref with the provided name containing the default value of T
    /// # Errors
    /// Errors if there is a NUL character in the dataref name, or if a dataref with that name already exists.
    pub fn new(name: &str) -> Result<Self, CreateError>
    where
        T: Default,
    {
        Self::new_with_value(name, &T::default())
    }

    /// Creates a new dataref with the provided name and value
    /// # Errors
    /// Errors if there is a NUL character in the dataref name, or if a dataref with that name already exists.
    /// # Panics
    /// Panics if the dataref ID returned from X-Plane is null. This should not occur.
    pub fn new_with_value(name: &str, value: &T) -> Result<Self, CreateError> {
        let name_c = CString::new(name)?;

        let existing = unsafe { XPLMFindDataRef(name_c.as_ptr()) };
        if !existing.is_null() {
            return Err(CreateError::Exists);
        }

        let value = value.to_storage();
        let mut value_box = Box::new(value);
        let value_ptr: *mut T::Storage = value_box.as_mut();

        let id = unsafe {
            XPLMRegisterDataAccessor(
                name_c.as_ptr(),
                T::sim_type(),
                Self::writeable(),
                Self::int_read(),
                Self::int_write(),
                Self::float_read(),
                Self::float_write(),
                Self::double_read(),
                Self::double_write(),
                Self::int_array_read(),
                Self::int_array_write(),
                Self::float_array_read(),
                Self::float_array_write(),
                Self::byte_array_read(),
                Self::byte_array_write(),
                value_ptr.cast::<std::ffi::c_void>(),
                value_ptr.cast::<std::ffi::c_void>(),
            )
        };

        assert!(!id.is_null(), "Dataref ID of created dataref is null!");
        Ok(OwnedData {
            id,
            value: value_box,
            access_phantom: PhantomData,
        })
    }

    /// Returns 1 if this dataref should be writeable by other plugins and X-Plane
    fn writeable() -> i32 {
        i32::from(A::writeable())
    }
    fn int_read() -> XPLMGetDatai_f {
        if T::sim_type() & xplmType_Int as i32 == 0 {
            None
        } else {
            Some(int_read)
        }
    }
    fn int_write() -> XPLMSetDatai_f {
        if T::sim_type() & xplmType_Int as i32 != 0 && A::writeable() {
            Some(int_write)
        } else {
            None
        }
    }
    fn float_read() -> XPLMGetDataf_f {
        if T::sim_type() & xplmType_Float as i32 == 0 {
            None
        } else {
            Some(float_read)
        }
    }
    fn float_write() -> XPLMSetDataf_f {
        if T::sim_type() & xplmType_Float as i32 != 0 && A::writeable() {
            Some(float_write)
        } else {
            None
        }
    }
    fn double_read() -> XPLMGetDatad_f {
        if T::sim_type() & xplmType_Double as i32 == 0 {
            None
        } else {
            Some(double_read)
        }
    }
    fn double_write() -> XPLMSetDatad_f {
        if T::sim_type() & xplmType_Double as i32 != 0 && A::writeable() {
            Some(double_write)
        } else {
            None
        }
    }
    fn int_array_read() -> XPLMGetDatavi_f {
        if T::sim_type() & xplmType_IntArray as i32 == 0 {
            None
        } else {
            Some(int_array_read)
        }
    }
    fn int_array_write() -> XPLMSetDatavi_f {
        if T::sim_type() & xplmType_IntArray as i32 != 0 && A::writeable() {
            Some(int_array_write)
        } else {
            None
        }
    }
    fn float_array_read() -> XPLMGetDatavf_f {
        if T::sim_type() & xplmType_FloatArray as i32 == 0 {
            None
        } else {
            Some(float_array_read)
        }
    }
    fn float_array_write() -> XPLMSetDatavf_f {
        if T::sim_type() & xplmType_FloatArray as i32 != 0 && A::writeable() {
            Some(float_array_write)
        } else {
            None
        }
    }
    fn byte_array_read() -> XPLMGetDatab_f {
        if T::sim_type() & xplmType_Data as i32 == 0 {
            None
        } else {
            Some(byte_array_read)
        }
    }
    fn byte_array_write() -> XPLMSetDatab_f {
        if T::sim_type() & xplmType_Data as i32 != 0 && A::writeable() {
            Some(byte_array_write)
        } else {
            None
        }
    }
}

impl<T: DataType + ?Sized, A> Drop for OwnedData<T, A> {
    fn drop(&mut self) {
        unsafe { XPLMUnregisterDataAccessor(self.id) }
    }
}

// DataRead and DataReadWrite
macro_rules! impl_read_write {
    (for $native_type:ty) => {
        impl<A> DataRead<$native_type> for OwnedData<$native_type, A> {
            fn get(&self) -> $native_type {
                *self.value
            }
        }
        impl<A> DataReadWrite<$native_type> for OwnedData<$native_type, A> {
            fn set(&mut self, value: $native_type) {
                *self.value = value;
            }
        }
    };
    (for array [$native_type:ty]) => {
        impl<A> ArrayRead<[$native_type]> for OwnedData<[$native_type], A> {
            fn get(&self, dest: &mut [$native_type]) -> usize {
                let copy_length = cmp::min(dest.len(), self.value.len());
                let dest_sub = &mut dest[..copy_length];
                let value_sub = &self.value[..copy_length];
                dest_sub.copy_from_slice(value_sub);
                copy_length
            }
            fn len(&self) -> usize {
                self.value.len()
            }
        }
        impl<A> ArrayReadWrite<[$native_type]> for OwnedData<[$native_type], A> {
            fn set(&mut self, values: &[$native_type]) {
                let copy_length = cmp::min(values.len(), self.value.len());
                let src_sub = &values[..copy_length];
                let values_sub = &mut self.value[..copy_length];
                values_sub.copy_from_slice(src_sub);
            }
        }
    };
}

impl_read_write!(for u8);
impl_read_write!(for i8);
impl_read_write!(for u16);
impl_read_write!(for i16);
impl_read_write!(for i32);
impl_read_write!(for u32);
impl_read_write!(for f32);
impl_read_write!(for f64);
impl_read_write!(for bool);
impl_read_write!(for array [i32]);
impl_read_write!(for array [u32]);
impl_read_write!(for array [f32]);
impl_read_write!(for array [u8]);
impl_read_write!(for array [i8]);

/// Errors that can occur when creating a `DataRef`
#[derive(thiserror::Error, Debug)]
pub enum CreateError {
    /// The provided DataRef name contained a null byte
    #[error("Null byte in dataref name")]
    Null(#[from] NulError),

    /// The DataRef exists already
    #[error("DataRef already exists")]
    Exists,
}

// Read/write callbacks
// The refcon is a pointer to the data

/// Integer read callback
unsafe extern "C" fn int_read(refcon: *mut c_void) -> c_int {
    let data_ptr = refcon.cast::<i32>();
    *data_ptr
}

/// Integer write callback
unsafe extern "C" fn int_write(refcon: *mut c_void, value: c_int) {
    let data_ptr = refcon.cast::<i32>();
    *data_ptr = value;
}

/// Float read callback
unsafe extern "C" fn float_read(refcon: *mut c_void) -> f32 {
    let data_ptr = refcon.cast::<f32>();
    *data_ptr
}

/// Float write callback
unsafe extern "C" fn float_write(refcon: *mut c_void, value: f32) {
    let data_ptr = refcon.cast::<f32>();
    *data_ptr = value;
}

/// Double read callback
unsafe extern "C" fn double_read(refcon: *mut c_void) -> f64 {
    let data_ptr = refcon.cast::<f64>();
    *data_ptr
}

/// Double write callback
unsafe extern "C" fn double_write(refcon: *mut c_void, value: f64) {
    let data_ptr = refcon.cast::<f64>();
    *data_ptr = value;
}

/// Integer array read callback
/// T is the actual data type
unsafe extern "C" fn int_array_read(
    refcon: *mut c_void,
    values: *mut c_int,
    offset: c_int,
    max: c_int,
) -> c_int {
    array_read::<i32>(refcon, values, offset, max)
}

/// Integer array write callback
unsafe extern "C" fn int_array_write(
    refcon: *mut c_void,
    values: *mut c_int,
    offset: c_int,
    max: c_int,
) {
    array_write::<i32>(refcon, values, offset, max);
}

/// Float array read callback
unsafe extern "C" fn float_array_read(
    refcon: *mut c_void,
    values: *mut f32,
    offset: c_int,
    max: c_int,
) -> c_int {
    array_read::<f32>(refcon, values, offset, max)
}

/// Float array write callback
unsafe extern "C" fn float_array_write(
    refcon: *mut c_void,
    values: *mut f32,
    offset: c_int,
    max: c_int,
) {
    array_write::<f32>(refcon, values, offset, max);
}

/// Byte array read callback
unsafe extern "C" fn byte_array_read(
    refcon: *mut c_void,
    values: *mut c_void,
    offset: c_int,
    max: c_int,
) -> c_int {
    array_read::<u8>(refcon, values.cast::<u8>(), offset, max)
}

/// Byte array write callback
unsafe extern "C" fn byte_array_write(
    refcon: *mut c_void,
    values: *mut c_void,
    offset: c_int,
    max: c_int,
) {
    array_write::<u8>(refcon, values as *const u8, offset, max);
}

/// If values is null, returns the length of this dataref.
/// Otherwise, reads up to max elements from this dataref starting at offset offset and copies them
/// into values.
#[inline]
#[allow(clippy::cast_sign_loss)]
unsafe fn array_read<T: Copy>(
    refcon: *mut c_void,
    values: *mut T,
    offset: c_int,
    max: c_int,
) -> c_int {
    let offset = offset as usize;
    let max = max as usize;
    let dataref_content = refcon as *const Vec<T>;
    let dataref_length = (*dataref_content).len();
    if values.is_null() {
        dataref_length as c_int
    } else {
        // Check that offset is within dataref content
        if offset >= dataref_length {
            return 0;
        }
        let dataref_offset = (*dataref_content).as_ptr().add(offset);
        let copy_length = cmp::min(max, dataref_length - offset);
        ptr::copy_nonoverlapping(dataref_offset, values, copy_length);
        copy_length as c_int
    }
}

/// Reads up to max items from values and writes them to this dataref, starting at offset offset
#[inline]
#[allow(clippy::cast_sign_loss)]
unsafe fn array_write<T: Copy>(refcon: *mut c_void, values: *const T, offset: c_int, max: c_int) {
    let offset = offset as usize;
    let max = max as usize;
    let dataref_content = refcon.cast::<Vec<T>>();
    let dataref_length = (*dataref_content).len();

    if offset >= dataref_length {
        return;
    }
    let dataref_offset = (*dataref_content).as_mut_ptr().add(offset);
    let copy_length = cmp::min(max, dataref_length - offset);
    ptr::copy_nonoverlapping(values, dataref_offset, copy_length);
}
