use super::{GamepadAxis, GamepadButton, KeyboardButton, MouseControl, SystemEvent, TouchpadAxis};

use internal::static_string;

macro_rules! buttons {
    ($($device_name:ident($as_func:ident): [$($(#[doc = $doc:expr])*($button_item:ident, $device_item:ident),)+])+) => {
        #[luajit_ffi_gen::luajit_ffi]
        #[derive(Debug)]
        pub enum Button {
            $(
                $(
                    $(#[doc = $doc])*
                    $button_item,
                )+
            )+
        }

        impl Button {
            $(
                pub fn $as_func(&self) -> Option<$device_name> {
                    match self {
                        $(Self::$button_item => Some($device_name::$device_item),)+
                        _ => None,
                    }
                }
            )+
        }

        $(
            impl From<$device_name> for Button {
                fn from(value: $device_name) -> Self {
                    match value {
                        $($device_name::$device_item => Self::$button_item,)+
                    }
                }
            }
        )+
    };
}

buttons! {
    KeyboardButton(as_keyboard_button): [
        // Buttons
        /// The `1` key over the letters.
        (KeyboardKey1, Key1),
        /// The `2` key over the letters.
        (KeyboardKey2, Key2),
        /// The `3` key over the letters.
        (KeyboardKey3, Key3),
        /// The `4` key over the letters.
        (KeyboardKey4, Key4),
        /// The `5` key over the letters.
        (KeyboardKey5, Key5),
        /// The `6` key over the letters.
        (KeyboardKey6, Key6),
        /// The `7` key over the letters.
        (KeyboardKey7, Key7),
        /// The `8` key over the letters.
        (KeyboardKey8, Key8),
        /// The `9` key over the letters.
        (KeyboardKey9, Key9),
        /// The `0` key over the letters.
        (KeyboardKey0, Key0),

        /// The `A` key.
        (KeyboardA, A),
        /// The `B` key.
        (KeyboardB, B),
        /// The `C` key.
        (KeyboardC, C),
        /// The `D` key.
        (KeyboardD, D),
        /// The `E` key.
        (KeyboardE, E),
        /// The `F` key.
        (KeyboardF, F),
        /// The `G` key.
        (KeyboardG, G),
        /// The `H` key.
        (KeyboardH, H),
        /// The `I` key.
        (KeyboardI, I),
        /// The `J` key.
        (KeyboardJ, J),
        /// The `K` key.
        (KeyboardK, K),
        /// The `L` key.
        (KeyboardL, L),
        /// The `M` key.
        (KeyboardM, M),
        /// The `N` key.
        (KeyboardN, N),
        /// The `O` key.
        (KeyboardO, O),
        /// The `P` key.
        (KeyboardP, P),
        /// The `Q` key.
        (KeyboardQ, Q),
        /// The `R` key.
        (KeyboardR, R),
        /// The `S` key.
        (KeyboardS, S),
        /// The `T` key.
        (KeyboardT, T),
        /// The `U` key.
        (KeyboardU, U),
        /// The `V` key.
        (KeyboardV, V),
        /// The `W` key.
        (KeyboardW, W),
        /// The `X` key.
        (KeyboardX, X),
        /// The `Y` key.
        (KeyboardY, Y),
        /// The `Z` key.
        (KeyboardZ, Z),

        /// The `Escape` / `ESC` key, next to the `F1` key.
        (KeyboardEscape, Escape),

        /// The `F1` key.
        (KeyboardF1, F1),
        /// The `F2` key.
        (KeyboardF2, F2),
        /// The `F3` key.
        (KeyboardF3, F3),
        /// The `F4` key.
        (KeyboardF4, F4),
        /// The `F5` key.
        (KeyboardF5, F5),
        /// The `F6` key.
        (KeyboardF6, F6),
        /// The `F7` key.
        (KeyboardF7, F7),
        /// The `F8` key.
        (KeyboardF8, F8),
        /// The `F9` key.
        (KeyboardF9, F9),
        /// The `F10` key.
        (KeyboardF10, F10),
        /// The `F11` key.
        (KeyboardF11, F11),
        /// The `F12` key.
        (KeyboardF12, F12),
        /// The `F13` key.
        (KeyboardF13, F13),
        /// The `F14` key.
        (KeyboardF14, F14),
        /// The `F15` key.
        (KeyboardF15, F15),
        /// The `F16` key.
        (KeyboardF16, F16),
        /// The `F17` key.
        (KeyboardF17, F17),
        /// The `F18` key.
        (KeyboardF18, F18),
        /// The `F19` key.
        (KeyboardF19, F19),
        /// The `F20` key.
        (KeyboardF20, F20),
        /// The `F21` key.
        (KeyboardF21, F21),
        /// The `F22` key.
        (KeyboardF22, F22),
        /// The `F23` key.
        (KeyboardF23, F23),
        /// The `F24` key.
        (KeyboardF24, F24),

        /// The `Snapshot` / `Print Screen` key.
        (KeyboardSnapshot, Snapshot),
        /// The `Scroll` / `Scroll Lock` key.
        (KeyboardScroll, Scroll),
        /// The `Pause` / `Break` key, next to the `Scroll` key.
        (KeyboardPause, Pause),

        /// The `Insert` key, next to the `Backspace` key.
        (KeyboardInsert, Insert),
        /// The `Home` key.
        (KeyboardHome, Home),
        /// The `Delete` key.
        (KeyboardDelete, Delete),
        /// The `End` key.
        (KeyboardEnd, End),
        /// The `PageDown` key.
        (KeyboardPageDown, PageDown),
        /// The `PageUp` key.
        (KeyboardPageUp, PageUp),

        /// The `Left` / `Left Arrow` key.
        (KeyboardLeft, Left),
        /// The `Up` / `Up Arrow` key.
        (KeyboardUp, Up),
        /// The `Right` / `Right Arrow` key.
        (KeyboardRight, Right),
        /// The `Down` / `Down Arrow` key.
        (KeyboardDown, Down),

        /// The `Back` / `Backspace` key.
        (KeyboardBack, Back),
        /// The `Return` / `Enter` key.
        (KeyboardReturn, Return),
        /// The `Space` / `Spacebar` / ` ` key.
        (KeyboardSpace, Space),

        /// The `Compose` key on Linux.
        (KeyboardCompose, Compose),
        /// The `Caret` / `^` key.
        (KeyboardCaret, Caret),

        /// The `Numlock` key.
        (KeyboardNumlock, Numlock),
        /// The `Numpad0` / `0` key.
        (KeyboardNumpad0, Numpad0),
        /// The `Numpad1` / `1` key.
        (KeyboardNumpad1, Numpad1),
        /// The `Numpad2` / `2` key.
        (KeyboardNumpad2, Numpad2),
        /// The `Numpad3` / `3` key.
        (KeyboardNumpad3, Numpad3),
        /// The `Numpad4` / `4` key.
        (KeyboardNumpad4, Numpad4),
        /// The `Numpad5` / `5` key.
        (KeyboardNumpad5, Numpad5),
        /// The `Numpad6` / `6` key.
        (KeyboardNumpad6, Numpad6),
        /// The `Numpad7` / `7` key.
        (KeyboardNumpad7, Numpad7),
        /// The `Numpad8` / `8` key.
        (KeyboardNumpad8, Numpad8),
        /// The `Numpad9` / `9` key.
        (KeyboardNumpad9, Numpad9),

        /// The `AbntC1` key.
        (KeyboardAbntC1, AbntC1),
        /// The `AbntC2` key.
        (KeyboardAbntC2, AbntC2),

        /// The `NumpadAdd` / `+` key.
        (KeyboardNumpadAdd, NumpadAdd),
        /// The `Apostrophe` / `'` key.
        (KeyboardApostrophe, Apostrophe),
        /// The `Apps` key.
        (KeyboardApps, Apps),
        /// The `Asterisk` / `*` key.
        (KeyboardAsterisk, Asterisk),
        /// The `Plus` / `+` key.
        (KeyboardPlus, Plus),
        /// The `At` / `@` key.
        (KeyboardAt, At),
        /// The `Ax` key.
        (KeyboardAx, Ax),
        /// The `Backslash` / `\` key.
        (KeyboardBackslash, Backslash),
        /// The `Calculator` key.
        (KeyboardCalculator, Calculator),
        /// The `Capital` key.
        (KeyboardCapital, Capital),
        /// The `Colon` / `:` key.
        (KeyboardColon, Colon),
        /// The `Comma` / `,` key.
        (KeyboardComma, Comma),
        /// The `Convert` key.
        (KeyboardConvert, Convert),
        /// The `NumpadDecimal` / `.` key.
        (KeyboardNumpadDecimal, NumpadDecimal),
        /// The `NumpadDivide` / `/` key.
        (KeyboardNumpadDivide, NumpadDivide),
        /// The `Equals` / `=` key.
        (KeyboardEquals, Equals),
        /// The `Grave` / `Backtick` / `` ` `` key.
        (KeyboardGrave, Grave),
        /// The `Kana` key.
        (KeyboardKana, Kana),
        /// The `Kanji` key.
        (KeyboardKanji, Kanji),

        /// The `Left Alt` key. Maps to `Left Option` on Mac.
        (KeyboardAltLeft, AltLeft),
        /// The `Left Bracket` / `[` key.
        (KeyboardBracketLeft, BracketLeft),
        /// The `Left Control` key.
        (KeyboardControlLeft, ControlLeft),
        /// The `Left Shift` key.
        (KeyboardShiftLeft, ShiftLeft),
        /// The `Left Super` key.
        /// Generic keyboards usually display this key with the *Microsoft Windows* logo.
        /// Apple keyboards call this key the *Command Key* and display it using the ⌘ character.
        (KeyboardSuperLeft,SuperLeft),

        /// The `Mail` key.
        (KeyboardMail, Mail),
        /// The `MediaSelect` key.
        (KeyboardMediaSelect, MediaSelect),
        /// The `MediaStop` key.
        (KeyboardMediaStop, MediaStop),
        /// The `Minus` / `-` key.
        (KeyboardMinus, Minus),
        /// The `NumpadMultiply` / `*` key.
        (KeyboardNumpadMultiply, NumpadMultiply),
        /// The `Mute` key.
        (KeyboardMute, Mute),
        /// The `MyComputer` key.
        (KeyboardMyComputer, MyComputer),
        /// The `NavigateForward` / `Prior` key.
        (KeyboardNavigateForward, NavigateForward),
        /// The `NavigateBackward` / `Next` key.
        (KeyboardNavigateBackward, NavigateBackward),
        /// The `NextTrack` key.
        (KeyboardNextTrack, NextTrack),
        /// The `NoConvert` key.
        (KeyboardNoConvert, NoConvert),
        /// The `NumpadComma` / `,` key.
        (KeyboardNumpadComma, NumpadComma),
        /// The `NumpadEnter` key.
        (KeyboardNumpadEnter, NumpadEnter),
        /// The `NumpadEquals` / `=` key.
        (KeyboardNumpadEquals, NumpadEquals),
        /// The `Oem102` key.
        (KeyboardOem102, Oem102),
        /// The `Period` / `.` key.
        (KeyboardPeriod, Period),
        /// The `PlayPause` key.
        (KeyboardPlayPause, PlayPause),
        /// The `Power` key.
        (KeyboardPower, Power),
        /// The `PrevTrack` key.
        (KeyboardPrevTrack, PrevTrack),

        /// The `Right Alt` key. Maps to `Right Option` on Mac.
        (KeyboardAltRight, AltRight),
        /// The `Right Bracket` / `]` key.
        (KeyboardBracketRight, BracketRight),
        /// The `Right Control` key.
        (KeyboardControlRight, ControlRight),
        /// The `Right Shift` key.
        (KeyboardShiftRight, ShiftRight),
        /// The `Right Super` key.
        /// Generic keyboards usually display this key with the *Microsoft Windows* logo.
        /// Apple keyboards call this key the *Command Key* and display it using the ⌘ character.
        (KeyboardSuperRight,SuperRight),

        /// The `Semicolon` / `;` key.
        (KeyboardSemicolon, Semicolon),
        /// The `Slash` / `/` key.
        (KeyboardSlash, Slash),
        /// The `Sleep` key.
        (KeyboardSleep, Sleep),
        /// The `Stop` key.
        (KeyboardStop, Stop),
        /// The `NumpadSubtract` / `-` key.
        (KeyboardNumpadSubtract, NumpadSubtract),
        /// The `Sysrq` key.
        (KeyboardSysrq, Sysrq),
        /// The `Tab` / `   ` key.
        (KeyboardTab, Tab),
        /// The `Underline` / `_` key.
        (KeyboardUnderline, Underline),
        /// The `Unlabeled` key.
        (KeyboardUnlabeled, Unlabeled),

        /// The `VolumeDown` key.
        (KeyboardVolumeDown, VolumeDown),
        /// The `VolumeUp` key.
        (KeyboardVolumeUp, VolumeUp),

        /// The `Wake` key.
        (KeyboardWake, Wake),

        /// The `WebBack` key.
        (KeyboardWebBack, WebBack),
        /// The `WebFavorites` key.
        (KeyboardWebFavorites, WebFavorites),
        /// The `WebForward` key.
        (KeyboardWebForward, WebForward),
        /// The `WebHome` key.
        (KeyboardWebHome, WebHome),
        /// The `WebRefresh` key.
        (KeyboardWebRefresh, WebRefresh),
        /// The `WebSearch` key.
        (KeyboardWebSearch, WebSearch),
        /// The `WebStop` key.
        (KeyboardWebStop, WebStop),

        /// The `Yen` key.
        (KeyboardYen, Yen),

        /// The `Copy` key.
        (KeyboardCopy, Copy),
        /// The `Paste` key.
        (KeyboardPaste, Paste),
        /// The `Cut` key.
        (KeyboardCut, Cut),
    ]
    MouseControl(as_mouse_control): [
        // Buttons
        (MouseLeft, Left),
        (MouseMiddle, Middle),
        (MouseRight, Right),
        (MouseX1, X1),
        (MouseX2, X2),
        // Axes
        (MouseDeltaX, DeltaX),
        (MouseDeltaY, DeltaY),
        (MouseScrollX, ScrollX),
        (MouseScrollY, ScrollY),
        (MouseScrollLineX, ScrollLineX),
        (MouseScrollLineY, ScrollLineY),
    ]
    GamepadButton(as_gamepad_button): [
        // Buttons
        /// The bottom action button of the action pad (i.e. PS: Cross, Xbox: A).
        (GamepadSouth, South),
        /// The right action button of the action pad (i.e. PS: Circle, Xbox: B).
        (GamepadEast, East),
        /// The upper action button of the action pad (i.e. PS: Triangle, Xbox: Y).
        (GamepadNorth, North),
        /// The left action button of the action pad (i.e. PS: Square, Xbox: X).
        (GamepadWest, West),

        /// The C button.
        (GamepadC, C),
        /// The Z button.
        (GamepadZ, Z),

        /// The first left trigger.
        (GamepadLeftTrigger, LeftTrigger),
        /// The second left trigger.
        (GamepadLeftTrigger2, LeftTrigger2),
        /// The first right trigger.
        (GamepadRightTrigger, RightTrigger),
        /// The second right trigger.
        (GamepadRightTrigger2, RightTrigger2),
        /// The select button.
        (GamepadSelect, Select),
        /// The start button.
        (GamepadStart, Start),
        /// The mode button.
        (GamepadMode, Mode),

        /// The left thumb stick button.
        (GamepadLeftThumb, LeftThumb),
        /// The right thumb stick button.
        (GamepadRightThumb, RightThumb),

        /// The up button of the D-Pad.
        (GamepadDPadUp, DPadUp),
        /// The down button of the D-Pad.
        (GamepadDPadDown, DPadDown),
        /// The left button of the D-Pad.
        (GamepadDPadLeft, DPadLeft),
        /// The right button of the D-Pad.
        (GamepadDPadRight, DPadRight),
    ]
    GamepadAxis(as_gamepad_axis): [
        // Axes
        /// The horizontal value of the left stick.
        (GamepadLeftStickX, LeftStickX),
        /// The vertical value of the left stick.
        (GamepadLeftStickY, LeftStickY),
        /// The value of the left `Z` button.
        (GamepadLeftZ, LeftZ),

        /// The horizontal value of the right stick.
        (GamepadRightStickX, RightStickX),
        /// The vertical value of the right stick.
        (GamepadRightStickY, RightStickY),
        /// The value of the right `Z` button.
        (GamepadRightZ, RightZ),
    ]
    TouchpadAxis(as_touchpad_axis): [
        // Axes
        (TouchpadX, X),
        (TouchpadY, Y),
        (TouchpadMagnifyDelta, MagnifyDelta),
        (TouchpadRotateDelta, RotateDelta),
    ]
    SystemEvent(as_system_event): [
        (SystemExit, Exit),
    ]
}
