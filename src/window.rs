// Copyright (c) 2023 Julia DeMille
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    mem,
    ops::Deref,
    os::raw::{c_char, c_int, c_void},
    ptr,
};

use xplane_sys;

use crate::make_x;

use super::geometry::{Point, Rect};

/// Cursor states that windows can apply
#[derive(Debug, Clone, Default)]
pub enum Cursor {
    /// X-Plane draws the default cursor
    #[default]
    Default,
    /// X-Plane draws an arrow cursor (not any other cursor type)
    Arrow,
    /// X-Plane hides the cursor. The plugin should draw its own cursor.
    None,
}

impl Cursor {
    /// Converts this cursor into an XPLMCursorStatus
    fn as_xplm(&self) -> xplane_sys::XPLMCursorStatus {
        match *self {
            Cursor::Default => xplane_sys::xplm_CursorDefault as xplane_sys::XPLMCursorStatus,
            Cursor::Arrow => xplane_sys::xplm_CursorArrow as xplane_sys::XPLMCursorStatus,
            Cursor::None => xplane_sys::xplm_CursorHidden as xplane_sys::XPLMCursorStatus,
        }
    }
}

/// Trait for things that can define the behavior of a window
pub trait WindowDelegate: 'static {
    /// Draws this window
    fn draw(&mut self, window: &Window);
    /// Handles a keyboard event
    ///
    /// The default implementation does nothing
    fn keyboard_event(&mut self, _window: &Window, _event: KeyEvent) {}
    /// Handles a mouse event
    ///
    /// Return false to consume the event or true to propagate it.
    ///
    /// The default implementation does nothing and allows the event to propagate.
    fn mouse_event(&mut self, _window: &Window, _event: MouseEvent) -> bool {
        true
    }
    /// Handles a scroll event
    ///
    /// Return false to consume the event or true to propagate it.
    ///
    /// The default implementation does nothing and allows the event to propagate.
    fn scroll_event(&mut self, _window: &Window, _event: ScrollEvent) -> bool {
        true
    }
    /// Tells X-Plane what cursor to draw over a section of the window
    ///
    /// The default implementation allows X-Plane to draw the default cursor.
    fn cursor(&mut self, _window: &Window, _position: Point<i32>) -> Cursor {
        Cursor::Default
    }
}

/// A reference to a window
pub struct WindowRef {
    /// The window
    window: Box<Window>,
}

impl Deref for WindowRef {
    type Target = Window;
    fn deref(&self) -> &Self::Target {
        self.window.deref()
    }
}

/// A basic window that may appear on the screen
///
/// A window has a position and size, but no appearance. Plugins must draw in their draw callbacks
/// to make windows appear.
pub struct Window {
    /// The window ID
    id: xplane_sys::XPLMWindowID,
    /// The delegate
    delegate: Box<dyn WindowDelegate>,
}

impl Window {
    /// Creates a new window with the provided geometry and returns a reference to it
    ///
    /// The window is originally not visible.
    pub fn create<R: Into<Rect<i32>>, D: WindowDelegate>(geometry: R, delegate: D) -> WindowRef {
        let geometry = geometry.into();

        let mut window_box = Box::new(Window {
            id: ptr::null_mut(),
            delegate: Box::new(delegate),
        });
        let window_ptr: *mut Window = &mut *window_box;

        let mut window_info = xplane_sys::XPLMCreateWindow_t {
            structSize: mem::size_of::<xplane_sys::XPLMCreateWindow_t>() as _,
            left: geometry.left(),
            top: geometry.top(),
            right: geometry.right(),
            bottom: geometry.bottom(),
            visible: 0,
            drawWindowFunc: Some(window_draw),
            handleMouseClickFunc: Some(window_mouse),
            handleKeyFunc: Some(window_key),
            handleCursorFunc: Some(window_cursor),
            handleMouseWheelFunc: Some(window_scroll),
            refcon: window_ptr as *mut _,
            decorateAsFloatingWindow: 0,
            layer: xplane_sys::xplm_WindowLayerFloatingWindows as _,
            handleRightClickFunc: None,
        };

        let window_id = unsafe { xplane_sys::XPLMCreateWindowEx(&mut window_info) };
        window_box.id = window_id;

        WindowRef { window: window_box }
    }

    /// Returns the geometry of this window
    pub fn geometry(&self) -> Rect<i32> {
        unsafe {
            let mut left = 0;
            let mut top = 0;
            let mut right = 0;
            let mut bottom = 0;
            xplane_sys::XPLMGetWindowGeometry(
                self.id,
                &mut left,
                &mut top,
                &mut right,
                &mut bottom,
            );
            Rect::from_left_top_right_bottom(left, top, right, bottom)
        }
    }
    /// Sets the geometry of this window
    pub fn set_geometry<R: Into<Rect<i32>>>(&self, geometry: R) {
        let geometry = geometry.into();
        unsafe {
            xplane_sys::XPLMSetWindowGeometry(
                self.id,
                geometry.left(),
                geometry.top(),
                geometry.right(),
                geometry.bottom(),
            );
        }
    }

    /// Returns true if this window is visible
    pub fn visible(&self) -> bool {
        1 == unsafe { xplane_sys::XPLMGetWindowIsVisible(self.id) }
    }
    /// Sets the window as visible or invisible
    pub fn set_visible(&self, visible: bool) {
        unsafe {
            xplane_sys::XPLMSetWindowIsVisible(self.id, visible as _);
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            xplane_sys::XPLMDestroyWindow(self.id);
        }
    }
}

/// Callback in which windows are drawn
unsafe extern "C" fn window_draw(_window: xplane_sys::XPLMWindowID, refcon: *mut c_void) {
    let window = refcon as *mut Window;
    (*window).delegate.draw(&*window);
}

/// Keyboard callback
unsafe extern "C" fn window_key(
    _window: xplane_sys::XPLMWindowID,
    key: c_char,
    flags: xplane_sys::XPLMKeyFlags,
    virtual_key: c_char,
    refcon: *mut c_void,
    losing_focus: c_int,
) {
    let window = refcon as *mut Window;
    if losing_focus == 0 {
        match KeyEvent::from_xplm(key, flags, virtual_key) {
            Ok(event) => (*window).delegate.keyboard_event(&*window, event),
            Err(e) => {
                let mut x = make_x();
                super::debugln!(x, "Invalid key event received: {:?}", e).unwrap()
                // This should always be a valid string.
            }
        }
    }
}

/// Mouse callback
unsafe extern "C" fn window_mouse(
    _window: xplane_sys::XPLMWindowID,
    x: c_int,
    y: c_int,
    status: xplane_sys::XPLMMouseStatus,
    refcon: *mut c_void,
) -> c_int {
    let window = refcon as *mut Window;
    if let Some(action) = MouseAction::from_xplm(status) {
        let position = Point::from((x, y));
        let event = MouseEvent::new(position, action);
        let propagate = (*window).delegate.mouse_event(&*window, event);
        if propagate {
            0
        } else {
            1
        }
    } else {
        // Propagate
        0
    }
}

/// Cursor callback
unsafe extern "C" fn window_cursor(
    _window: xplane_sys::XPLMWindowID,
    x: c_int,
    y: c_int,
    refcon: *mut c_void,
) -> xplane_sys::XPLMCursorStatus {
    let window = refcon as *mut Window;
    let cursor = (*window).delegate.cursor(&*window, Point::from((x, y)));
    cursor.as_xplm()
}

/// Scroll callback
unsafe extern "C" fn window_scroll(
    _window: xplane_sys::XPLMWindowID,
    x: c_int,
    y: c_int,
    wheel: c_int,
    clicks: c_int,
    refcon: *mut c_void,
) -> c_int {
    let window = refcon as *mut Window;

    let position = Point::from((x, y));
    let (dx, dy) = if wheel == 1 {
        // Horizontal
        (clicks, 0)
    } else {
        // Vertical
        (0, clicks)
    };
    let event = ScrollEvent::new(position, dx, dy);

    let propagate = (*window).delegate.scroll_event(&*window, event);
    if propagate {
        0
    } else {
        1
    }
}

/// Key actions
#[derive(Debug, Clone)]
pub enum KeyAction {
    /// The key was pressed down
    Press,
    /// The key was released
    Release,
}

/// Keys that may be pressed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Back,
    Tab,
    Clear,
    Return,
    Escape,
    Space,
    Prior,
    Next,
    End,
    Home,
    Left,
    Up,
    Right,
    Down,
    Select,
    Print,
    Execute,
    Snapshot,
    Insert,
    Delete,
    Help,
    /// The 0 key at the top of a keyboard
    Key0,
    /// The 1 key at the top of a keyboard
    Key1,
    /// The 2 key at the top of a keyboard
    Key2,
    /// The 3 key at the top of a keyboard
    Key3,
    /// The 4 key at the top of a keyboard
    Key4,
    /// The 5 key at the top of a keyboard
    Key5,
    /// The 6 key at the top of a keyboard
    Key6,
    /// The 7 key at the top of a keyboard
    Key7,
    /// The 8 key at the top of a keyboard
    Key8,
    /// The 9 key at the top of a keyboard
    Key9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    /// The 0 key on the numerical keypad
    Numpad0,
    /// The 1 key on the numerical keypad
    Numpad1,
    /// The 2 key on the numerical keypad
    Numpad2,
    /// The 3 key on the numerical keypad
    Numpad3,
    /// The 4 key on the numerical keypad
    Numpad4,
    /// The 5 key on the numerical keypad
    Numpad5,
    /// The 6 key on the numerical keypad
    Numpad6,
    /// The 7 key on the numerical keypad
    Numpad7,
    /// The 8 key on the numerical keypad
    Numpad8,
    /// The 9 key on the numerical keypad
    Numpad9,
    Multiply,
    Add,
    Separator,
    Subtract,
    Decimal,
    Divide,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Equal,
    Minus,
    ClosingBrace,
    OpeningBrace,
    Quote,
    Semicolon,
    Backslash,
    Comma,
    Slash,
    Period,
    Backquote,
    /// Enter, also known as return in Mac OS
    Enter,
    NumpadEnter,
    NumpadEqual,
}

impl Key {
    /// Converts an XPLM virtual key code into a Key
    fn from_xplm(xplm_key: c_char) -> Option<Self> {
        match xplm_key as u32 {
            xplane_sys::XPLM_VK_BACK => Some(Key::Back),
            xplane_sys::XPLM_VK_TAB => Some(Key::Tab),
            xplane_sys::XPLM_VK_CLEAR => Some(Key::Clear),
            xplane_sys::XPLM_VK_RETURN => Some(Key::Return),
            xplane_sys::XPLM_VK_ESCAPE => Some(Key::Escape),
            xplane_sys::XPLM_VK_SPACE => Some(Key::Space),
            xplane_sys::XPLM_VK_PRIOR => Some(Key::Prior),
            xplane_sys::XPLM_VK_NEXT => Some(Key::Next),
            xplane_sys::XPLM_VK_END => Some(Key::End),
            xplane_sys::XPLM_VK_HOME => Some(Key::Home),
            xplane_sys::XPLM_VK_LEFT => Some(Key::Left),
            xplane_sys::XPLM_VK_UP => Some(Key::Up),
            xplane_sys::XPLM_VK_RIGHT => Some(Key::Right),
            xplane_sys::XPLM_VK_DOWN => Some(Key::Down),
            xplane_sys::XPLM_VK_SELECT => Some(Key::Select),
            xplane_sys::XPLM_VK_PRINT => Some(Key::Print),
            xplane_sys::XPLM_VK_EXECUTE => Some(Key::Execute),
            xplane_sys::XPLM_VK_SNAPSHOT => Some(Key::Snapshot),
            xplane_sys::XPLM_VK_INSERT => Some(Key::Insert),
            xplane_sys::XPLM_VK_DELETE => Some(Key::Delete),
            xplane_sys::XPLM_VK_HELP => Some(Key::Help),
            xplane_sys::XPLM_VK_0 => Some(Key::Key0),
            xplane_sys::XPLM_VK_1 => Some(Key::Key1),
            xplane_sys::XPLM_VK_2 => Some(Key::Key2),
            xplane_sys::XPLM_VK_3 => Some(Key::Key3),
            xplane_sys::XPLM_VK_4 => Some(Key::Key4),
            xplane_sys::XPLM_VK_5 => Some(Key::Key5),
            xplane_sys::XPLM_VK_6 => Some(Key::Key6),
            xplane_sys::XPLM_VK_7 => Some(Key::Key7),
            xplane_sys::XPLM_VK_8 => Some(Key::Key8),
            xplane_sys::XPLM_VK_9 => Some(Key::Key9),
            xplane_sys::XPLM_VK_A => Some(Key::A),
            xplane_sys::XPLM_VK_B => Some(Key::B),
            xplane_sys::XPLM_VK_C => Some(Key::C),
            xplane_sys::XPLM_VK_D => Some(Key::D),
            xplane_sys::XPLM_VK_E => Some(Key::E),
            xplane_sys::XPLM_VK_F => Some(Key::F),
            xplane_sys::XPLM_VK_G => Some(Key::G),
            xplane_sys::XPLM_VK_H => Some(Key::H),
            xplane_sys::XPLM_VK_I => Some(Key::I),
            xplane_sys::XPLM_VK_J => Some(Key::J),
            xplane_sys::XPLM_VK_K => Some(Key::K),
            xplane_sys::XPLM_VK_L => Some(Key::L),
            xplane_sys::XPLM_VK_M => Some(Key::M),
            xplane_sys::XPLM_VK_N => Some(Key::N),
            xplane_sys::XPLM_VK_O => Some(Key::O),
            xplane_sys::XPLM_VK_P => Some(Key::P),
            xplane_sys::XPLM_VK_Q => Some(Key::Q),
            xplane_sys::XPLM_VK_R => Some(Key::R),
            xplane_sys::XPLM_VK_S => Some(Key::S),
            xplane_sys::XPLM_VK_T => Some(Key::T),
            xplane_sys::XPLM_VK_U => Some(Key::U),
            xplane_sys::XPLM_VK_V => Some(Key::V),
            xplane_sys::XPLM_VK_W => Some(Key::W),
            xplane_sys::XPLM_VK_X => Some(Key::X),
            xplane_sys::XPLM_VK_Y => Some(Key::Y),
            xplane_sys::XPLM_VK_Z => Some(Key::Z),
            xplane_sys::XPLM_VK_NUMPAD0 => Some(Key::Numpad0),
            xplane_sys::XPLM_VK_NUMPAD1 => Some(Key::Numpad1),
            xplane_sys::XPLM_VK_NUMPAD2 => Some(Key::Numpad2),
            xplane_sys::XPLM_VK_NUMPAD3 => Some(Key::Numpad3),
            xplane_sys::XPLM_VK_NUMPAD4 => Some(Key::Numpad4),
            xplane_sys::XPLM_VK_NUMPAD5 => Some(Key::Numpad5),
            xplane_sys::XPLM_VK_NUMPAD6 => Some(Key::Numpad6),
            xplane_sys::XPLM_VK_NUMPAD7 => Some(Key::Numpad7),
            xplane_sys::XPLM_VK_NUMPAD8 => Some(Key::Numpad8),
            xplane_sys::XPLM_VK_NUMPAD9 => Some(Key::Numpad9),
            xplane_sys::XPLM_VK_MULTIPLY => Some(Key::Multiply),
            xplane_sys::XPLM_VK_ADD => Some(Key::Add),
            xplane_sys::XPLM_VK_SEPARATOR => Some(Key::Separator),
            xplane_sys::XPLM_VK_SUBTRACT => Some(Key::Subtract),
            xplane_sys::XPLM_VK_DECIMAL => Some(Key::Decimal),
            xplane_sys::XPLM_VK_DIVIDE => Some(Key::Divide),
            xplane_sys::XPLM_VK_F1 => Some(Key::F1),
            xplane_sys::XPLM_VK_F2 => Some(Key::F2),
            xplane_sys::XPLM_VK_F3 => Some(Key::F3),
            xplane_sys::XPLM_VK_F4 => Some(Key::F4),
            xplane_sys::XPLM_VK_F5 => Some(Key::F5),
            xplane_sys::XPLM_VK_F6 => Some(Key::F6),
            xplane_sys::XPLM_VK_F7 => Some(Key::F7),
            xplane_sys::XPLM_VK_F8 => Some(Key::F8),
            xplane_sys::XPLM_VK_F9 => Some(Key::F9),
            xplane_sys::XPLM_VK_F10 => Some(Key::F10),
            xplane_sys::XPLM_VK_F11 => Some(Key::F11),
            xplane_sys::XPLM_VK_F12 => Some(Key::F12),
            xplane_sys::XPLM_VK_F13 => Some(Key::F13),
            xplane_sys::XPLM_VK_F14 => Some(Key::F14),
            xplane_sys::XPLM_VK_F15 => Some(Key::F15),
            xplane_sys::XPLM_VK_F16 => Some(Key::F16),
            xplane_sys::XPLM_VK_F17 => Some(Key::F17),
            xplane_sys::XPLM_VK_F18 => Some(Key::F18),
            xplane_sys::XPLM_VK_F19 => Some(Key::F19),
            xplane_sys::XPLM_VK_F20 => Some(Key::F20),
            xplane_sys::XPLM_VK_F21 => Some(Key::F21),
            xplane_sys::XPLM_VK_F22 => Some(Key::F22),
            xplane_sys::XPLM_VK_F23 => Some(Key::F23),
            xplane_sys::XPLM_VK_F24 => Some(Key::F24),
            xplane_sys::XPLM_VK_EQUAL => Some(Key::Equal),
            xplane_sys::XPLM_VK_MINUS => Some(Key::Minus),
            xplane_sys::XPLM_VK_RBRACE => Some(Key::ClosingBrace),
            xplane_sys::XPLM_VK_LBRACE => Some(Key::OpeningBrace),
            xplane_sys::XPLM_VK_QUOTE => Some(Key::Quote),
            xplane_sys::XPLM_VK_SEMICOLON => Some(Key::Semicolon),
            xplane_sys::XPLM_VK_BACKSLASH => Some(Key::Backslash),
            xplane_sys::XPLM_VK_COMMA => Some(Key::Comma),
            xplane_sys::XPLM_VK_SLASH => Some(Key::Slash),
            xplane_sys::XPLM_VK_PERIOD => Some(Key::Period),
            xplane_sys::XPLM_VK_BACKQUOTE => Some(Key::Backquote),
            xplane_sys::XPLM_VK_ENTER => Some(Key::Enter),
            xplane_sys::XPLM_VK_NUMPAD_ENT => Some(Key::NumpadEnter),
            xplane_sys::XPLM_VK_NUMPAD_EQ => Some(Key::NumpadEqual),
            _ => None,
        }
    }
}

/// An event associated with a key press
#[derive(Debug)]
pub struct KeyEvent {
    /// A character representing the key
    basic_char: Option<char>,
    /// The key
    key: Key,
    /// The action
    action: KeyAction,
    /// If the control key was pressed
    control_pressed: bool,
    /// If the option/alt key was pressed
    alt_pressed: bool,
    /// If the shift key was pressed
    shift_pressed: bool,
}

impl KeyEvent {
    /// Creates a key event from XPLM key information
    fn from_xplm(
        key: c_char,
        flags: xplane_sys::XPLMKeyFlags,
        virtual_key: c_char,
    ) -> Result<Self, KeyEventError> {
        let basic_char = match key as u8 {
            // Accept printable characters, including spaces and tabs
            b'\t' | b' '..=b'~' => Some(key as u8 as char),
            _ => None,
        };
        let action = if flags & xplane_sys::xplm_DownFlag as ::xplane_sys::XPLMKeyFlags != 0 {
            KeyAction::Press
        } else if flags & xplane_sys::xplm_UpFlag as ::xplane_sys::XPLMKeyFlags != 0 {
            KeyAction::Release
        } else {
            return Err(KeyEventError::InvalidFlags(flags));
        };
        let control_pressed =
            flags & xplane_sys::xplm_ControlFlag as ::xplane_sys::XPLMKeyFlags != 0;
        let shift_pressed = flags & xplane_sys::xplm_ShiftFlag as ::xplane_sys::XPLMKeyFlags != 0;
        let option_pressed =
            flags & xplane_sys::xplm_OptionAltFlag as ::xplane_sys::XPLMKeyFlags != 0;
        let key = match Key::from_xplm(virtual_key) {
            Some(key) => key,
            None => return Err(KeyEventError::InvalidKey(virtual_key)),
        };

        Ok(KeyEvent {
            basic_char,
            key,
            action,
            control_pressed,
            alt_pressed: option_pressed,
            shift_pressed,
        })
    }
    /// Returns the character corresponding to the key associated with this event, if one exists
    ///
    /// Some key combinations, including combinations with non-Shift modifiers, may not have
    /// corresponding characters.
    pub fn char(&self) -> Option<char> {
        self.basic_char
    }
    /// Returns the key associated with this event
    pub fn key(&self) -> Key {
        self.key.clone()
    }
    /// Returns true if the control key was held down when the action occurred
    pub fn control_pressed(&self) -> bool {
        self.control_pressed
    }
    /// Returns true if the option/alt key was held down when the action occurred
    pub fn option_pressed(&self) -> bool {
        self.alt_pressed
    }
    /// Returns true if a shift key was held down when the action occurred
    pub fn shift_pressed(&self) -> bool {
        self.shift_pressed
    }
    /// Returns the key action that occurred
    pub fn action(&self) -> KeyAction {
        self.action.clone()
    }
}

/// Key event creation error
#[derive(thiserror::Error, Debug)]
enum KeyEventError {
    #[error("Unexpected key flags {0:b}")]
    InvalidFlags(xplane_sys::XPLMKeyFlags),

    #[error("Invalid or unsupported key with code: 0x{0:x}")]
    InvalidKey(c_char),
}

/// Actions that the mouse/cursor can perform
#[derive(Debug, Clone)]
pub enum MouseAction {
    /// The user pressed the mouse button down
    Down,
    /// The user moved the mouse with the mouse button down
    Drag,
    /// The user released the mouse button
    Up,
}

impl MouseAction {
    fn from_xplm(status: xplane_sys::XPLMMouseStatus) -> Option<MouseAction> {
        if status == xplane_sys::xplm_MouseDown as xplane_sys::XPLMMouseStatus {
            Some(MouseAction::Down)
        } else if status == xplane_sys::xplm_MouseDrag as xplane_sys::XPLMMouseStatus {
            Some(MouseAction::Drag)
        } else if status == xplane_sys::xplm_MouseUp as xplane_sys::XPLMMouseStatus {
            Some(MouseAction::Up)
        } else {
            None
        }
    }
}

/// A mouse event
#[derive(Debug)]
pub struct MouseEvent {
    /// The position of the mouse, in global window coordinates
    position: Point<i32>,
    /// The action of the mouse
    action: MouseAction,
}

impl MouseEvent {
    /// Creates a new event
    fn new(position: Point<i32>, action: MouseAction) -> Self {
        MouseEvent { position, action }
    }
    /// Returns the position of the mouse, in global coordinates relative to the X-Plane
    /// main window
    pub fn position(&self) -> Point<i32> {
        self.position
    }
    /// Returns the action that the user performed with the mouse
    pub fn action(&self) -> MouseAction {
        self.action.clone()
    }
}

/// A scroll event
#[derive(Debug, Clone)]
pub struct ScrollEvent {
    /// The position of the mouse, in global window coordinates
    position: Point<i32>,
    /// The amount of scroll in the X direction
    scroll_x: i32,
    /// The amount of scroll in the Y direction
    scroll_y: i32,
}

impl ScrollEvent {
    /// Creates a new event
    fn new(position: Point<i32>, scroll_x: i32, scroll_y: i32) -> Self {
        ScrollEvent {
            position,
            scroll_x,
            scroll_y,
        }
    }
    /// Returns the position of the mouse, in global coordinates relative to the X-Plane
    /// main window
    pub fn position(&self) -> Point<i32> {
        self.position
    }
    /// Returns the amount of scroll in the X direction
    pub fn scroll_x(&self) -> i32 {
        self.scroll_x
    }
    /// Returns the amount of scroll in the Y direction
    pub fn scroll_y(&self) -> i32 {
        self.scroll_y
    }
}
