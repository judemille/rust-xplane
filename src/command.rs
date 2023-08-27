// Copyright (c) 2023 Julia DeMille
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    ffi::{CString, NulError},
    marker::PhantomData,
    ops::DerefMut,
    os::raw::{c_int, c_void},
};

use xplane_sys::{
    xplm_CommandBegin, xplm_CommandContinue, xplm_CommandEnd, XPLMCommandBegin,
    XPLMCommandCallback_f, XPLMCommandEnd, XPLMCommandOnce, XPLMCommandPhase, XPLMCommandRef,
    XPLMCreateCommand, XPLMFindCommand, XPLMRegisterCommandHandler, XPLMUnregisterCommandHandler,
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
    pub fn find(name: &str) -> Result<Self, CommandFindError> {
        let name_c = CString::new(name)?;
        let command_ref = unsafe { XPLMFindCommand(name_c.as_ptr()) };
        if !command_ref.is_null() {
            Ok(Command {
                id: command_ref,
                _phantom: PhantomData,
            })
        } else {
            Err(CommandFindError::NotFound)
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
    pub fn hold_down<'a>(&'a mut self) -> CommandHold<'a> {
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
pub trait CommandHandler: 'static {
    /// Called when the command begins (corresponds to a button being pressed down)
    fn command_begin(&mut self);
    /// Called frequently while the command button is held down
    fn command_continue(&mut self);
    /// Called when the command ends (corresponds to a button being released)
    fn command_end(&mut self);
}

/// A command created by this plugin that can be triggered by other components
pub struct OwnedCommand {
    /// The heap-allocated data
    data: Box<OwnedCommandData>,
    /// The handler callback, used to unregister
    callback: XPLMCommandCallback_f,
}

impl OwnedCommand {
    /// Creates a new command with a provided name and description
    ///
    /// Returns an error if a matching command already exists.
    pub fn new<H: CommandHandler>(
        name: &str,
        description: &str,
        handler: H,
    ) -> Result<Self, CommandCreateError> {
        let mut data = Box::new(OwnedCommandData::new(name, description, handler)?);
        let data_ptr: *mut OwnedCommandData = data.deref_mut();
        unsafe {
            XPLMRegisterCommandHandler(
                data.id,
                Some(command_handler::<H>),
                1,
                data_ptr as *mut c_void,
            );
        }
        Ok(OwnedCommand {
            data,
            callback: Some(command_handler::<H>),
        })
    }
}

impl Drop for OwnedCommand {
    fn drop(&mut self) {
        let data_ptr: *mut OwnedCommandData = self.data.deref_mut();
        unsafe {
            XPLMUnregisterCommandHandler(self.data.id, self.callback, 1, data_ptr as *mut c_void);
        }
    }
}

/// Data for an owned command, used as a refcon
struct OwnedCommandData {
    /// The command reference
    id: XPLMCommandRef,
    /// The handler
    handler: Box<dyn CommandHandler>,
}

impl OwnedCommandData {
    pub fn new<H: CommandHandler>(
        name: &str,
        description: &str,
        handler: H,
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
        })
    }
}

/// Command handler callback
unsafe extern "C" fn command_handler<H: CommandHandler>(
    _: XPLMCommandRef,
    phase: XPLMCommandPhase,
    refcon: *mut c_void,
) -> c_int {
    let data = refcon as *mut OwnedCommandData;
    let handler: *mut dyn CommandHandler = (*data).handler.deref_mut();
    let handler = handler as *mut H;
    if phase == xplm_CommandBegin as i32 {
        (*handler).command_begin();
    } else if phase == xplm_CommandContinue as i32 {
        (*handler).command_continue();
    } else if phase == xplm_CommandEnd as i32 {
        (*handler).command_end();
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
