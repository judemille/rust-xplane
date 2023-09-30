// Copyright (c) 2023 Julia DeMille
// 
// Licensed under the EUPL, Version 1.2
// 
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

use std::{
    ffi::{CString, NulError},
    marker::PhantomData,
    os::raw::c_void,
    ptr,
};

use xplane_sys::{
    XPLMCanWriteDataRef, XPLMDataRef, XPLMFindDataRef, XPLMGetDataRefTypes, XPLMGetDatab,
    XPLMGetDatad, XPLMGetDataf, XPLMGetDatai, XPLMGetDatavf, XPLMGetDatavi, XPLMSetDatab,
    XPLMSetDatad, XPLMSetDataf, XPLMSetDatai, XPLMSetDatavf, XPLMSetDatavi,
};

use super::{ArrayRead, ArrayReadWrite, DataRead, DataReadWrite, DataType, ReadOnly, ReadWrite};

/// A dataref created by X-Plane or another plugin
///
/// T is the data type stored in the dataref.
///
/// A is the access level (`ReadOnly` or `ReadWrite`)
pub struct DataRef<T: ?Sized, A = ReadOnly> {
    /// The dataref handle
    id: XPLMDataRef,
    /// Type phantom data
    _type_phantom: PhantomData<T>,
    /// Data access phantom data
    _access_phantom: PhantomData<A>,
}

impl<T: DataType + ?Sized> DataRef<T, ReadOnly> {
    /// Finds a readable dataref by its name
    /// # Errors
    /// Returns an error if the dataref does not exist or has the wrong type
    pub fn find(name: &str) -> Result<Self, FindError> {
        let name_c = CString::new(name)?;
        let expected_type = T::sim_type();

        let dataref = unsafe { XPLMFindDataRef(name_c.as_ptr()) };
        if dataref.is_null() {
            return Err(FindError::NotFound);
        }

        let actual_type = unsafe { XPLMGetDataRefTypes(dataref) };
        if actual_type & expected_type == 0 {
            Err(FindError::WrongType)
        } else {
            Ok(DataRef {
                id: dataref,
                _type_phantom: PhantomData,
                _access_phantom: PhantomData,
            })
        }
    }

    /// Makes this dataref writable
    /// # Errors
    /// Returns an error if the dataref cannot be written.
    pub fn writeable(self) -> Result<DataRef<T, ReadWrite>, Self> {
        let writable = unsafe { XPLMCanWriteDataRef(self.id) == 1 };
        if writable {
            Ok(DataRef {
                id: self.id,
                _type_phantom: PhantomData,
                _access_phantom: PhantomData,
            })
        } else {
            Err(self)
        }
    }
}

/// Creates a `DataType` implementation, `DataRef::get` and `DataRef::set` for a type
macro_rules! dataref_type {
    // Basic case
    (
        $(#[$meta:meta])*
        dataref type {
            native $native_type:ty;
            sim $sim_type:ident as $sim_native_type:ty;
            read $read_fn:ident;
            write $write_fn:ident;
        }
    ) => {
        impl<A> DataRead<$native_type> for DataRef<$native_type, A> {
            fn get(&self) -> $native_type {
                unsafe { $read_fn(self.id) as $native_type }
            }
        }
        impl DataReadWrite<$native_type> for DataRef<$native_type, ReadWrite> {
            fn set(&mut self, value: $native_type) {
                unsafe { $write_fn(self.id, value as $sim_native_type) }
            }
        }
    };
    // Array case
    (
        $(#[$meta:meta])*
        dataref array type {
            native [$native_type:ty];
            sim $sim_type:ident as [$sim_native_type:ty];
            $(#[$read_meta:meta])*
            read $read_fn:ident;
            $(#[$write_meta:meta])*
            write $write_fn:ident;
        }
    ) => {
        impl<A> ArrayRead<[$native_type]> for DataRef<[$native_type], A> {
            #[allow(trivial_casts)]
            fn get(&self, dest: &mut [$native_type]) -> usize {
                let size = array_size(dest.len());
                let copy_count = unsafe {
                    $read_fn(
                        self.id,
                        dest.as_mut_ptr().cast::<$sim_native_type>(),
                        0,
                        size,
                    )
                };
                copy_count as usize
            }
            fn len(&self) -> usize {
                let size = unsafe { $read_fn(self.id, ptr::null_mut(), 0, 0) };
                size as usize
            }
        }

        impl ArrayReadWrite<[$native_type]> for DataRef<[$native_type], ReadWrite> {
            fn set(&mut self, values: &[$native_type]) {
                let size = array_size(values.len());
                unsafe {
                    // Cast to *mut because the API requires it
                    $write_fn(self.id, values.as_ptr() as *mut $sim_native_type, 0, size);
                }
            }
        }
    };
}

dataref_type! {
    dataref type {
        native u8;
        sim xplmType_Int as i32;
        read XPLMGetDatai;
        write XPLMSetDatai;
    }
}
dataref_type! {
    dataref type {
        native i8;
        sim xplmType_Int as i32;
        read XPLMGetDatai;
        write XPLMSetDatai;
    }
}
dataref_type! {
    dataref type {
        native u16;
        sim xplmType_Int as i32;
        read XPLMGetDatai;
        write XPLMSetDatai;
    }
}
dataref_type! {
    dataref type {
        native i16;
        sim xplmType_Int as i32;
        read XPLMGetDatai;
        write XPLMSetDatai;
    }
}
dataref_type! {
    dataref type {
        native u32;
        sim xplmType_Int as i32;
        read XPLMGetDatai;
        write XPLMSetDatai;
    }
}
dataref_type! {
    dataref type {
        native i32;
        sim xplmType_Int as i32;
        read XPLMGetDatai;
        write XPLMSetDatai;
    }
}
dataref_type! {
    dataref type {
        native f32;
        sim xplmType_Float as f32;
        read XPLMGetDataf;
        write XPLMSetDataf;
    }
}
dataref_type! {
    dataref type {
        native f64;
        sim xplmType_Double as f64;
        read XPLMGetDatad;
        write XPLMSetDatad;
    }
}
dataref_type! {
    dataref array type {
        native [i32];
        sim xplmType_IntArray as [i32];
        read XPLMGetDatavi;
        write XPLMSetDatavi;
    }
}
dataref_type! {
    dataref array type {
        native [u32];
        sim xplmType_IntArray as [i32];
        read XPLMGetDatavi;
        write XPLMSetDatavi;
    }
}
dataref_type! {
    dataref array type {
        native [f32];
        sim xplmType_FloatArray as [f32];
        read XPLMGetDatavf;
        write XPLMSetDatavf;
    }
}
dataref_type! {
    dataref array type {
        native [u8];
        sim xplmType_Data as [c_void];
        read XPLMGetDatab;
        write XPLMSetDatab;
    }
}
dataref_type! {
    dataref array type {
        native [i8];
        sim xplmType_Data as [c_void];
        read XPLMGetDatab;
        write XPLMSetDatab;
    }
}
impl<A> DataRead<bool> for DataRef<bool, A> {
    fn get(&self) -> bool {
        let int_value = unsafe { XPLMGetDatai(self.id) };
        int_value != 0
    }
}

impl DataReadWrite<bool> for DataRef<bool, ReadWrite> {
    fn set(&mut self, value: bool) {
        let int_value = i32::from(value);
        unsafe { XPLMSetDatai(self.id, int_value) };
    }
}

/// Converts a usize into an i32. Returns `i32::MAX` if the provided size is too large for an i32
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
fn array_size(size: usize) -> i32 {
    if size > (i32::MAX as usize) {
        i32::MAX
    } else {
        size as i32
    }
}

/// Errors that can occur when finding `DataRef`s
#[derive(thiserror::Error, Debug)]
pub enum FindError {
    /// The provided DataRef name contained a null byte
    #[error("Null byte in DataRef name")]
    Null(#[from] NulError),

    /// The DataRef could not be found
    #[error("DataRef not found")]
    NotFound,

    /// The DataRef is not writable
    #[error("DataRef not writable")]
    NotWritable,

    /// The DataRef does not have the correct type
    #[error("Incorrect DataRef type")]
    WrongType,
}

#[cfg(test)]
mod tests {
    /// Checks that the as operator truncates values
    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn test_as_truncate() {
        let x = 0x1122_3344_u32;
        let x8 = x as u8;
        assert_eq!(x8, 0x44u8);
    }
}
