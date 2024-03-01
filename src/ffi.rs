// SPDX-FileCopyrightText: 2024 Julia DeMille <me@jdemille.com>
//
// SPDX-License-Identifier: MPL-2.0

//! Foreign function interface utilities
//!

use std::{ffi::CString, iter, str::Utf8Error, string::FromUtf8Error};

use std::ffi::c_char;

/// A fixed-length array of characters that can be passed to C functions and converted into a
/// String
#[derive(Debug)]
pub struct StringBuffer {
    /// The bytes in this buffer
    bytes: Vec<u8>,
}

impl StringBuffer {
    /// Creates a new `StringBuffer` with the provided length in bytes. All bytes in the string are
    /// set to null bytes (`\0`).
    pub fn new(length: usize) -> StringBuffer {
        StringBuffer {
            bytes: vec![b'\0'; length],
        }
    }

    /// Returns a mutable pointer to the data in this buffer
    pub unsafe fn as_mut_ptr(&mut self) -> *mut c_char {
        self.bytes.as_mut_ptr().cast::<c_char>()
    }

    /// Returns the bytes in this buffer
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns a mutable slice into the bytes in this buffer
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    /// Returns a reference to the string in this buffer
    ///
    /// The returned string will not contain any null bytes.
    ///
    /// An error is returned if the data in this buffer is not valid UTF-8.
    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        let chars_before_null = self.bytes.iter().take_while(|&&c| c != b'\0').count();
        std::str::from_utf8(&self.bytes[..chars_before_null])
    }

    /// Converts this buffer into a String
    ///
    /// The returned string will not contain any null bytes.
    ///
    /// An error is returned if the data in this buffer is not valid UTF-8.
    pub fn into_string(self) -> Result<String, FromUtf8Error> {
        let chars_before_null = self.bytes.into_iter().take_while(|&c| c != b'\0');
        String::from_utf8(chars_before_null.collect())
    }
}

impl From<StringBuffer> for CString {
    fn from(StringBuffer { mut bytes }: StringBuffer) -> Self {
        if let Some(i) = bytes.iter().position(|&b| b == b'\0') {
            bytes.truncate(i + 1);
        } else {
            bytes.reserve_exact(1);
            bytes.push(b'\0');
        }
        bytes.shrink_to_fit();
        unsafe { CString::from_vec_with_nul_unchecked(bytes) }
    }
}
