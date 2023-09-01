// Copyright (c) 2023 Julia DeMille
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{cell::UnsafeCell, marker::PhantomData, rc::Rc};

use rustc_hash::FxHashMap;

use crate::NoSendSync;

pub struct StateData<T> {
    internal: Rc<UnsafeCell<T>>,
    _phantom: NoSendSync,
}

impl<T> StateData<T> {
    #[allow(dead_code)] // This code will not be dead in time.
    pub(crate) fn new(internal: Rc<UnsafeCell<T>>) -> Self {
        Self {
            internal,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn with_handle<U, V>(&mut self, closure: U) -> V
    where
        U: FnOnce(&mut T) -> V,
    {
        let i = unsafe { &mut *self.internal.get() };
        closure(i)
    }
}

#[allow(dead_code)] // Will be making this code not dead.
pub(crate) struct HandleableObjectMap<T> {
    internal: *mut FxHashMap<String, Rc<UnsafeCell<T>>>,
    _phantom: NoSendSync,
}

#[allow(dead_code)] // Will be making this code not dead.
impl<T> HandleableObjectMap<T> {
    pub(crate) fn new(p: *mut FxHashMap<String, Rc<UnsafeCell<T>>>) -> Self {
        Self {
            internal: p,
            _phantom: PhantomData,
        }
    }
    pub(crate) fn with_handle<U, V>(&mut self, closure: U) -> V
    where
        U: FnOnce(&mut FxHashMap<String, Rc<UnsafeCell<T>>>) -> V,
    {
        let i = unsafe { &mut *self.internal }; // internal will never be null.
        closure(i)
    }
}
