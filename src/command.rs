// Copyright (c) 2023 Julia DeMille
//
// Licensed under the EUPL, Version 1.2
//
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

use core::ffi::{c_int, c_void};
use std::{
    cell::RefCell,
    ffi::{CString, NulError},
    marker::PhantomData,
};

use xplane_sys::{
    XPLMCommandBegin, XPLMCommandCallback_f, XPLMCommandEnd, XPLMCommandOnce, XPLMCommandPhase,
    XPLMCommandRef, XPLMCreateCommand, XPLMFindCommand, XPLMRegisterCommandHandler,
    XPLMUnregisterCommandHandler,
};

use crate::NoSendSync;

pub struct CommandAPI {
    _phantom: NoSendSync,
}

impl CommandAPI {}

/// A command created by X-Plane or another plugin, that can be triggered
#[derive(Debug)]
pub struct Command {
    /// The command reference
    id: XPLMCommandRef,
    _phantom: NoSendSync,
}

impl Command {
    /// Finds a command
    ///
    /// The command should have already been created by X-Plane or another plugin.
    /// # Errors
    /// Errors if command could not be found.
    pub fn find(name: &str) -> Result<Self, CommandFindError> {
        let name_c = CString::new(name)?;
        let command_ref = unsafe { XPLMFindCommand(name_c.as_ptr()) };
        if command_ref.is_null() {
            Err(CommandFindError::NotFound)
        } else {
            Ok(Command {
                id: command_ref,
                _phantom: PhantomData,
            })
        }
    }

    /// Triggers a command once
    ///
    /// This is equivalent to pressing a button down and immediately releasing it.
    pub fn trigger(&mut self) {
        unsafe {
            XPLMCommandOnce(self.id);
        }
    }

    /// Starts holding down this command
    ///
    /// The command will be released when the returned hold object is dropped.
    pub fn hold_down(&'_ mut self) -> CommandHold<'_> {
        unsafe {
            XPLMCommandBegin(self.id);
        }
        CommandHold { command: self }
    }

    /// Releases this command
    fn release(&mut self) {
        unsafe {
            XPLMCommandEnd(self.id);
        }
    }
}

/// An RAII lock that keeps a command held down
///
/// The command will be released when this object is dropped.
#[derive(Debug)]
pub struct CommandHold<'a> {
    /// The command being held
    command: &'a mut Command,
}

impl<'a> Drop for CommandHold<'a> {
    fn drop(&mut self) {
        self.command.release();
    }
}

/// Errors that can occur when finding a command
#[derive(thiserror::Error, Debug)]
pub enum CommandFindError {
    /// The provided command name contained a null byte
    #[error("Null byte in command name")]
    Null(#[from] NulError),

    /// The Command could not be found
    #[error("Command not found")]
    NotFound,
}

/// Trait for things that can handle commands
pub trait CommandHandler<T>: 'static {
    /// Called when the command begins (corresponds to a button being pressed down)
    fn command_begin(&mut self, state: &mut T);
    /// Called frequently while the command button is held down
    fn command_continue(&mut self, state: &mut T);
    /// Called when the command ends (corresponds to a button being released)
    fn command_end(&mut self, state: &mut T);
}

/// A command created by this plugin that can be triggered by other components
pub struct OwnedCommand<T> {
    /// The heap-allocated data
    data: Box<OwnedCommandData<T>>,
    /// The handler callback, used to unregister
    callback: XPLMCommandCallback_f,
}

impl<T> OwnedCommand<T> {
    /// Creates a new command with a provided name and description
    /// # Errors
    /// Returns an error if a matching command already exists.
    pub fn new<H: CommandHandler<T>>(
        name: &str,
        description: &str,
        handler: H,
        base_state: T,
    ) -> Result<Self, CommandCreateError> {
        let mut data = Box::new(OwnedCommandData::new(
            name,
            description,
            handler,
            base_state,
        )?);
        let data_ptr: *mut OwnedCommandData<T> = &mut *data;
        unsafe {
            XPLMRegisterCommandHandler(
                data.id,
                Some(command_handler::<T, H>),
                1,
                data_ptr.cast::<c_void>(),
            );
        }
        Ok(OwnedCommand {
            data,
            callback: Some(command_handler::<T, H>),
        })
    }
}

impl<T> Drop for OwnedCommand<T> {
    fn drop(&mut self) {
        let data_ptr: *mut OwnedCommandData<T> = &mut *self.data;
        unsafe {
            XPLMUnregisterCommandHandler(self.data.id, self.callback, 1, data_ptr.cast::<c_void>());
        }
    }
}

/// Data for an owned command, used as a refcon
struct OwnedCommandData<T> {
    /// The command reference
    id: XPLMCommandRef,
    /// The handler
    handler: Box<dyn CommandHandler<T>>,
    /// The state data.
    state: RefCell<T>,
}

impl<T> OwnedCommandData<T> {
    pub fn new<H: CommandHandler<T>>(
        name: &str,
        description: &str,
        handler: H,
        base_state: T,
    ) -> Result<Self, CommandCreateError> {
        let name_c = CString::new(name)?;
        let description_c = CString::new(description)?;

        let existing = unsafe { XPLMFindCommand(name_c.as_ptr()) };
        if !existing.is_null() {
            return Err(CommandCreateError::Exists);
        }

        // Command does not exist, proceed
        let command_id = unsafe { XPLMCreateCommand(name_c.as_ptr(), description_c.as_ptr()) };
        Ok(OwnedCommandData {
            id: command_id,
            handler: Box::new(handler),
            state: RefCell::new(base_state),
        })
    }
}

#[allow(clippy::cast_possible_wrap)]
/// Command handler callback
unsafe extern "C" fn command_handler<T, H: CommandHandler<T>>(
    _: XPLMCommandRef,
    phase: XPLMCommandPhase,
    refcon: *mut c_void,
) -> c_int {
    let data = refcon.cast::<OwnedCommandData<T>>();
    let handler: *mut dyn CommandHandler<T> = &mut *(*data).handler;
    let handler = handler.cast::<H>();
    let state = (*data).state.get_mut(); // This should hopefully not cause issues. Leaving the check in to avoid UB.
    if phase == XPLMCommandPhase::Begin {
        (*handler).command_begin(state);
    } else if phase == XPLMCommandPhase::Continue {
        (*handler).command_continue(state);
    } else if phase == XPLMCommandPhase::End {
        (*handler).command_end(state);
    }
    // Prevent other components from handling this equivalent
    0
}

/// Errors that can occur when creating a Command
#[derive(thiserror::Error, Debug)]
pub enum CommandCreateError {
    /// The provided Command name contained a null byte
    #[error("Null byte in Command name")]
    Null(#[from] NulError),

    /// The Command exists already
    #[error("Command exists already")]
    Exists,
}
