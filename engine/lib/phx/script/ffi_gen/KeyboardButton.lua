-- KeyboardButton --------------------------------------------------------------

local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 KeyboardButton;
    ]]

    return 2, 'KeyboardButton'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local KeyboardButton

    do -- C Definitions
        ffi.cdef [[
            KeyboardButton KeyboardButton_Backquote;
            KeyboardButton KeyboardButton_Backslash;
            KeyboardButton KeyboardButton_BracketLeft;
            KeyboardButton KeyboardButton_BracketRight;
            KeyboardButton KeyboardButton_Comma;
            KeyboardButton KeyboardButton_Key0;
            KeyboardButton KeyboardButton_Key1;
            KeyboardButton KeyboardButton_Key2;
            KeyboardButton KeyboardButton_Key3;
            KeyboardButton KeyboardButton_Key4;
            KeyboardButton KeyboardButton_Key5;
            KeyboardButton KeyboardButton_Key6;
            KeyboardButton KeyboardButton_Key7;
            KeyboardButton KeyboardButton_Key8;
            KeyboardButton KeyboardButton_Key9;
            KeyboardButton KeyboardButton_Equal;
            KeyboardButton KeyboardButton_IntlBackslash;
            KeyboardButton KeyboardButton_IntlRo;
            KeyboardButton KeyboardButton_IntlYen;
            KeyboardButton KeyboardButton_A;
            KeyboardButton KeyboardButton_B;
            KeyboardButton KeyboardButton_C;
            KeyboardButton KeyboardButton_D;
            KeyboardButton KeyboardButton_E;
            KeyboardButton KeyboardButton_F;
            KeyboardButton KeyboardButton_G;
            KeyboardButton KeyboardButton_H;
            KeyboardButton KeyboardButton_I;
            KeyboardButton KeyboardButton_J;
            KeyboardButton KeyboardButton_K;
            KeyboardButton KeyboardButton_L;
            KeyboardButton KeyboardButton_M;
            KeyboardButton KeyboardButton_N;
            KeyboardButton KeyboardButton_O;
            KeyboardButton KeyboardButton_P;
            KeyboardButton KeyboardButton_Q;
            KeyboardButton KeyboardButton_R;
            KeyboardButton KeyboardButton_S;
            KeyboardButton KeyboardButton_T;
            KeyboardButton KeyboardButton_U;
            KeyboardButton KeyboardButton_V;
            KeyboardButton KeyboardButton_W;
            KeyboardButton KeyboardButton_X;
            KeyboardButton KeyboardButton_Y;
            KeyboardButton KeyboardButton_Z;
            KeyboardButton KeyboardButton_Minus;
            KeyboardButton KeyboardButton_Period;
            KeyboardButton KeyboardButton_Quote;
            KeyboardButton KeyboardButton_Semicolon;
            KeyboardButton KeyboardButton_Slash;
            KeyboardButton KeyboardButton_AltLeft;
            KeyboardButton KeyboardButton_AltRight;
            KeyboardButton KeyboardButton_Backspace;
            KeyboardButton KeyboardButton_CapsLock;
            KeyboardButton KeyboardButton_ContextMenu;
            KeyboardButton KeyboardButton_ControlLeft;
            KeyboardButton KeyboardButton_ControlRight;
            KeyboardButton KeyboardButton_Enter;
            KeyboardButton KeyboardButton_SuperLeft;
            KeyboardButton KeyboardButton_SuperRight;
            KeyboardButton KeyboardButton_ShiftLeft;
            KeyboardButton KeyboardButton_ShiftRight;
            KeyboardButton KeyboardButton_Space;
            KeyboardButton KeyboardButton_Tab;
            KeyboardButton KeyboardButton_Convert;
            KeyboardButton KeyboardButton_KanaMode;
            KeyboardButton KeyboardButton_Lang1;
            KeyboardButton KeyboardButton_Lang2;
            KeyboardButton KeyboardButton_Lang3;
            KeyboardButton KeyboardButton_Lang4;
            KeyboardButton KeyboardButton_Lang5;
            KeyboardButton KeyboardButton_NonConvert;
            KeyboardButton KeyboardButton_Delete;
            KeyboardButton KeyboardButton_End;
            KeyboardButton KeyboardButton_Help;
            KeyboardButton KeyboardButton_Home;
            KeyboardButton KeyboardButton_Insert;
            KeyboardButton KeyboardButton_PageDown;
            KeyboardButton KeyboardButton_PageUp;
            KeyboardButton KeyboardButton_Down;
            KeyboardButton KeyboardButton_Left;
            KeyboardButton KeyboardButton_Right;
            KeyboardButton KeyboardButton_Up;
            KeyboardButton KeyboardButton_NumLock;
            KeyboardButton KeyboardButton_Numpad0;
            KeyboardButton KeyboardButton_Numpad1;
            KeyboardButton KeyboardButton_Numpad2;
            KeyboardButton KeyboardButton_Numpad3;
            KeyboardButton KeyboardButton_Numpad4;
            KeyboardButton KeyboardButton_Numpad5;
            KeyboardButton KeyboardButton_Numpad6;
            KeyboardButton KeyboardButton_Numpad7;
            KeyboardButton KeyboardButton_Numpad8;
            KeyboardButton KeyboardButton_Numpad9;
            KeyboardButton KeyboardButton_NumpadAdd;
            KeyboardButton KeyboardButton_NumpadBackspace;
            KeyboardButton KeyboardButton_NumpadClear;
            KeyboardButton KeyboardButton_NumpadClearEntry;
            KeyboardButton KeyboardButton_NumpadComma;
            KeyboardButton KeyboardButton_NumpadDecimal;
            KeyboardButton KeyboardButton_NumpadDivide;
            KeyboardButton KeyboardButton_NumpadEnter;
            KeyboardButton KeyboardButton_NumpadEqual;
            KeyboardButton KeyboardButton_NumpadHash;
            KeyboardButton KeyboardButton_NumpadMemoryAdd;
            KeyboardButton KeyboardButton_NumpadMemoryClear;
            KeyboardButton KeyboardButton_NumpadMemoryRecall;
            KeyboardButton KeyboardButton_NumpadMemoryStore;
            KeyboardButton KeyboardButton_NumpadMemorySubtract;
            KeyboardButton KeyboardButton_NumpadMultiply;
            KeyboardButton KeyboardButton_NumpadParenLeft;
            KeyboardButton KeyboardButton_NumpadParenRight;
            KeyboardButton KeyboardButton_NumpadStar;
            KeyboardButton KeyboardButton_NumpadSubtract;
            KeyboardButton KeyboardButton_Escape;
            KeyboardButton KeyboardButton_Fn;
            KeyboardButton KeyboardButton_FnLock;
            KeyboardButton KeyboardButton_PrintScreen;
            KeyboardButton KeyboardButton_ScrollLock;
            KeyboardButton KeyboardButton_Pause;
            KeyboardButton KeyboardButton_BrowserBack;
            KeyboardButton KeyboardButton_BrowserFavorites;
            KeyboardButton KeyboardButton_BrowserForward;
            KeyboardButton KeyboardButton_BrowserHome;
            KeyboardButton KeyboardButton_BrowserRefresh;
            KeyboardButton KeyboardButton_BrowserSearch;
            KeyboardButton KeyboardButton_BrowserStop;
            KeyboardButton KeyboardButton_Eject;
            KeyboardButton KeyboardButton_LaunchApp1;
            KeyboardButton KeyboardButton_LaunchApp2;
            KeyboardButton KeyboardButton_LaunchMail;
            KeyboardButton KeyboardButton_MediaPlayPause;
            KeyboardButton KeyboardButton_MediaSelect;
            KeyboardButton KeyboardButton_MediaStop;
            KeyboardButton KeyboardButton_MediaTrackNext;
            KeyboardButton KeyboardButton_MediaTrackPrevious;
            KeyboardButton KeyboardButton_Power;
            KeyboardButton KeyboardButton_Sleep;
            KeyboardButton KeyboardButton_AudioVolumeDown;
            KeyboardButton KeyboardButton_AudioVolumeMute;
            KeyboardButton KeyboardButton_AudioVolumeUp;
            KeyboardButton KeyboardButton_WakeUp;
            KeyboardButton KeyboardButton_Meta;
            KeyboardButton KeyboardButton_Hyper;
            KeyboardButton KeyboardButton_Turbo;
            KeyboardButton KeyboardButton_Abort;
            KeyboardButton KeyboardButton_Resume;
            KeyboardButton KeyboardButton_Suspend;
            KeyboardButton KeyboardButton_Again;
            KeyboardButton KeyboardButton_Copy;
            KeyboardButton KeyboardButton_Cut;
            KeyboardButton KeyboardButton_Find;
            KeyboardButton KeyboardButton_Open;
            KeyboardButton KeyboardButton_Paste;
            KeyboardButton KeyboardButton_Props;
            KeyboardButton KeyboardButton_Select;
            KeyboardButton KeyboardButton_Undo;
            KeyboardButton KeyboardButton_Hiragana;
            KeyboardButton KeyboardButton_Katakana;
            KeyboardButton KeyboardButton_F1;
            KeyboardButton KeyboardButton_F2;
            KeyboardButton KeyboardButton_F3;
            KeyboardButton KeyboardButton_F4;
            KeyboardButton KeyboardButton_F5;
            KeyboardButton KeyboardButton_F6;
            KeyboardButton KeyboardButton_F7;
            KeyboardButton KeyboardButton_F8;
            KeyboardButton KeyboardButton_F9;
            KeyboardButton KeyboardButton_F10;
            KeyboardButton KeyboardButton_F11;
            KeyboardButton KeyboardButton_F12;
            KeyboardButton KeyboardButton_F13;
            KeyboardButton KeyboardButton_F14;
            KeyboardButton KeyboardButton_F15;
            KeyboardButton KeyboardButton_F16;
            KeyboardButton KeyboardButton_F17;
            KeyboardButton KeyboardButton_F18;
            KeyboardButton KeyboardButton_F19;
            KeyboardButton KeyboardButton_F20;
            KeyboardButton KeyboardButton_F21;
            KeyboardButton KeyboardButton_F22;
            KeyboardButton KeyboardButton_F23;
            KeyboardButton KeyboardButton_F24;
            KeyboardButton KeyboardButton_F25;
            KeyboardButton KeyboardButton_F26;
            KeyboardButton KeyboardButton_F27;
            KeyboardButton KeyboardButton_F28;
            KeyboardButton KeyboardButton_F29;
            KeyboardButton KeyboardButton_F30;
            KeyboardButton KeyboardButton_F31;
            KeyboardButton KeyboardButton_F32;
            KeyboardButton KeyboardButton_F33;
            KeyboardButton KeyboardButton_F34;
            KeyboardButton KeyboardButton_F35;

            cstr           KeyboardButton_ToString(KeyboardButton);
        ]]
    end

    do -- Global Symbol Table
        KeyboardButton = {
            Backquote            = libphx.KeyboardButton_Backquote,
            Backslash            = libphx.KeyboardButton_Backslash,
            BracketLeft          = libphx.KeyboardButton_BracketLeft,
            BracketRight         = libphx.KeyboardButton_BracketRight,
            Comma                = libphx.KeyboardButton_Comma,
            Key0                 = libphx.KeyboardButton_Key0,
            Key1                 = libphx.KeyboardButton_Key1,
            Key2                 = libphx.KeyboardButton_Key2,
            Key3                 = libphx.KeyboardButton_Key3,
            Key4                 = libphx.KeyboardButton_Key4,
            Key5                 = libphx.KeyboardButton_Key5,
            Key6                 = libphx.KeyboardButton_Key6,
            Key7                 = libphx.KeyboardButton_Key7,
            Key8                 = libphx.KeyboardButton_Key8,
            Key9                 = libphx.KeyboardButton_Key9,
            Equal                = libphx.KeyboardButton_Equal,
            IntlBackslash        = libphx.KeyboardButton_IntlBackslash,
            IntlRo               = libphx.KeyboardButton_IntlRo,
            IntlYen              = libphx.KeyboardButton_IntlYen,
            A                    = libphx.KeyboardButton_A,
            B                    = libphx.KeyboardButton_B,
            C                    = libphx.KeyboardButton_C,
            D                    = libphx.KeyboardButton_D,
            E                    = libphx.KeyboardButton_E,
            F                    = libphx.KeyboardButton_F,
            G                    = libphx.KeyboardButton_G,
            H                    = libphx.KeyboardButton_H,
            I                    = libphx.KeyboardButton_I,
            J                    = libphx.KeyboardButton_J,
            K                    = libphx.KeyboardButton_K,
            L                    = libphx.KeyboardButton_L,
            M                    = libphx.KeyboardButton_M,
            N                    = libphx.KeyboardButton_N,
            O                    = libphx.KeyboardButton_O,
            P                    = libphx.KeyboardButton_P,
            Q                    = libphx.KeyboardButton_Q,
            R                    = libphx.KeyboardButton_R,
            S                    = libphx.KeyboardButton_S,
            T                    = libphx.KeyboardButton_T,
            U                    = libphx.KeyboardButton_U,
            V                    = libphx.KeyboardButton_V,
            W                    = libphx.KeyboardButton_W,
            X                    = libphx.KeyboardButton_X,
            Y                    = libphx.KeyboardButton_Y,
            Z                    = libphx.KeyboardButton_Z,
            Minus                = libphx.KeyboardButton_Minus,
            Period               = libphx.KeyboardButton_Period,
            Quote                = libphx.KeyboardButton_Quote,
            Semicolon            = libphx.KeyboardButton_Semicolon,
            Slash                = libphx.KeyboardButton_Slash,
            AltLeft              = libphx.KeyboardButton_AltLeft,
            AltRight             = libphx.KeyboardButton_AltRight,
            Backspace            = libphx.KeyboardButton_Backspace,
            CapsLock             = libphx.KeyboardButton_CapsLock,
            ContextMenu          = libphx.KeyboardButton_ContextMenu,
            ControlLeft          = libphx.KeyboardButton_ControlLeft,
            ControlRight         = libphx.KeyboardButton_ControlRight,
            Enter                = libphx.KeyboardButton_Enter,
            SuperLeft            = libphx.KeyboardButton_SuperLeft,
            SuperRight           = libphx.KeyboardButton_SuperRight,
            ShiftLeft            = libphx.KeyboardButton_ShiftLeft,
            ShiftRight           = libphx.KeyboardButton_ShiftRight,
            Space                = libphx.KeyboardButton_Space,
            Tab                  = libphx.KeyboardButton_Tab,
            Convert              = libphx.KeyboardButton_Convert,
            KanaMode             = libphx.KeyboardButton_KanaMode,
            Lang1                = libphx.KeyboardButton_Lang1,
            Lang2                = libphx.KeyboardButton_Lang2,
            Lang3                = libphx.KeyboardButton_Lang3,
            Lang4                = libphx.KeyboardButton_Lang4,
            Lang5                = libphx.KeyboardButton_Lang5,
            NonConvert           = libphx.KeyboardButton_NonConvert,
            Delete               = libphx.KeyboardButton_Delete,
            End                  = libphx.KeyboardButton_End,
            Help                 = libphx.KeyboardButton_Help,
            Home                 = libphx.KeyboardButton_Home,
            Insert               = libphx.KeyboardButton_Insert,
            PageDown             = libphx.KeyboardButton_PageDown,
            PageUp               = libphx.KeyboardButton_PageUp,
            Down                 = libphx.KeyboardButton_Down,
            Left                 = libphx.KeyboardButton_Left,
            Right                = libphx.KeyboardButton_Right,
            Up                   = libphx.KeyboardButton_Up,
            NumLock              = libphx.KeyboardButton_NumLock,
            Numpad0              = libphx.KeyboardButton_Numpad0,
            Numpad1              = libphx.KeyboardButton_Numpad1,
            Numpad2              = libphx.KeyboardButton_Numpad2,
            Numpad3              = libphx.KeyboardButton_Numpad3,
            Numpad4              = libphx.KeyboardButton_Numpad4,
            Numpad5              = libphx.KeyboardButton_Numpad5,
            Numpad6              = libphx.KeyboardButton_Numpad6,
            Numpad7              = libphx.KeyboardButton_Numpad7,
            Numpad8              = libphx.KeyboardButton_Numpad8,
            Numpad9              = libphx.KeyboardButton_Numpad9,
            NumpadAdd            = libphx.KeyboardButton_NumpadAdd,
            NumpadBackspace      = libphx.KeyboardButton_NumpadBackspace,
            NumpadClear          = libphx.KeyboardButton_NumpadClear,
            NumpadClearEntry     = libphx.KeyboardButton_NumpadClearEntry,
            NumpadComma          = libphx.KeyboardButton_NumpadComma,
            NumpadDecimal        = libphx.KeyboardButton_NumpadDecimal,
            NumpadDivide         = libphx.KeyboardButton_NumpadDivide,
            NumpadEnter          = libphx.KeyboardButton_NumpadEnter,
            NumpadEqual          = libphx.KeyboardButton_NumpadEqual,
            NumpadHash           = libphx.KeyboardButton_NumpadHash,
            NumpadMemoryAdd      = libphx.KeyboardButton_NumpadMemoryAdd,
            NumpadMemoryClear    = libphx.KeyboardButton_NumpadMemoryClear,
            NumpadMemoryRecall   = libphx.KeyboardButton_NumpadMemoryRecall,
            NumpadMemoryStore    = libphx.KeyboardButton_NumpadMemoryStore,
            NumpadMemorySubtract = libphx.KeyboardButton_NumpadMemorySubtract,
            NumpadMultiply       = libphx.KeyboardButton_NumpadMultiply,
            NumpadParenLeft      = libphx.KeyboardButton_NumpadParenLeft,
            NumpadParenRight     = libphx.KeyboardButton_NumpadParenRight,
            NumpadStar           = libphx.KeyboardButton_NumpadStar,
            NumpadSubtract       = libphx.KeyboardButton_NumpadSubtract,
            Escape               = libphx.KeyboardButton_Escape,
            Fn                   = libphx.KeyboardButton_Fn,
            FnLock               = libphx.KeyboardButton_FnLock,
            PrintScreen          = libphx.KeyboardButton_PrintScreen,
            ScrollLock           = libphx.KeyboardButton_ScrollLock,
            Pause                = libphx.KeyboardButton_Pause,
            BrowserBack          = libphx.KeyboardButton_BrowserBack,
            BrowserFavorites     = libphx.KeyboardButton_BrowserFavorites,
            BrowserForward       = libphx.KeyboardButton_BrowserForward,
            BrowserHome          = libphx.KeyboardButton_BrowserHome,
            BrowserRefresh       = libphx.KeyboardButton_BrowserRefresh,
            BrowserSearch        = libphx.KeyboardButton_BrowserSearch,
            BrowserStop          = libphx.KeyboardButton_BrowserStop,
            Eject                = libphx.KeyboardButton_Eject,
            LaunchApp1           = libphx.KeyboardButton_LaunchApp1,
            LaunchApp2           = libphx.KeyboardButton_LaunchApp2,
            LaunchMail           = libphx.KeyboardButton_LaunchMail,
            MediaPlayPause       = libphx.KeyboardButton_MediaPlayPause,
            MediaSelect          = libphx.KeyboardButton_MediaSelect,
            MediaStop            = libphx.KeyboardButton_MediaStop,
            MediaTrackNext       = libphx.KeyboardButton_MediaTrackNext,
            MediaTrackPrevious   = libphx.KeyboardButton_MediaTrackPrevious,
            Power                = libphx.KeyboardButton_Power,
            Sleep                = libphx.KeyboardButton_Sleep,
            AudioVolumeDown      = libphx.KeyboardButton_AudioVolumeDown,
            AudioVolumeMute      = libphx.KeyboardButton_AudioVolumeMute,
            AudioVolumeUp        = libphx.KeyboardButton_AudioVolumeUp,
            WakeUp               = libphx.KeyboardButton_WakeUp,
            Meta                 = libphx.KeyboardButton_Meta,
            Hyper                = libphx.KeyboardButton_Hyper,
            Turbo                = libphx.KeyboardButton_Turbo,
            Abort                = libphx.KeyboardButton_Abort,
            Resume               = libphx.KeyboardButton_Resume,
            Suspend              = libphx.KeyboardButton_Suspend,
            Again                = libphx.KeyboardButton_Again,
            Copy                 = libphx.KeyboardButton_Copy,
            Cut                  = libphx.KeyboardButton_Cut,
            Find                 = libphx.KeyboardButton_Find,
            Open                 = libphx.KeyboardButton_Open,
            Paste                = libphx.KeyboardButton_Paste,
            Props                = libphx.KeyboardButton_Props,
            Select               = libphx.KeyboardButton_Select,
            Undo                 = libphx.KeyboardButton_Undo,
            Hiragana             = libphx.KeyboardButton_Hiragana,
            Katakana             = libphx.KeyboardButton_Katakana,
            F1                   = libphx.KeyboardButton_F1,
            F2                   = libphx.KeyboardButton_F2,
            F3                   = libphx.KeyboardButton_F3,
            F4                   = libphx.KeyboardButton_F4,
            F5                   = libphx.KeyboardButton_F5,
            F6                   = libphx.KeyboardButton_F6,
            F7                   = libphx.KeyboardButton_F7,
            F8                   = libphx.KeyboardButton_F8,
            F9                   = libphx.KeyboardButton_F9,
            F10                  = libphx.KeyboardButton_F10,
            F11                  = libphx.KeyboardButton_F11,
            F12                  = libphx.KeyboardButton_F12,
            F13                  = libphx.KeyboardButton_F13,
            F14                  = libphx.KeyboardButton_F14,
            F15                  = libphx.KeyboardButton_F15,
            F16                  = libphx.KeyboardButton_F16,
            F17                  = libphx.KeyboardButton_F17,
            F18                  = libphx.KeyboardButton_F18,
            F19                  = libphx.KeyboardButton_F19,
            F20                  = libphx.KeyboardButton_F20,
            F21                  = libphx.KeyboardButton_F21,
            F22                  = libphx.KeyboardButton_F22,
            F23                  = libphx.KeyboardButton_F23,
            F24                  = libphx.KeyboardButton_F24,
            F25                  = libphx.KeyboardButton_F25,
            F26                  = libphx.KeyboardButton_F26,
            F27                  = libphx.KeyboardButton_F27,
            F28                  = libphx.KeyboardButton_F28,
            F29                  = libphx.KeyboardButton_F29,
            F30                  = libphx.KeyboardButton_F30,
            F31                  = libphx.KeyboardButton_F31,
            F32                  = libphx.KeyboardButton_F32,
            F33                  = libphx.KeyboardButton_F33,
            F34                  = libphx.KeyboardButton_F34,
            F35                  = libphx.KeyboardButton_F35,

            ToString             = libphx.KeyboardButton_ToString,
        }

        if onDef_KeyboardButton then onDef_KeyboardButton(KeyboardButton, mt) end
        KeyboardButton = setmetatable(KeyboardButton, mt)
    end

    return KeyboardButton
end

return Loader
