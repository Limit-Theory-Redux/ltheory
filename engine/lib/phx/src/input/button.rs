use super::{GamepadAxis, GamepadButton, KeyboardButton, MouseControl, SystemEvent, TouchpadAxis};

macro_rules! buttons {
    ($($device_name:ident($as_func:ident): [$($(#[doc = $doc:expr])*($button_item:ident, $device_item:ident),)+])+) => {
        #[luajit_ffi_gen::luajit_ffi]
        #[derive(Debug, Clone, Copy)]
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
        (KeyboardBackquote, Backquote),
        (KeyboardBackslash, Backslash),
        (KeyboardBracketLeft, BracketLeft),
        (KeyboardBracketRight, BracketRight),
        (KeyboardComma, Comma),
        (KeyboardKey0, Key0),
        (KeyboardKey1, Key1),
        (KeyboardKey2, Key2),
        (KeyboardKey3, Key3),
        (KeyboardKey4, Key4),
        (KeyboardKey5, Key5),
        (KeyboardKey6, Key6),
        (KeyboardKey7, Key7),
        (KeyboardKey8, Key8),
        (KeyboardKey9, Key9),
        (KeyboardEqual, Equal),
        (KeyboardIntlBackslash, IntlBackslash),
        (KeyboardIntlRo, IntlRo),
        (KeyboardIntlYen, IntlYen),
        (KeyboardA, A),
        (KeyboardB, B),
        (KeyboardC, C),
        (KeyboardD, D),
        (KeyboardE, E),
        (KeyboardF, F),
        (KeyboardG, G),
        (KeyboardH, H),
        (KeyboardI, I),
        (KeyboardJ, J),
        (KeyboardK, K),
        (KeyboardL, L),
        (KeyboardM, M),
        (KeyboardN, N),
        (KeyboardO, O),
        (KeyboardP, P),
        (KeyboardQ, Q),
        (KeyboardR, R),
        (KeyboardS, S),
        (KeyboardT, T),
        (KeyboardU, U),
        (KeyboardV, V),
        (KeyboardW, W),
        (KeyboardX, X),
        (KeyboardY, Y),
        (KeyboardZ, Z),
        (KeyboardMinus, Minus),
        (KeyboardPeriod, Period),
        (KeyboardQuote, Quote),
        (KeyboardSemicolon, Semicolon),
        (KeyboardSlash, Slash),
        (KeyboardAltLeft, AltLeft),
        (KeyboardAltRight, AltRight),
        (KeyboardBackspace, Backspace),
        (KeyboardCapsLock, CapsLock),
        (KeyboardContextMenu, ContextMenu),
        (KeyboardControlLeft, ControlLeft),
        (KeyboardControlRight, ControlRight),
        (KeyboardEnter, Enter),
        (KeyboardSuperLeft, SuperLeft),
        (KeyboardSuperRight, SuperRight),
        (KeyboardShiftLeft, ShiftLeft),
        (KeyboardShiftRight, ShiftRight),
        (KeyboardSpace, Space),
        (KeyboardTab, Tab),
        (KeyboardConvert, Convert),
        (KeyboardKanaMode, KanaMode),
        (KeyboardLang1, Lang1),
        (KeyboardLang2, Lang2),
        (KeyboardLang3, Lang3),
        (KeyboardLang4, Lang4),
        (KeyboardLang5, Lang5),
        (KeyboardNonConvert, NonConvert),
        (KeyboardDelete, Delete),
        (KeyboardEnd, End),
        (KeyboardHelp, Help),
        (KeyboardHome, Home),
        (KeyboardInsert, Insert),
        (KeyboardPageDown, PageDown),
        (KeyboardPageUp, PageUp),
        (KeyboardDown, Down),
        (KeyboardLeft, Left),
        (KeyboardRight, Right),
        (KeyboardUp, Up),
        (KeyboardNumLock, NumLock),
        (KeyboardNumpad0, Numpad0),
        (KeyboardNumpad1, Numpad1),
        (KeyboardNumpad2, Numpad2),
        (KeyboardNumpad3, Numpad3),
        (KeyboardNumpad4, Numpad4),
        (KeyboardNumpad5, Numpad5),
        (KeyboardNumpad6, Numpad6),
        (KeyboardNumpad7, Numpad7),
        (KeyboardNumpad8, Numpad8),
        (KeyboardNumpad9, Numpad9),
        (KeyboardNumpadAdd, NumpadAdd),
        (KeyboardNumpadBackspace, NumpadBackspace),
        (KeyboardNumpadClear, NumpadClear),
        (KeyboardNumpadClearEntry, NumpadClearEntry),
        (KeyboardNumpadComma, NumpadComma),
        (KeyboardNumpadDecimal, NumpadDecimal),
        (KeyboardNumpadDivide, NumpadDivide),
        (KeyboardNumpadEnter, NumpadEnter),
        (KeyboardNumpadEqual, NumpadEqual),
        (KeyboardNumpadHash, NumpadHash),
        (KeyboardNumpadMemoryAdd, NumpadMemoryAdd),
        (KeyboardNumpadMemoryClear, NumpadMemoryClear),
        (KeyboardNumpadMemoryRecall, NumpadMemoryRecall),
        (KeyboardNumpadMemoryStore, NumpadMemoryStore),
        (KeyboardNumpadMemorySubtract, NumpadMemorySubtract),
        (KeyboardNumpadMultiply, NumpadMultiply),
        (KeyboardNumpadParenLeft, NumpadParenLeft),
        (KeyboardNumpadParenRight, NumpadParenRight),
        (KeyboardNumpadStar, NumpadStar),
        (KeyboardNumpadSubtract, NumpadSubtract),
        (KeyboardEscape, Escape),
        (KeyboardFn, Fn),
        (KeyboardFnLock, FnLock),
        (KeyboardPrintScreen, PrintScreen),
        (KeyboardScrollLock, ScrollLock),
        (KeyboardPause, Pause),
        (KeyboardBrowserBack, BrowserBack),
        (KeyboardBrowserFavorites, BrowserFavorites),
        (KeyboardBrowserForward, BrowserForward),
        (KeyboardBrowserHome, BrowserHome),
        (KeyboardBrowserRefresh, BrowserRefresh),
        (KeyboardBrowserSearch, BrowserSearch),
        (KeyboardBrowserStop, BrowserStop),
        (KeyboardEject, Eject),
        (KeyboardLaunchApp1, LaunchApp1),
        (KeyboardLaunchApp2, LaunchApp2),
        (KeyboardLaunchMail, LaunchMail),
        (KeyboardMediaPlayPause, MediaPlayPause),
        (KeyboardMediaSelect, MediaSelect),
        (KeyboardMediaStop, MediaStop),
        (KeyboardMediaTrackNext, MediaTrackNext),
        (KeyboardMediaTrackPrevious, MediaTrackPrevious),
        (KeyboardPower, Power),
        (KeyboardSleep, Sleep),
        (KeyboardAudioVolumeDown, AudioVolumeDown),
        (KeyboardAudioVolumeMute, AudioVolumeMute),
        (KeyboardAudioVolumeUp, AudioVolumeUp),
        (KeyboardWakeUp, WakeUp),
        (KeyboardMeta, Meta),
        (KeyboardHyper, Hyper),
        (KeyboardTurbo, Turbo),
        (KeyboardAbort, Abort),
        (KeyboardResume, Resume),
        (KeyboardSuspend, Suspend),
        (KeyboardAgain, Again),
        (KeyboardCopy, Copy),
        (KeyboardCut, Cut),
        (KeyboardFind, Find),
        (KeyboardOpen, Open),
        (KeyboardPaste, Paste),
        (KeyboardProps, Props),
        (KeyboardSelect, Select),
        (KeyboardUndo, Undo),
        (KeyboardHiragana, Hiragana),
        (KeyboardKatakana, Katakana),
        (KeyboardF1, F1),
        (KeyboardF2, F2),
        (KeyboardF3, F3),
        (KeyboardF4, F4),
        (KeyboardF5, F5),
        (KeyboardF6, F6),
        (KeyboardF7, F7),
        (KeyboardF8, F8),
        (KeyboardF9, F9),
        (KeyboardF10, F10),
        (KeyboardF11, F11),
        (KeyboardF12, F12),
        (KeyboardF13, F13),
        (KeyboardF14, F14),
        (KeyboardF15, F15),
        (KeyboardF16, F16),
        (KeyboardF17, F17),
        (KeyboardF18, F18),
        (KeyboardF19, F19),
        (KeyboardF20, F20),
        (KeyboardF21, F21),
        (KeyboardF22, F22),
        (KeyboardF23, F23),
        (KeyboardF24, F24),
        (KeyboardF25, F25),
        (KeyboardF26, F26),
        (KeyboardF27, F27),
        (KeyboardF28, F28),
        (KeyboardF29, F29),
        (KeyboardF30, F30),
        (KeyboardF31, F31),
        (KeyboardF32, F32),
        (KeyboardF33, F33),
        (KeyboardF34, F34),
        (KeyboardF35, F35),
    ]
    MouseControl(as_mouse_control): [
        // Buttons
        (MouseLeft, Left),
        (MouseMiddle, Middle),
        (MouseRight, Right),
        (MouseForward, Forward),
        (MouseBack, Back),
        (MouseX1, X1),
        (MouseX2, X2),
        // Axes
        (MouseDeltaX, DeltaX),
        (MouseDeltaY, DeltaY),
        (MouseScrollX, ScrollX),
        (MouseScrollY, ScrollY),
        (MouseScrollPixelX, ScrollPixelX),
        (MouseScrollPixelY, ScrollPixelY),
    ]
    GamepadButton(as_gamepad_button): [
        // Buttons
        (GamepadSouth, South),
        (GamepadEast, East),
        (GamepadNorth, North),
        (GamepadWest, West),

        (GamepadC, C),
        (GamepadZ, Z),

        (GamepadLeftTrigger, LeftTrigger),
        (GamepadLeftTrigger2, LeftTrigger2),
        (GamepadRightTrigger, RightTrigger),
        (GamepadRightTrigger2, RightTrigger2),
        (GamepadSelect, Select),
        (GamepadStart, Start),
        (GamepadMode, Mode),

        (GamepadLeftThumb, LeftThumb),
        (GamepadRightThumb, RightThumb),

        (GamepadDPadUp, DPadUp),
        (GamepadDPadDown, DPadDown),
        (GamepadDPadLeft, DPadLeft),
        (GamepadDPadRight, DPadRight),
    ]
    GamepadAxis(as_gamepad_axis): [
        // Axes
        (GamepadLeftStickX, LeftStickX),
        (GamepadLeftStickY, LeftStickY),
        (GamepadLeftZ, LeftZ),

        (GamepadRightStickX, RightStickX),
        (GamepadRightStickY, RightStickY),
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
