// Copyright (c) 2023 Julia DeMille
// 
// Licensed under the EUPL, Version 1.2
// 
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

use std::{cell::UnsafeCell, marker::PhantomData, rc::Rc};

use slotmap::{SlotMap, Key, DefaultKey};

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
pub struct HandleableObjectMap<K: Key, V> {
    internal: *mut SlotMap<K, Rc<UnsafeCell<V>>>,
    _phantom: NoSendSync,
}

#[allow(dead_code)] // Will be making this code not dead.
impl<K, V> HandleableObjectMap<K, V> 
where K: Key{
    pub(crate) fn new(p: *mut SlotMap<K, Rc<UnsafeCell<V>>>) -> Self {
        Self {
            internal: p,
            _phantom: PhantomData,
        }
    }
    pub(crate) fn with_handle<F, T>(&mut self, closure: F) -> T
    where
        F: FnOnce(&mut SlotMap<K, Rc<UnsafeCell<V>>>) -> T,
    {
        let i = unsafe { &mut *self.internal }; // internal will never be null.
        closure(i)
    }
}
