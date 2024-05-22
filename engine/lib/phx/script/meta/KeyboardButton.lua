---@meta

---@enum KeyboardButton
KeyboardButton = {
    -- <kbd>`</kbd> on a US keyboard. This is also called a backtick or grave.
    -- This is the <kbd>半角</kbd>/<kbd>全角</kbd>/<kbd>漢字</kbd>
    -- (hankaku/zenkaku/kanji) key on Japanese keyboards
    Backquote = 0,
    -- Used for both the US <kbd>\\</kbd> (on the 101-key layout) and also for the key
    -- located between the <kbd>"</kbd> and <kbd>Enter</kbd> keys on row C of the 102-,
    -- 104- and 106-key layouts.
    -- Labeled <kbd>#</kbd> on a UK (102) keyboard.
    Backslash = 1,
    -- <kbd>[</kbd> on a US keyboard.
    BracketLeft = 2,
    -- <kbd>]</kbd> on a US keyboard.
    BracketRight = 3,
    -- <kbd>,</kbd> on a US keyboard.
    Comma = 4,
    -- <kbd>0</kbd> on a US keyboard.
    Key0 = 5,
    -- <kbd>1</kbd> on a US keyboard.
    Key1 = 6,
    -- <kbd>2</kbd> on a US keyboard.
    Key2 = 7,
    -- <kbd>3</kbd> on a US keyboard.
    Key3 = 8,
    -- <kbd>4</kbd> on a US keyboard.
    Key4 = 9,
    -- <kbd>5</kbd> on a US keyboard.
    Key5 = 10,
    -- <kbd>6</kbd> on a US keyboard.
    Key6 = 11,
    -- <kbd>7</kbd> on a US keyboard.
    Key7 = 12,
    -- <kbd>8</kbd> on a US keyboard.
    Key8 = 13,
    -- <kbd>9</kbd> on a US keyboard.
    Key9 = 14,
    -- <kbd>=</kbd> on a US keyboard.
    Equal = 15,
    -- Located between the left <kbd>Shift</kbd> and <kbd>Z</kbd> keys.
    -- Labeled <kbd>\\</kbd> on a UK keyboard.
    IntlBackslash = 16,
    -- Located between the <kbd>/</kbd> and right <kbd>Shift</kbd> keys.
    -- Labeled <kbd>\\</kbd> (ro) on a Japanese keyboard.
    IntlRo = 17,
    -- Located between the <kbd>=</kbd> and <kbd>Backspace</kbd> keys.
    -- Labeled <kbd>¥</kbd> (yen) on a Japanese keyboard. <kbd>\\</kbd> on a
    -- Russian keyboard.
    IntlYen = 18,
    -- <kbd>a</kbd> on a US keyboard.
    -- Labeled <kbd>q</kbd> on an AZERTY (e.g., French) keyboard.
    A = 19,
    -- <kbd>b</kbd> on a US keyboard.
    B = 20,
    -- <kbd>c</kbd> on a US keyboard.
    C = 21,
    -- <kbd>d</kbd> on a US keyboard.
    D = 22,
    -- <kbd>e</kbd> on a US keyboard.
    E = 23,
    -- <kbd>f</kbd> on a US keyboard.
    F = 24,
    -- <kbd>g</kbd> on a US keyboard.
    G = 25,
    -- <kbd>h</kbd> on a US keyboard.
    H = 26,
    -- <kbd>i</kbd> on a US keyboard.
    I = 27,
    -- <kbd>j</kbd> on a US keyboard.
    J = 28,
    -- <kbd>k</kbd> on a US keyboard.
    K = 29,
    -- <kbd>l</kbd> on a US keyboard.
    L = 30,
    -- <kbd>m</kbd> on a US keyboard.
    M = 31,
    -- <kbd>n</kbd> on a US keyboard.
    N = 32,
    -- <kbd>o</kbd> on a US keyboard.
    O = 33,
    -- <kbd>p</kbd> on a US keyboard.
    P = 34,
    -- <kbd>q</kbd> on a US keyboard.
    -- Labeled <kbd>a</kbd> on an AZERTY (e.g., French) keyboard.
    Q = 35,
    -- <kbd>r</kbd> on a US keyboard.
    R = 36,
    -- <kbd>s</kbd> on a US keyboard.
    S = 37,
    -- <kbd>t</kbd> on a US keyboard.
    T = 38,
    -- <kbd>u</kbd> on a US keyboard.
    U = 39,
    -- <kbd>v</kbd> on a US keyboard.
    V = 40,
    -- <kbd>w</kbd> on a US keyboard.
    -- Labeled <kbd>z</kbd> on an AZERTY (e.g., French) keyboard.
    W = 41,
    -- <kbd>x</kbd> on a US keyboard.
    X = 42,
    -- <kbd>y</kbd> on a US keyboard.
    -- Labeled <kbd>z</kbd> on a QWERTZ (e.g., German) keyboard.
    Y = 43,
    -- <kbd>z</kbd> on a US keyboard.
    -- Labeled <kbd>w</kbd> on an AZERTY (e.g., French) keyboard, and <kbd>y</kbd> on a
    -- QWERTZ (e.g., German) keyboard.
    Z = 44,
    -- <kbd>-</kbd> on a US keyboard.
    Minus = 45,
    -- <kbd>.</kbd> on a US keyboard.
    Period = 46,
    -- <kbd>'</kbd> on a US keyboard.
    Quote = 47,
    -- <kbd>;</kbd> on a US keyboard.
    Semicolon = 48,
    -- <kbd>/</kbd> on a US keyboard.
    Slash = 49,
    -- <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
    AltLeft = 50,
    -- <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
    -- This is labeled <kbd>AltGr</kbd> on many keyboard layouts.
    AltRight = 51,
    -- <kbd>Backspace</kbd> or <kbd>⌫</kbd>.
    -- Labeled <kbd>Delete</kbd> on Apple keyboards.
    Backspace = 52,
    -- <kbd>CapsLock</kbd> or <kbd>⇪</kbd>
    CapsLock = 53,
    -- The application context menu key, which is typically found between the right
    -- <kbd>Super</kbd> key and the right <kbd>Control</kbd> key.
    ContextMenu = 54,
    -- <kbd>Control</kbd> or <kbd>⌃</kbd>
    ControlLeft = 55,
    -- <kbd>Control</kbd> or <kbd>⌃</kbd>
    ControlRight = 56,
    -- <kbd>Enter</kbd> or <kbd>↵</kbd>. Labeled <kbd>Return</kbd> on Apple keyboards.
    Enter = 57,
    -- The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
    SuperLeft = 58,
    -- The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
    SuperRight = 59,
    -- <kbd>Shift</kbd> or <kbd>⇧</kbd>
    ShiftLeft = 60,
    -- <kbd>Shift</kbd> or <kbd>⇧</kbd>
    ShiftRight = 61,
    -- <kbd> </kbd> (space)
    Space = 62,
    -- <kbd>Tab</kbd> or <kbd>⇥</kbd>
    Tab = 63,
    -- Japanese: <kbd>変</kbd> (henkan)
    Convert = 64,
    -- Japanese: <kbd>カタカナ</kbd>/<kbd>ひらがな</kbd>/<kbd>ローマ字</kbd> (katakana/hiragana/romaji)
    KanaMode = 65,
    -- Korean: HangulMode <kbd>한/영</kbd> (han/yeong)
    -- 
    -- Japanese (Mac keyboard): <kbd>か</kbd> (kana)
    Lang1 = 66,
    -- Korean: Hanja <kbd>한</kbd> (hanja)
    -- 
    -- Japanese (Mac keyboard): <kbd>英</kbd> (eisu)
    Lang2 = 67,
    -- Japanese (word-processing keyboard): Katakana
    Lang3 = 68,
    -- Japanese (word-processing keyboard): Hiragana
    Lang4 = 69,
    -- Japanese (word-processing keyboard): Zenkaku/Hankaku
    Lang5 = 70,
    -- Japanese: <kbd>無変換</kbd> (muhenkan)
    NonConvert = 71,
    -- <kbd>⌦</kbd>. The forward delete key.
    -- Note that on Apple keyboards, the key labelled <kbd>Delete</kbd> on the main part of
    -- the keyboard is encoded as [`Backspace`].
    -- 
    -- [`Backspace`]: Self::Backspace
    Delete = 72,
    -- <kbd>Page Down</kbd>, <kbd>End</kbd>, or <kbd>↘</kbd>
    End = 73,
    -- <kbd>Help</kbd>. Not present on standard PC keyboards.
    Help = 74,
    -- <kbd>Home</kbd> or <kbd>↖</kbd>
    Home = 75,
    -- <kbd>Insert</kbd> or <kbd>Ins</kbd>. Not present on Apple keyboards.
    Insert = 76,
    -- <kbd>Page Down</kbd>, <kbd>PgDn</kbd>, or <kbd>⇟</kbd>
    PageDown = 77,
    -- <kbd>Page Up</kbd>, <kbd>PgUp</kbd>, or <kbd>⇞</kbd>
    PageUp = 78,
    -- <kbd>↓</kbd>
    Down = 79,
    -- <kbd>←</kbd>
    Left = 80,
    -- <kbd>→</kbd>
    Right = 81,
    -- <kbd>↑</kbd>
    Up = 82,
    -- On the Mac, this is used for the numpad <kbd>Clear</kbd> key.
    NumLock = 83,
    -- <kbd>0 Ins</kbd> on a keyboard. <kbd>0</kbd> on a phone or remote control
    Numpad0 = 84,
    -- <kbd>1 End</kbd> on a keyboard. <kbd>1</kbd> or <kbd>1 QZ</kbd> on a phone or remote control
    Numpad1 = 85,
    -- <kbd>2 ↓</kbd> on a keyboard. <kbd>2 ABC</kbd> on a phone or remote control
    Numpad2 = 86,
    -- <kbd>3 PgDn</kbd> on a keyboard. <kbd>3 DEF</kbd> on a phone or remote control
    Numpad3 = 87,
    -- <kbd>4 ←</kbd> on a keyboard. <kbd>4 GHI</kbd> on a phone or remote control
    Numpad4 = 88,
    -- <kbd>5</kbd> on a keyboard. <kbd>5 JKL</kbd> on a phone or remote control
    Numpad5 = 89,
    -- <kbd>6 →</kbd> on a keyboard. <kbd>6 MNO</kbd> on a phone or remote control
    Numpad6 = 90,
    -- <kbd>7 Home</kbd> on a keyboard. <kbd>7 PQRS</kbd> or <kbd>7 PRS</kbd> on a phone
    -- or remote control
    Numpad7 = 91,
    -- <kbd>8 ↑</kbd> on a keyboard. <kbd>8 TUV</kbd> on a phone or remote control
    Numpad8 = 92,
    -- <kbd>9 PgUp</kbd> on a keyboard. <kbd>9 WXYZ</kbd> or <kbd>9 WXY</kbd> on a phone
    -- or remote control
    Numpad9 = 93,
    -- <kbd>+</kbd>
    NumpadAdd = 94,
    -- Found on the Microsoft Natural Keyboard.
    NumpadBackspace = 95,
    -- <kbd>C</kbd> or <kbd>A</kbd> (All Clear). Also for use with numpads that have a
    -- <kbd>Clear</kbd> key that is separate from the <kbd>NumLock</kbd> key. On the Mac, the
    -- numpad <kbd>Clear</kbd> key is encoded as [`NumLock`].
    -- 
    -- [`NumLock`]: Self::NumLock
    NumpadClear = 96,
    -- <kbd>C</kbd> (Clear Entry)
    NumpadClearEntry = 97,
    -- <kbd>,</kbd> (thousands separator). For locales where the thousands separator
    -- is a "." (e.g., Brazil), this key may generate a <kbd>.</kbd>.
    NumpadComma = 98,
    -- <kbd>. Del</kbd>. For locales where the decimal separator is "," (e.g.,
    -- Brazil), this key may generate a <kbd>,</kbd>.
    NumpadDecimal = 99,
    -- <kbd>/</kbd>
    NumpadDivide = 100,
    NumpadEnter = 101,
    -- <kbd>=</kbd>
    NumpadEqual = 102,
    -- <kbd>#</kbd> on a phone or remote control device. This key is typically found
    -- below the <kbd>9</kbd> key and to the right of the <kbd>0</kbd> key.
    NumpadHash = 103,
    -- <kbd>M</kbd> Add current entry to the value stored in memory.
    NumpadMemoryAdd = 104,
    -- <kbd>M</kbd> Clear the value stored in memory.
    NumpadMemoryClear = 105,
    -- <kbd>M</kbd> Replace the current entry with the value stored in memory.
    NumpadMemoryRecall = 106,
    -- <kbd>M</kbd> Replace the value stored in memory with the current entry.
    NumpadMemoryStore = 107,
    -- <kbd>M</kbd> Subtract current entry from the value stored in memory.
    NumpadMemorySubtract = 108,
    -- <kbd>*</kbd> on a keyboard. For use with numpads that provide mathematical
    -- operations (<kbd>+</kbd>, <kbd>-</kbd> <kbd>*</kbd> and <kbd>/</kbd>).
    -- 
    -- Use `NumpadStar` for the <kbd>*</kbd> key on phones and remote controls.
    NumpadMultiply = 109,
    -- <kbd>(</kbd> Found on the Microsoft Natural Keyboard.
    NumpadParenLeft = 110,
    -- <kbd>)</kbd> Found on the Microsoft Natural Keyboard.
    NumpadParenRight = 111,
    -- <kbd>*</kbd> on a phone or remote control device.
    -- 
    -- This key is typically found below the <kbd>7</kbd> key and to the left of
    -- the <kbd>0</kbd> key.
    -- 
    -- Use <kbd>"NumpadMultiply"</kbd> for the <kbd>*</kbd> key on
    -- numeric keypads.
    NumpadStar = 112,
    -- <kbd>-</kbd>
    NumpadSubtract = 113,
    -- <kbd>Esc</kbd> or <kbd>⎋</kbd>
    Escape = 114,
    -- <kbd>Fn</kbd> This is typically a hardware key that does not generate a separate code.
    Fn = 115,
    -- <kbd>FLock</kbd> or <kbd>FnLock</kbd>. Function Lock key. Found on the Microsoft
    -- Natural Keyboard.
    FnLock = 116,
    -- <kbd>PrtScr SysRq</kbd> or <kbd>Print Screen</kbd>
    PrintScreen = 117,
    -- <kbd>Scroll Lock</kbd>
    ScrollLock = 118,
    -- <kbd>Pause Break</kbd>
    Pause = 119,
    -- Some laptops place this key to the left of the <kbd>↑</kbd> key.
    -- 
    -- This also the "back" button (triangle) on Android.
    BrowserBack = 120,
    BrowserFavorites = 121,
    -- Some laptops place this key to the right of the <kbd>↑</kbd> key.
    BrowserForward = 122,
    -- The "home" button on Android.
    BrowserHome = 123,
    BrowserRefresh = 124,
    BrowserSearch = 125,
    BrowserStop = 126,
    -- <kbd>Eject</kbd> or <kbd>⏏</kbd>. This key is placed in the function section on some Apple
    -- keyboards.
    Eject = 127,
    -- Sometimes labelled <kbd>My Computer</kbd> on the keyboard
    LaunchApp1 = 128,
    -- Sometimes labelled <kbd>Calculator</kbd> on the keyboard
    LaunchApp2 = 129,
    LaunchMail = 130,
    MediaPlayPause = 131,
    MediaSelect = 132,
    MediaStop = 133,
    MediaTrackNext = 134,
    MediaTrackPrevious = 135,
    -- This key is placed in the function section on some Apple keyboards, replacing the
    -- <kbd>Eject</kbd> key.
    Power = 136,
    Sleep = 137,
    AudioVolumeDown = 138,
    AudioVolumeMute = 139,
    AudioVolumeUp = 140,
    WakeUp = 141,
    Meta = 142,
    Hyper = 143,
    Turbo = 144,
    Abort = 145,
    Resume = 146,
    Suspend = 147,
    -- Found on Sun’s USB keyboard.
    Again = 148,
    -- Found on Sun’s USB keyboard.
    Copy = 149,
    -- Found on Sun’s USB keyboard.
    Cut = 150,
    -- Found on Sun’s USB keyboard.
    Find = 151,
    -- Found on Sun’s USB keyboard.
    Open = 152,
    -- Found on Sun’s USB keyboard.
    Paste = 153,
    -- Found on Sun’s USB keyboard.
    Props = 154,
    -- Found on Sun’s USB keyboard.
    Select = 155,
    -- Found on Sun’s USB keyboard.
    Undo = 156,
    -- Use for dedicated <kbd>ひらがな</kbd> key found on some Japanese word processing keyboards.
    Hiragana = 157,
    -- Use for dedicated <kbd>カタカナ</kbd> key found on some Japanese word processing keyboards.
    Katakana = 158,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F1 = 159,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F2 = 160,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F3 = 161,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F4 = 162,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F5 = 163,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F6 = 164,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F7 = 165,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F8 = 166,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F9 = 167,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F10 = 168,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F11 = 169,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F12 = 170,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F13 = 171,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F14 = 172,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F15 = 173,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F16 = 174,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F17 = 175,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F18 = 176,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F19 = 177,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F20 = 178,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F21 = 179,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F22 = 180,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F23 = 181,
    -- General-purpose function key.
    -- Usually found at the top of the keyboard.
    F24 = 182,
    -- General-purpose function key.
    F25 = 183,
    -- General-purpose function key.
    F26 = 184,
    -- General-purpose function key.
    F27 = 185,
    -- General-purpose function key.
    F28 = 186,
    -- General-purpose function key.
    F29 = 187,
    -- General-purpose function key.
    F30 = 188,
    -- General-purpose function key.
    F31 = 189,
    -- General-purpose function key.
    F32 = 190,
    -- General-purpose function key.
    F33 = 191,
    -- General-purpose function key.
    F34 = 192,
    -- General-purpose function key.
    F35 = 193,
}

