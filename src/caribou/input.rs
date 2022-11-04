use std::fmt::Debug;
use crate::bit_flags;
use crate::caribou::math::ScalarPair;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    Shift, Control, Alt, Meta,
}

bit_flags! {
    pub enum Modifier2: u32 {
        shift = 0b00000001,
        control = 0b00000010,
        alt = 0b00000100,
        meta = 0b00001000,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Key0,
    
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    Escape,
    
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,
    
    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    Backspace,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    NumLock,
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, 
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,
    // also called "Next"
    NavigateForward,
    // also called "Prior"
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MouseButton {
    Primary,
    Secondary,
    Tertiary,
    Other(u16),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DragInfo {
    pub button: MouseButton,
    pub begin: ScalarPair,
    pub current: ScalarPair,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MouseEvent {
    Enter,
    Leave,
    Move {
        position: ScalarPair,
        modifiers: Modifier2,
    },
    Button {
        position: ScalarPair,
        button: MouseButton,
        is_down: bool,
        modifiers: Modifier2,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyEvent {
    pub key: Key,
    pub is_down: bool,
    pub modifiers: Modifier2,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChainResult<E: Debug + Clone + PartialEq> {
    Capture,
    Propagate,
    Intercept(E),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FocusEvent {
    Gain,
    Lost,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FocusResult {
    Accept,
    Reject
}