use crate::input::*;

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardButton {
    /// <kbd>`</kbd> on a US keyboard. This is also called a backtick or grave.
    /// This is the <kbd>半角</kbd>/<kbd>全角</kbd>/<kbd>漢字</kbd>
    /// (hankaku/zenkaku/kanji) key on Japanese keyboards
    Backquote,
    /// Used for both the US <kbd>\\</kbd> (on the 101-key layout) and also for the key
    /// located between the <kbd>"</kbd> and <kbd>Enter</kbd> keys on row C of the 102-,
    /// 104- and 106-key layouts.
    /// Labeled <kbd>#</kbd> on a UK (102) keyboard.
    Backslash,
    /// <kbd>[</kbd> on a US keyboard.
    BracketLeft,
    /// <kbd>]</kbd> on a US keyboard.
    BracketRight,
    /// <kbd>,</kbd> on a US keyboard.
    Comma,
    /// <kbd>0</kbd> on a US keyboard.
    Key0,
    /// <kbd>1</kbd> on a US keyboard.
    Key1,
    /// <kbd>2</kbd> on a US keyboard.
    Key2,
    /// <kbd>3</kbd> on a US keyboard.
    Key3,
    /// <kbd>4</kbd> on a US keyboard.
    Key4,
    /// <kbd>5</kbd> on a US keyboard.
    Key5,
    /// <kbd>6</kbd> on a US keyboard.
    Key6,
    /// <kbd>7</kbd> on a US keyboard.
    Key7,
    /// <kbd>8</kbd> on a US keyboard.
    Key8,
    /// <kbd>9</kbd> on a US keyboard.
    Key9,
    /// <kbd>=</kbd> on a US keyboard.
    Equal,
    /// Located between the left <kbd>Shift</kbd> and <kbd>Z</kbd> keys.
    /// Labeled <kbd>\\</kbd> on a UK keyboard.
    IntlBackslash,
    /// Located between the <kbd>/</kbd> and right <kbd>Shift</kbd> keys.
    /// Labeled <kbd>\\</kbd> (ro) on a Japanese keyboard.
    IntlRo,
    /// Located between the <kbd>=</kbd> and <kbd>Backspace</kbd> keys.
    /// Labeled <kbd>¥</kbd> (yen) on a Japanese keyboard. <kbd>\\</kbd> on a
    /// Russian keyboard.
    IntlYen,
    /// <kbd>a</kbd> on a US keyboard.
    /// Labeled <kbd>q</kbd> on an AZERTY (e.g., French) keyboard.
    A,
    /// <kbd>b</kbd> on a US keyboard.
    B,
    /// <kbd>c</kbd> on a US keyboard.
    C,
    /// <kbd>d</kbd> on a US keyboard.
    D,
    /// <kbd>e</kbd> on a US keyboard.
    E,
    /// <kbd>f</kbd> on a US keyboard.
    F,
    /// <kbd>g</kbd> on a US keyboard.
    G,
    /// <kbd>h</kbd> on a US keyboard.
    H,
    /// <kbd>i</kbd> on a US keyboard.
    I,
    /// <kbd>j</kbd> on a US keyboard.
    J,
    /// <kbd>k</kbd> on a US keyboard.
    K,
    /// <kbd>l</kbd> on a US keyboard.
    L,
    /// <kbd>m</kbd> on a US keyboard.
    M,
    /// <kbd>n</kbd> on a US keyboard.
    N,
    /// <kbd>o</kbd> on a US keyboard.
    O,
    /// <kbd>p</kbd> on a US keyboard.
    P,
    /// <kbd>q</kbd> on a US keyboard.
    /// Labeled <kbd>a</kbd> on an AZERTY (e.g., French) keyboard.
    Q,
    /// <kbd>r</kbd> on a US keyboard.
    R,
    /// <kbd>s</kbd> on a US keyboard.
    S,
    /// <kbd>t</kbd> on a US keyboard.
    T,
    /// <kbd>u</kbd> on a US keyboard.
    U,
    /// <kbd>v</kbd> on a US keyboard.
    V,
    /// <kbd>w</kbd> on a US keyboard.
    /// Labeled <kbd>z</kbd> on an AZERTY (e.g., French) keyboard.
    W,
    /// <kbd>x</kbd> on a US keyboard.
    X,
    /// <kbd>y</kbd> on a US keyboard.
    /// Labeled <kbd>z</kbd> on a QWERTZ (e.g., German) keyboard.
    Y,
    /// <kbd>z</kbd> on a US keyboard.
    /// Labeled <kbd>w</kbd> on an AZERTY (e.g., French) keyboard, and <kbd>y</kbd> on a
    /// QWERTZ (e.g., German) keyboard.
    Z,
    /// <kbd>-</kbd> on a US keyboard.
    Minus,
    /// <kbd>.</kbd> on a US keyboard.
    Period,
    /// <kbd>'</kbd> on a US keyboard.
    Quote,
    /// <kbd>;</kbd> on a US keyboard.
    Semicolon,
    /// <kbd>/</kbd> on a US keyboard.
    Slash,
    /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
    AltLeft,
    /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
    /// This is labeled <kbd>AltGr</kbd> on many keyboard layouts.
    AltRight,
    /// <kbd>Backspace</kbd> or <kbd>⌫</kbd>.
    /// Labeled <kbd>Delete</kbd> on Apple keyboards.
    Backspace,
    /// <kbd>CapsLock</kbd> or <kbd>⇪</kbd>
    CapsLock,
    /// The application context menu key, which is typically found between the right
    /// <kbd>Super</kbd> key and the right <kbd>Control</kbd> key.
    ContextMenu,
    /// <kbd>Control</kbd> or <kbd>⌃</kbd>
    ControlLeft,
    /// <kbd>Control</kbd> or <kbd>⌃</kbd>
    ControlRight,
    /// <kbd>Enter</kbd> or <kbd>↵</kbd>. Labeled <kbd>Return</kbd> on Apple keyboards.
    Enter,
    /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
    SuperLeft,
    /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
    SuperRight,
    /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
    ShiftLeft,
    /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
    ShiftRight,
    /// <kbd> </kbd> (space)
    Space,
    /// <kbd>Tab</kbd> or <kbd>⇥</kbd>
    Tab,
    /// Japanese: <kbd>変</kbd> (henkan)
    Convert,
    /// Japanese: <kbd>カタカナ</kbd>/<kbd>ひらがな</kbd>/<kbd>ローマ字</kbd> (katakana/hiragana/romaji)
    KanaMode,
    /// Korean: HangulMode <kbd>한/영</kbd> (han/yeong)
    ///
    /// Japanese (Mac keyboard): <kbd>か</kbd> (kana)
    Lang1,
    /// Korean: Hanja <kbd>한</kbd> (hanja)
    ///
    /// Japanese (Mac keyboard): <kbd>英</kbd> (eisu)
    Lang2,
    /// Japanese (word-processing keyboard): Katakana
    Lang3,
    /// Japanese (word-processing keyboard): Hiragana
    Lang4,
    /// Japanese (word-processing keyboard): Zenkaku/Hankaku
    Lang5,
    /// Japanese: <kbd>無変換</kbd> (muhenkan)
    NonConvert,
    /// <kbd>⌦</kbd>. The forward delete key.
    /// Note that on Apple keyboards, the key labelled <kbd>Delete</kbd> on the main part of
    /// the keyboard is encoded as [`Backspace`].
    ///
    /// [`Backspace`]: Self::Backspace
    Delete,
    /// <kbd>Page Down</kbd>, <kbd>End</kbd>, or <kbd>↘</kbd>
    End,
    /// <kbd>Help</kbd>. Not present on standard PC keyboards.
    Help,
    /// <kbd>Home</kbd> or <kbd>↖</kbd>
    Home,
    /// <kbd>Insert</kbd> or <kbd>Ins</kbd>. Not present on Apple keyboards.
    Insert,
    /// <kbd>Page Down</kbd>, <kbd>PgDn</kbd>, or <kbd>⇟</kbd>
    PageDown,
    /// <kbd>Page Up</kbd>, <kbd>PgUp</kbd>, or <kbd>⇞</kbd>
    PageUp,
    /// <kbd>↓</kbd>
    Down,
    /// <kbd>←</kbd>
    Left,
    /// <kbd>→</kbd>
    Right,
    /// <kbd>↑</kbd>
    Up,
    /// On the Mac, this is used for the numpad <kbd>Clear</kbd> key.
    NumLock,
    /// <kbd>0 Ins</kbd> on a keyboard. <kbd>0</kbd> on a phone or remote control
    Numpad0,
    /// <kbd>1 End</kbd> on a keyboard. <kbd>1</kbd> or <kbd>1 QZ</kbd> on a phone or remote control
    Numpad1,
    /// <kbd>2 ↓</kbd> on a keyboard. <kbd>2 ABC</kbd> on a phone or remote control
    Numpad2,
    /// <kbd>3 PgDn</kbd> on a keyboard. <kbd>3 DEF</kbd> on a phone or remote control
    Numpad3,
    /// <kbd>4 ←</kbd> on a keyboard. <kbd>4 GHI</kbd> on a phone or remote control
    Numpad4,
    /// <kbd>5</kbd> on a keyboard. <kbd>5 JKL</kbd> on a phone or remote control
    Numpad5,
    /// <kbd>6 →</kbd> on a keyboard. <kbd>6 MNO</kbd> on a phone or remote control
    Numpad6,
    /// <kbd>7 Home</kbd> on a keyboard. <kbd>7 PQRS</kbd> or <kbd>7 PRS</kbd> on a phone
    /// or remote control
    Numpad7,
    /// <kbd>8 ↑</kbd> on a keyboard. <kbd>8 TUV</kbd> on a phone or remote control
    Numpad8,
    /// <kbd>9 PgUp</kbd> on a keyboard. <kbd>9 WXYZ</kbd> or <kbd>9 WXY</kbd> on a phone
    /// or remote control
    Numpad9,
    /// <kbd>+</kbd>
    NumpadAdd,
    /// Found on the Microsoft Natural Keyboard.
    NumpadBackspace,
    /// <kbd>C</kbd> or <kbd>A</kbd> (All Clear). Also for use with numpads that have a
    /// <kbd>Clear</kbd> key that is separate from the <kbd>NumLock</kbd> key. On the Mac, the
    /// numpad <kbd>Clear</kbd> key is encoded as [`NumLock`].
    ///
    /// [`NumLock`]: Self::NumLock
    NumpadClear,
    /// <kbd>C</kbd> (Clear Entry)
    NumpadClearEntry,
    /// <kbd>,</kbd> (thousands separator). For locales where the thousands separator
    /// is a "." (e.g., Brazil), this key may generate a <kbd>.</kbd>.
    NumpadComma,
    /// <kbd>. Del</kbd>. For locales where the decimal separator is "," (e.g.,
    /// Brazil), this key may generate a <kbd>,</kbd>.
    NumpadDecimal,
    /// <kbd>/</kbd>
    NumpadDivide,
    NumpadEnter,
    /// <kbd>=</kbd>
    NumpadEqual,
    /// <kbd>#</kbd> on a phone or remote control device. This key is typically found
    /// below the <kbd>9</kbd> key and to the right of the <kbd>0</kbd> key.
    NumpadHash,
    /// <kbd>M</kbd> Add current entry to the value stored in memory.
    NumpadMemoryAdd,
    /// <kbd>M</kbd> Clear the value stored in memory.
    NumpadMemoryClear,
    /// <kbd>M</kbd> Replace the current entry with the value stored in memory.
    NumpadMemoryRecall,
    /// <kbd>M</kbd> Replace the value stored in memory with the current entry.
    NumpadMemoryStore,
    /// <kbd>M</kbd> Subtract current entry from the value stored in memory.
    NumpadMemorySubtract,
    /// <kbd>*</kbd> on a keyboard. For use with numpads that provide mathematical
    /// operations (<kbd>+</kbd>, <kbd>-</kbd> <kbd>*</kbd> and <kbd>/</kbd>).
    ///
    /// Use `NumpadStar` for the <kbd>*</kbd> key on phones and remote controls.
    NumpadMultiply,
    /// <kbd>(</kbd> Found on the Microsoft Natural Keyboard.
    NumpadParenLeft,
    /// <kbd>)</kbd> Found on the Microsoft Natural Keyboard.
    NumpadParenRight,
    /// <kbd>*</kbd> on a phone or remote control device.
    ///
    /// This key is typically found below the <kbd>7</kbd> key and to the left of
    /// the <kbd>0</kbd> key.
    ///
    /// Use <kbd>"NumpadMultiply"</kbd> for the <kbd>*</kbd> key on
    /// numeric keypads.
    NumpadStar,
    /// <kbd>-</kbd>
    NumpadSubtract,
    /// <kbd>Esc</kbd> or <kbd>⎋</kbd>
    Escape,
    /// <kbd>Fn</kbd> This is typically a hardware key that does not generate a separate code.
    Fn,
    /// <kbd>FLock</kbd> or <kbd>FnLock</kbd>. Function Lock key. Found on the Microsoft
    /// Natural Keyboard.
    FnLock,
    /// <kbd>PrtScr SysRq</kbd> or <kbd>Print Screen</kbd>
    PrintScreen,
    /// <kbd>Scroll Lock</kbd>
    ScrollLock,
    /// <kbd>Pause Break</kbd>
    Pause,
    /// Some laptops place this key to the left of the <kbd>↑</kbd> key.
    ///
    /// This also the "back" button (triangle) on Android.
    BrowserBack,
    BrowserFavorites,
    /// Some laptops place this key to the right of the <kbd>↑</kbd> key.
    BrowserForward,
    /// The "home" button on Android.
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    /// <kbd>Eject</kbd> or <kbd>⏏</kbd>. This key is placed in the function section on some Apple
    /// keyboards.
    Eject,
    /// Sometimes labelled <kbd>My Computer</kbd> on the keyboard
    LaunchApp1,
    /// Sometimes labelled <kbd>Calculator</kbd> on the keyboard
    LaunchApp2,
    LaunchMail,
    MediaPlayPause,
    MediaSelect,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    /// This key is placed in the function section on some Apple keyboards, replacing the
    /// <kbd>Eject</kbd> key.
    Power,
    Sleep,
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    WakeUp,
    // Legacy modifier key. Also called "Super" in certain places.
    Meta,
    // Legacy modifier key.
    Hyper,
    Turbo,
    Abort,
    Resume,
    Suspend,
    /// Found on Sun’s USB keyboard.
    Again,
    /// Found on Sun’s USB keyboard.
    Copy,
    /// Found on Sun’s USB keyboard.
    Cut,
    /// Found on Sun’s USB keyboard.
    Find,
    /// Found on Sun’s USB keyboard.
    Open,
    /// Found on Sun’s USB keyboard.
    Paste,
    /// Found on Sun’s USB keyboard.
    Props,
    /// Found on Sun’s USB keyboard.
    Select,
    /// Found on Sun’s USB keyboard.
    Undo,
    /// Use for dedicated <kbd>ひらがな</kbd> key found on some Japanese word processing keyboards.
    Hiragana,
    /// Use for dedicated <kbd>カタカナ</kbd> key found on some Japanese word processing keyboards.
    Katakana,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F1,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F2,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F3,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F4,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F5,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F6,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F7,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F8,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F9,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F10,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F11,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F12,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F13,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F14,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F15,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F16,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F17,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F18,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F19,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F20,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F21,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F22,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F23,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F24,
    /// General-purpose function key.
    F25,
    /// General-purpose function key.
    F26,
    /// General-purpose function key.
    F27,
    /// General-purpose function key.
    F28,
    /// General-purpose function key.
    F29,
    /// General-purpose function key.
    F30,
    /// General-purpose function key.
    F31,
    /// General-purpose function key.
    F32,
    /// General-purpose function key.
    F33,
    /// General-purpose function key.
    F34,
    /// General-purpose function key.
    F35,
}

#[derive(Default)]
pub struct KeyboardState {
    control_state: ControlState,
    button_state: ButtonState<{ KeyboardButton::SIZE }>,
    /// Entered text. Usually a single character.
    text: String,
}

impl KeyboardState {
    pub fn control_state(&self) -> &ControlState {
        &self.control_state
    }

    pub fn control_state_mut(&mut self) -> &mut ControlState {
        &mut self.control_state
    }

    pub fn reset(&mut self) {
        self.button_state.reset();
        self.text.clear();
    }

    pub fn update(&mut self, button: KeyboardButton, pressed: bool) -> bool {
        self.button_state.update(button as usize, pressed) && self.control_state.update()
    }

    pub fn set_text(&mut self, text: &str) -> bool {
        self.text = text.into();
        true
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl KeyboardState {
    pub fn is_pressed(&self, button: KeyboardButton) -> bool {
        self.button_state.is_pressed(button as usize)
    }

    pub fn is_down(&self, button: KeyboardButton) -> bool {
        self.button_state.is_down(button as usize)
    }

    pub fn is_released(&self, button: KeyboardButton) -> bool {
        self.button_state.is_released(button as usize)
    }

    pub fn alt_pressed(&self) -> bool {
        self.is_pressed(KeyboardButton::AltLeft) || self.is_pressed(KeyboardButton::AltRight)
    }

    pub fn ctrl_pressed(&self) -> bool {
        self.is_pressed(KeyboardButton::ControlLeft)
            || self.is_pressed(KeyboardButton::ControlRight)
    }

    pub fn shift_pressed(&self) -> bool {
        self.is_pressed(KeyboardButton::ShiftLeft) || self.is_pressed(KeyboardButton::ShiftRight)
    }

    pub fn value(&self, button: KeyboardButton) -> f32 {
        if self.button_state.is_down(button as usize) {
            1.0
        } else {
            0.0
        }
    }

    /// Text entered in the current frame. Usually a single symbol.
    pub fn text(&self) -> &str {
        &self.text
    }
}
