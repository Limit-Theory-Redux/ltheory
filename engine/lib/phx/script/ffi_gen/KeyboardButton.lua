-- KeyboardButton --------------------------------------------------------------

function declareType()
    ffi.cdef [[
        typedef uint8 KeyboardButton;
    ]]

    return 2, 'KeyboardButton'
end

function defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local KeyboardButton

    do -- C Definitions
        ffi.cdef [[
            KeyboardButton KeyboardButton_Key1;
            KeyboardButton KeyboardButton_Key2;
            KeyboardButton KeyboardButton_Key3;
            KeyboardButton KeyboardButton_Key4;
            KeyboardButton KeyboardButton_Key5;
            KeyboardButton KeyboardButton_Key6;
            KeyboardButton KeyboardButton_Key7;
            KeyboardButton KeyboardButton_Key8;
            KeyboardButton KeyboardButton_Key9;
            KeyboardButton KeyboardButton_Key0;
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
            KeyboardButton KeyboardButton_Escape;
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
            KeyboardButton KeyboardButton_Snapshot;
            KeyboardButton KeyboardButton_Scroll;
            KeyboardButton KeyboardButton_Pause;
            KeyboardButton KeyboardButton_Insert;
            KeyboardButton KeyboardButton_Home;
            KeyboardButton KeyboardButton_Delete;
            KeyboardButton KeyboardButton_End;
            KeyboardButton KeyboardButton_PageDown;
            KeyboardButton KeyboardButton_PageUp;
            KeyboardButton KeyboardButton_Left;
            KeyboardButton KeyboardButton_Up;
            KeyboardButton KeyboardButton_Right;
            KeyboardButton KeyboardButton_Down;
            KeyboardButton KeyboardButton_Back;
            KeyboardButton KeyboardButton_Return;
            KeyboardButton KeyboardButton_Space;
            KeyboardButton KeyboardButton_Compose;
            KeyboardButton KeyboardButton_Caret;
            KeyboardButton KeyboardButton_Numlock;
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
            KeyboardButton KeyboardButton_AbntC1;
            KeyboardButton KeyboardButton_AbntC2;
            KeyboardButton KeyboardButton_NumpadAdd;
            KeyboardButton KeyboardButton_Apostrophe;
            KeyboardButton KeyboardButton_Apps;
            KeyboardButton KeyboardButton_Asterisk;
            KeyboardButton KeyboardButton_Plus;
            KeyboardButton KeyboardButton_At;
            KeyboardButton KeyboardButton_Ax;
            KeyboardButton KeyboardButton_Backslash;
            KeyboardButton KeyboardButton_Calculator;
            KeyboardButton KeyboardButton_Capital;
            KeyboardButton KeyboardButton_Colon;
            KeyboardButton KeyboardButton_Comma;
            KeyboardButton KeyboardButton_Convert;
            KeyboardButton KeyboardButton_NumpadDecimal;
            KeyboardButton KeyboardButton_NumpadDivide;
            KeyboardButton KeyboardButton_Equals;
            KeyboardButton KeyboardButton_Grave;
            KeyboardButton KeyboardButton_Kana;
            KeyboardButton KeyboardButton_Kanji;
            KeyboardButton KeyboardButton_AltLeft;
            KeyboardButton KeyboardButton_BracketLeft;
            KeyboardButton KeyboardButton_ControlLeft;
            KeyboardButton KeyboardButton_ShiftLeft;
            KeyboardButton KeyboardButton_SuperLeft;
            KeyboardButton KeyboardButton_Mail;
            KeyboardButton KeyboardButton_MediaSelect;
            KeyboardButton KeyboardButton_MediaStop;
            KeyboardButton KeyboardButton_Minus;
            KeyboardButton KeyboardButton_NumpadMultiply;
            KeyboardButton KeyboardButton_Mute;
            KeyboardButton KeyboardButton_MyComputer;
            KeyboardButton KeyboardButton_NavigateForward;
            KeyboardButton KeyboardButton_NavigateBackward;
            KeyboardButton KeyboardButton_NextTrack;
            KeyboardButton KeyboardButton_NoConvert;
            KeyboardButton KeyboardButton_NumpadComma;
            KeyboardButton KeyboardButton_NumpadEnter;
            KeyboardButton KeyboardButton_NumpadEquals;
            KeyboardButton KeyboardButton_Oem102;
            KeyboardButton KeyboardButton_Period;
            KeyboardButton KeyboardButton_PlayPause;
            KeyboardButton KeyboardButton_Power;
            KeyboardButton KeyboardButton_PrevTrack;
            KeyboardButton KeyboardButton_AltRight;
            KeyboardButton KeyboardButton_BracketRight;
            KeyboardButton KeyboardButton_ControlRight;
            KeyboardButton KeyboardButton_ShiftRight;
            KeyboardButton KeyboardButton_SuperRight;
            KeyboardButton KeyboardButton_Semicolon;
            KeyboardButton KeyboardButton_Slash;
            KeyboardButton KeyboardButton_Sleep;
            KeyboardButton KeyboardButton_Stop;
            KeyboardButton KeyboardButton_NumpadSubtract;
            KeyboardButton KeyboardButton_Sysrq;
            KeyboardButton KeyboardButton_Tab;
            KeyboardButton KeyboardButton_Underline;
            KeyboardButton KeyboardButton_Unlabeled;
            KeyboardButton KeyboardButton_VolumeDown;
            KeyboardButton KeyboardButton_VolumeUp;
            KeyboardButton KeyboardButton_Wake;
            KeyboardButton KeyboardButton_WebBack;
            KeyboardButton KeyboardButton_WebFavorites;
            KeyboardButton KeyboardButton_WebForward;
            KeyboardButton KeyboardButton_WebHome;
            KeyboardButton KeyboardButton_WebRefresh;
            KeyboardButton KeyboardButton_WebSearch;
            KeyboardButton KeyboardButton_WebStop;
            KeyboardButton KeyboardButton_Yen;
            KeyboardButton KeyboardButton_Copy;
            KeyboardButton KeyboardButton_Paste;
            KeyboardButton KeyboardButton_Cut;

            cstr           KeyboardButton_ToString(KeyboardButton);
        ]]
    end

    do -- Global Symbol Table
        KeyboardButton = {
            Key1             = libphx.KeyboardButton_Key1,
            Key2             = libphx.KeyboardButton_Key2,
            Key3             = libphx.KeyboardButton_Key3,
            Key4             = libphx.KeyboardButton_Key4,
            Key5             = libphx.KeyboardButton_Key5,
            Key6             = libphx.KeyboardButton_Key6,
            Key7             = libphx.KeyboardButton_Key7,
            Key8             = libphx.KeyboardButton_Key8,
            Key9             = libphx.KeyboardButton_Key9,
            Key0             = libphx.KeyboardButton_Key0,
            A                = libphx.KeyboardButton_A,
            B                = libphx.KeyboardButton_B,
            C                = libphx.KeyboardButton_C,
            D                = libphx.KeyboardButton_D,
            E                = libphx.KeyboardButton_E,
            F                = libphx.KeyboardButton_F,
            G                = libphx.KeyboardButton_G,
            H                = libphx.KeyboardButton_H,
            I                = libphx.KeyboardButton_I,
            J                = libphx.KeyboardButton_J,
            K                = libphx.KeyboardButton_K,
            L                = libphx.KeyboardButton_L,
            M                = libphx.KeyboardButton_M,
            N                = libphx.KeyboardButton_N,
            O                = libphx.KeyboardButton_O,
            P                = libphx.KeyboardButton_P,
            Q                = libphx.KeyboardButton_Q,
            R                = libphx.KeyboardButton_R,
            S                = libphx.KeyboardButton_S,
            T                = libphx.KeyboardButton_T,
            U                = libphx.KeyboardButton_U,
            V                = libphx.KeyboardButton_V,
            W                = libphx.KeyboardButton_W,
            X                = libphx.KeyboardButton_X,
            Y                = libphx.KeyboardButton_Y,
            Z                = libphx.KeyboardButton_Z,
            Escape           = libphx.KeyboardButton_Escape,
            F1               = libphx.KeyboardButton_F1,
            F2               = libphx.KeyboardButton_F2,
            F3               = libphx.KeyboardButton_F3,
            F4               = libphx.KeyboardButton_F4,
            F5               = libphx.KeyboardButton_F5,
            F6               = libphx.KeyboardButton_F6,
            F7               = libphx.KeyboardButton_F7,
            F8               = libphx.KeyboardButton_F8,
            F9               = libphx.KeyboardButton_F9,
            F10              = libphx.KeyboardButton_F10,
            F11              = libphx.KeyboardButton_F11,
            F12              = libphx.KeyboardButton_F12,
            F13              = libphx.KeyboardButton_F13,
            F14              = libphx.KeyboardButton_F14,
            F15              = libphx.KeyboardButton_F15,
            F16              = libphx.KeyboardButton_F16,
            F17              = libphx.KeyboardButton_F17,
            F18              = libphx.KeyboardButton_F18,
            F19              = libphx.KeyboardButton_F19,
            F20              = libphx.KeyboardButton_F20,
            F21              = libphx.KeyboardButton_F21,
            F22              = libphx.KeyboardButton_F22,
            F23              = libphx.KeyboardButton_F23,
            F24              = libphx.KeyboardButton_F24,
            Snapshot         = libphx.KeyboardButton_Snapshot,
            Scroll           = libphx.KeyboardButton_Scroll,
            Pause            = libphx.KeyboardButton_Pause,
            Insert           = libphx.KeyboardButton_Insert,
            Home             = libphx.KeyboardButton_Home,
            Delete           = libphx.KeyboardButton_Delete,
            End              = libphx.KeyboardButton_End,
            PageDown         = libphx.KeyboardButton_PageDown,
            PageUp           = libphx.KeyboardButton_PageUp,
            Left             = libphx.KeyboardButton_Left,
            Up               = libphx.KeyboardButton_Up,
            Right            = libphx.KeyboardButton_Right,
            Down             = libphx.KeyboardButton_Down,
            Back             = libphx.KeyboardButton_Back,
            Return           = libphx.KeyboardButton_Return,
            Space            = libphx.KeyboardButton_Space,
            Compose          = libphx.KeyboardButton_Compose,
            Caret            = libphx.KeyboardButton_Caret,
            Numlock          = libphx.KeyboardButton_Numlock,
            Numpad0          = libphx.KeyboardButton_Numpad0,
            Numpad1          = libphx.KeyboardButton_Numpad1,
            Numpad2          = libphx.KeyboardButton_Numpad2,
            Numpad3          = libphx.KeyboardButton_Numpad3,
            Numpad4          = libphx.KeyboardButton_Numpad4,
            Numpad5          = libphx.KeyboardButton_Numpad5,
            Numpad6          = libphx.KeyboardButton_Numpad6,
            Numpad7          = libphx.KeyboardButton_Numpad7,
            Numpad8          = libphx.KeyboardButton_Numpad8,
            Numpad9          = libphx.KeyboardButton_Numpad9,
            AbntC1           = libphx.KeyboardButton_AbntC1,
            AbntC2           = libphx.KeyboardButton_AbntC2,
            NumpadAdd        = libphx.KeyboardButton_NumpadAdd,
            Apostrophe       = libphx.KeyboardButton_Apostrophe,
            Apps             = libphx.KeyboardButton_Apps,
            Asterisk         = libphx.KeyboardButton_Asterisk,
            Plus             = libphx.KeyboardButton_Plus,
            At               = libphx.KeyboardButton_At,
            Ax               = libphx.KeyboardButton_Ax,
            Backslash        = libphx.KeyboardButton_Backslash,
            Calculator       = libphx.KeyboardButton_Calculator,
            Capital          = libphx.KeyboardButton_Capital,
            Colon            = libphx.KeyboardButton_Colon,
            Comma            = libphx.KeyboardButton_Comma,
            Convert          = libphx.KeyboardButton_Convert,
            NumpadDecimal    = libphx.KeyboardButton_NumpadDecimal,
            NumpadDivide     = libphx.KeyboardButton_NumpadDivide,
            Equals           = libphx.KeyboardButton_Equals,
            Grave            = libphx.KeyboardButton_Grave,
            Kana             = libphx.KeyboardButton_Kana,
            Kanji            = libphx.KeyboardButton_Kanji,
            AltLeft          = libphx.KeyboardButton_AltLeft,
            BracketLeft      = libphx.KeyboardButton_BracketLeft,
            ControlLeft      = libphx.KeyboardButton_ControlLeft,
            ShiftLeft        = libphx.KeyboardButton_ShiftLeft,
            SuperLeft        = libphx.KeyboardButton_SuperLeft,
            Mail             = libphx.KeyboardButton_Mail,
            MediaSelect      = libphx.KeyboardButton_MediaSelect,
            MediaStop        = libphx.KeyboardButton_MediaStop,
            Minus            = libphx.KeyboardButton_Minus,
            NumpadMultiply   = libphx.KeyboardButton_NumpadMultiply,
            Mute             = libphx.KeyboardButton_Mute,
            MyComputer       = libphx.KeyboardButton_MyComputer,
            NavigateForward  = libphx.KeyboardButton_NavigateForward,
            NavigateBackward = libphx.KeyboardButton_NavigateBackward,
            NextTrack        = libphx.KeyboardButton_NextTrack,
            NoConvert        = libphx.KeyboardButton_NoConvert,
            NumpadComma      = libphx.KeyboardButton_NumpadComma,
            NumpadEnter      = libphx.KeyboardButton_NumpadEnter,
            NumpadEquals     = libphx.KeyboardButton_NumpadEquals,
            Oem102           = libphx.KeyboardButton_Oem102,
            Period           = libphx.KeyboardButton_Period,
            PlayPause        = libphx.KeyboardButton_PlayPause,
            Power            = libphx.KeyboardButton_Power,
            PrevTrack        = libphx.KeyboardButton_PrevTrack,
            AltRight         = libphx.KeyboardButton_AltRight,
            BracketRight     = libphx.KeyboardButton_BracketRight,
            ControlRight     = libphx.KeyboardButton_ControlRight,
            ShiftRight       = libphx.KeyboardButton_ShiftRight,
            SuperRight       = libphx.KeyboardButton_SuperRight,
            Semicolon        = libphx.KeyboardButton_Semicolon,
            Slash            = libphx.KeyboardButton_Slash,
            Sleep            = libphx.KeyboardButton_Sleep,
            Stop             = libphx.KeyboardButton_Stop,
            NumpadSubtract   = libphx.KeyboardButton_NumpadSubtract,
            Sysrq            = libphx.KeyboardButton_Sysrq,
            Tab              = libphx.KeyboardButton_Tab,
            Underline        = libphx.KeyboardButton_Underline,
            Unlabeled        = libphx.KeyboardButton_Unlabeled,
            VolumeDown       = libphx.KeyboardButton_VolumeDown,
            VolumeUp         = libphx.KeyboardButton_VolumeUp,
            Wake             = libphx.KeyboardButton_Wake,
            WebBack          = libphx.KeyboardButton_WebBack,
            WebFavorites     = libphx.KeyboardButton_WebFavorites,
            WebForward       = libphx.KeyboardButton_WebForward,
            WebHome          = libphx.KeyboardButton_WebHome,
            WebRefresh       = libphx.KeyboardButton_WebRefresh,
            WebSearch        = libphx.KeyboardButton_WebSearch,
            WebStop          = libphx.KeyboardButton_WebStop,
            Yen              = libphx.KeyboardButton_Yen,
            Copy             = libphx.KeyboardButton_Copy,
            Paste            = libphx.KeyboardButton_Paste,
            Cut              = libphx.KeyboardButton_Cut,

            ToString         = libphx.KeyboardButton_ToString,
        }

        if onDef_KeyboardButton then onDef_KeyboardButton(KeyboardButton, mt) end
        KeyboardButton = setmetatable(KeyboardButton, mt)
    end

    return KeyboardButton
end

