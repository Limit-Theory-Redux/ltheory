-- Button2 ---------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local Button2

do -- C Definitions
    ffi.cdef [[
        typedef uint8 Button2;

        Button2 Button2_KeyboardKey1;
        Button2 Button2_KeyboardKey2;
        Button2 Button2_KeyboardKey3;
        Button2 Button2_KeyboardKey4;
        Button2 Button2_KeyboardKey5;
        Button2 Button2_KeyboardKey6;
        Button2 Button2_KeyboardKey7;
        Button2 Button2_KeyboardKey8;
        Button2 Button2_KeyboardKey9;
        Button2 Button2_KeyboardKey0;
        Button2 Button2_KeyboardA;
        Button2 Button2_KeyboardB;
        Button2 Button2_KeyboardC;
        Button2 Button2_KeyboardD;
        Button2 Button2_KeyboardE;
        Button2 Button2_KeyboardF;
        Button2 Button2_KeyboardG;
        Button2 Button2_KeyboardH;
        Button2 Button2_KeyboardI;
        Button2 Button2_KeyboardJ;
        Button2 Button2_KeyboardK;
        Button2 Button2_KeyboardL;
        Button2 Button2_KeyboardM;
        Button2 Button2_KeyboardN;
        Button2 Button2_KeyboardO;
        Button2 Button2_KeyboardP;
        Button2 Button2_KeyboardQ;
        Button2 Button2_KeyboardR;
        Button2 Button2_KeyboardS;
        Button2 Button2_KeyboardT;
        Button2 Button2_KeyboardU;
        Button2 Button2_KeyboardV;
        Button2 Button2_KeyboardW;
        Button2 Button2_KeyboardX;
        Button2 Button2_KeyboardY;
        Button2 Button2_KeyboardZ;
        Button2 Button2_KeyboardEscape;
        Button2 Button2_KeyboardF1;
        Button2 Button2_KeyboardF2;
        Button2 Button2_KeyboardF3;
        Button2 Button2_KeyboardF4;
        Button2 Button2_KeyboardF5;
        Button2 Button2_KeyboardF6;
        Button2 Button2_KeyboardF7;
        Button2 Button2_KeyboardF8;
        Button2 Button2_KeyboardF9;
        Button2 Button2_KeyboardF10;
        Button2 Button2_KeyboardF11;
        Button2 Button2_KeyboardF12;
        Button2 Button2_KeyboardF13;
        Button2 Button2_KeyboardF14;
        Button2 Button2_KeyboardF15;
        Button2 Button2_KeyboardF16;
        Button2 Button2_KeyboardF17;
        Button2 Button2_KeyboardF18;
        Button2 Button2_KeyboardF19;
        Button2 Button2_KeyboardF20;
        Button2 Button2_KeyboardF21;
        Button2 Button2_KeyboardF22;
        Button2 Button2_KeyboardF23;
        Button2 Button2_KeyboardF24;
        Button2 Button2_KeyboardSnapshot;
        Button2 Button2_KeyboardScroll;
        Button2 Button2_KeyboardPause;
        Button2 Button2_KeyboardInsert;
        Button2 Button2_KeyboardHome;
        Button2 Button2_KeyboardDelete;
        Button2 Button2_KeyboardEnd;
        Button2 Button2_KeyboardPageDown;
        Button2 Button2_KeyboardPageUp;
        Button2 Button2_KeyboardLeft;
        Button2 Button2_KeyboardUp;
        Button2 Button2_KeyboardRight;
        Button2 Button2_KeyboardDown;
        Button2 Button2_KeyboardBack;
        Button2 Button2_KeyboardReturn;
        Button2 Button2_KeyboardSpace;
        Button2 Button2_KeyboardCompose;
        Button2 Button2_KeyboardCaret;
        Button2 Button2_KeyboardNumlock;
        Button2 Button2_KeyboardNumpad0;
        Button2 Button2_KeyboardNumpad1;
        Button2 Button2_KeyboardNumpad2;
        Button2 Button2_KeyboardNumpad3;
        Button2 Button2_KeyboardNumpad4;
        Button2 Button2_KeyboardNumpad5;
        Button2 Button2_KeyboardNumpad6;
        Button2 Button2_KeyboardNumpad7;
        Button2 Button2_KeyboardNumpad8;
        Button2 Button2_KeyboardNumpad9;
        Button2 Button2_KeyboardAbntC1;
        Button2 Button2_KeyboardAbntC2;
        Button2 Button2_KeyboardNumpadAdd;
        Button2 Button2_KeyboardApostrophe;
        Button2 Button2_KeyboardApps;
        Button2 Button2_KeyboardAsterisk;
        Button2 Button2_KeyboardPlus;
        Button2 Button2_KeyboardAt;
        Button2 Button2_KeyboardAx;
        Button2 Button2_KeyboardBackslash;
        Button2 Button2_KeyboardCalculator;
        Button2 Button2_KeyboardCapital;
        Button2 Button2_KeyboardColon;
        Button2 Button2_KeyboardComma;
        Button2 Button2_KeyboardConvert;
        Button2 Button2_KeyboardNumpadDecimal;
        Button2 Button2_KeyboardNumpadDivide;
        Button2 Button2_KeyboardEquals;
        Button2 Button2_KeyboardGrave;
        Button2 Button2_KeyboardKana;
        Button2 Button2_KeyboardKanji;
        Button2 Button2_KeyboardAltLeft;
        Button2 Button2_KeyboardBracketLeft;
        Button2 Button2_KeyboardControlLeft;
        Button2 Button2_KeyboardShiftLeft;
        Button2 Button2_KeyboardSuperLeft;
        Button2 Button2_KeyboardMail;
        Button2 Button2_KeyboardMediaSelect;
        Button2 Button2_KeyboardMediaStop;
        Button2 Button2_KeyboardMinus;
        Button2 Button2_KeyboardNumpadMultiply;
        Button2 Button2_KeyboardMute;
        Button2 Button2_KeyboardMyComputer;
        Button2 Button2_KeyboardNavigateForward;
        Button2 Button2_KeyboardNavigateBackward;
        Button2 Button2_KeyboardNextTrack;
        Button2 Button2_KeyboardNoConvert;
        Button2 Button2_KeyboardNumpadComma;
        Button2 Button2_KeyboardNumpadEnter;
        Button2 Button2_KeyboardNumpadEquals;
        Button2 Button2_KeyboardOem102;
        Button2 Button2_KeyboardPeriod;
        Button2 Button2_KeyboardPlayPause;
        Button2 Button2_KeyboardPower;
        Button2 Button2_KeyboardPrevTrack;
        Button2 Button2_KeyboardAltRight;
        Button2 Button2_KeyboardBracketRight;
        Button2 Button2_KeyboardControlRight;
        Button2 Button2_KeyboardShiftRight;
        Button2 Button2_KeyboardSuperRight;
        Button2 Button2_KeyboardSemicolon;
        Button2 Button2_KeyboardSlash;
        Button2 Button2_KeyboardSleep;
        Button2 Button2_KeyboardStop;
        Button2 Button2_KeyboardNumpadSubtract;
        Button2 Button2_KeyboardSysrq;
        Button2 Button2_KeyboardTab;
        Button2 Button2_KeyboardUnderline;
        Button2 Button2_KeyboardUnlabeled;
        Button2 Button2_KeyboardVolumeDown;
        Button2 Button2_KeyboardVolumeUp;
        Button2 Button2_KeyboardWake;
        Button2 Button2_KeyboardWebBack;
        Button2 Button2_KeyboardWebFavorites;
        Button2 Button2_KeyboardWebForward;
        Button2 Button2_KeyboardWebHome;
        Button2 Button2_KeyboardWebRefresh;
        Button2 Button2_KeyboardWebSearch;
        Button2 Button2_KeyboardWebStop;
        Button2 Button2_KeyboardYen;
        Button2 Button2_KeyboardCopy;
        Button2 Button2_KeyboardPaste;
        Button2 Button2_KeyboardCut;
        Button2 Button2_MouseLeft;
        Button2 Button2_MouseMiddle;
        Button2 Button2_MouseRight;
        Button2 Button2_MouseX1;
        Button2 Button2_MouseX2;
        Button2 Button2_MouseDeltaX;
        Button2 Button2_MouseDeltaY;
        Button2 Button2_MouseScrollPixelX;
        Button2 Button2_MouseScrollPixelY;
        Button2 Button2_MouseScrollLineX;
        Button2 Button2_MouseScrollLineY;
        Button2 Button2_GamepadSouth;
        Button2 Button2_GamepadEast;
        Button2 Button2_GamepadNorth;
        Button2 Button2_GamepadWest;
        Button2 Button2_GamepadC;
        Button2 Button2_GamepadZ;
        Button2 Button2_GamepadLeftTrigger;
        Button2 Button2_GamepadLeftTrigger2;
        Button2 Button2_GamepadRightTrigger;
        Button2 Button2_GamepadRightTrigger2;
        Button2 Button2_GamepadSelect;
        Button2 Button2_GamepadStart;
        Button2 Button2_GamepadMode;
        Button2 Button2_GamepadLeftThumb;
        Button2 Button2_GamepadRightThumb;
        Button2 Button2_GamepadDPadUp;
        Button2 Button2_GamepadDPadDown;
        Button2 Button2_GamepadDPadLeft;
        Button2 Button2_GamepadDPadRight;
        Button2 Button2_GamepadLeftStickX;
        Button2 Button2_GamepadLeftStickY;
        Button2 Button2_GamepadLeftZ;
        Button2 Button2_GamepadRightStickX;
        Button2 Button2_GamepadRightStickY;
        Button2 Button2_GamepadRightZ;
        Button2 Button2_TouchpadX;
        Button2 Button2_TouchpadY;
        Button2 Button2_TouchpadMagnifyDelta;
        Button2 Button2_TouchpadRotateDelta;

        cstr    Button2_ToString(Button2);
    ]]
end

do -- Global Symbol Table
    Button2 = {
        KeyboardKey1             = libphx.Button2_KeyboardKey1,
        KeyboardKey2             = libphx.Button2_KeyboardKey2,
        KeyboardKey3             = libphx.Button2_KeyboardKey3,
        KeyboardKey4             = libphx.Button2_KeyboardKey4,
        KeyboardKey5             = libphx.Button2_KeyboardKey5,
        KeyboardKey6             = libphx.Button2_KeyboardKey6,
        KeyboardKey7             = libphx.Button2_KeyboardKey7,
        KeyboardKey8             = libphx.Button2_KeyboardKey8,
        KeyboardKey9             = libphx.Button2_KeyboardKey9,
        KeyboardKey0             = libphx.Button2_KeyboardKey0,
        KeyboardA                = libphx.Button2_KeyboardA,
        KeyboardB                = libphx.Button2_KeyboardB,
        KeyboardC                = libphx.Button2_KeyboardC,
        KeyboardD                = libphx.Button2_KeyboardD,
        KeyboardE                = libphx.Button2_KeyboardE,
        KeyboardF                = libphx.Button2_KeyboardF,
        KeyboardG                = libphx.Button2_KeyboardG,
        KeyboardH                = libphx.Button2_KeyboardH,
        KeyboardI                = libphx.Button2_KeyboardI,
        KeyboardJ                = libphx.Button2_KeyboardJ,
        KeyboardK                = libphx.Button2_KeyboardK,
        KeyboardL                = libphx.Button2_KeyboardL,
        KeyboardM                = libphx.Button2_KeyboardM,
        KeyboardN                = libphx.Button2_KeyboardN,
        KeyboardO                = libphx.Button2_KeyboardO,
        KeyboardP                = libphx.Button2_KeyboardP,
        KeyboardQ                = libphx.Button2_KeyboardQ,
        KeyboardR                = libphx.Button2_KeyboardR,
        KeyboardS                = libphx.Button2_KeyboardS,
        KeyboardT                = libphx.Button2_KeyboardT,
        KeyboardU                = libphx.Button2_KeyboardU,
        KeyboardV                = libphx.Button2_KeyboardV,
        KeyboardW                = libphx.Button2_KeyboardW,
        KeyboardX                = libphx.Button2_KeyboardX,
        KeyboardY                = libphx.Button2_KeyboardY,
        KeyboardZ                = libphx.Button2_KeyboardZ,
        KeyboardEscape           = libphx.Button2_KeyboardEscape,
        KeyboardF1               = libphx.Button2_KeyboardF1,
        KeyboardF2               = libphx.Button2_KeyboardF2,
        KeyboardF3               = libphx.Button2_KeyboardF3,
        KeyboardF4               = libphx.Button2_KeyboardF4,
        KeyboardF5               = libphx.Button2_KeyboardF5,
        KeyboardF6               = libphx.Button2_KeyboardF6,
        KeyboardF7               = libphx.Button2_KeyboardF7,
        KeyboardF8               = libphx.Button2_KeyboardF8,
        KeyboardF9               = libphx.Button2_KeyboardF9,
        KeyboardF10              = libphx.Button2_KeyboardF10,
        KeyboardF11              = libphx.Button2_KeyboardF11,
        KeyboardF12              = libphx.Button2_KeyboardF12,
        KeyboardF13              = libphx.Button2_KeyboardF13,
        KeyboardF14              = libphx.Button2_KeyboardF14,
        KeyboardF15              = libphx.Button2_KeyboardF15,
        KeyboardF16              = libphx.Button2_KeyboardF16,
        KeyboardF17              = libphx.Button2_KeyboardF17,
        KeyboardF18              = libphx.Button2_KeyboardF18,
        KeyboardF19              = libphx.Button2_KeyboardF19,
        KeyboardF20              = libphx.Button2_KeyboardF20,
        KeyboardF21              = libphx.Button2_KeyboardF21,
        KeyboardF22              = libphx.Button2_KeyboardF22,
        KeyboardF23              = libphx.Button2_KeyboardF23,
        KeyboardF24              = libphx.Button2_KeyboardF24,
        KeyboardSnapshot         = libphx.Button2_KeyboardSnapshot,
        KeyboardScroll           = libphx.Button2_KeyboardScroll,
        KeyboardPause            = libphx.Button2_KeyboardPause,
        KeyboardInsert           = libphx.Button2_KeyboardInsert,
        KeyboardHome             = libphx.Button2_KeyboardHome,
        KeyboardDelete           = libphx.Button2_KeyboardDelete,
        KeyboardEnd              = libphx.Button2_KeyboardEnd,
        KeyboardPageDown         = libphx.Button2_KeyboardPageDown,
        KeyboardPageUp           = libphx.Button2_KeyboardPageUp,
        KeyboardLeft             = libphx.Button2_KeyboardLeft,
        KeyboardUp               = libphx.Button2_KeyboardUp,
        KeyboardRight            = libphx.Button2_KeyboardRight,
        KeyboardDown             = libphx.Button2_KeyboardDown,
        KeyboardBack             = libphx.Button2_KeyboardBack,
        KeyboardReturn           = libphx.Button2_KeyboardReturn,
        KeyboardSpace            = libphx.Button2_KeyboardSpace,
        KeyboardCompose          = libphx.Button2_KeyboardCompose,
        KeyboardCaret            = libphx.Button2_KeyboardCaret,
        KeyboardNumlock          = libphx.Button2_KeyboardNumlock,
        KeyboardNumpad0          = libphx.Button2_KeyboardNumpad0,
        KeyboardNumpad1          = libphx.Button2_KeyboardNumpad1,
        KeyboardNumpad2          = libphx.Button2_KeyboardNumpad2,
        KeyboardNumpad3          = libphx.Button2_KeyboardNumpad3,
        KeyboardNumpad4          = libphx.Button2_KeyboardNumpad4,
        KeyboardNumpad5          = libphx.Button2_KeyboardNumpad5,
        KeyboardNumpad6          = libphx.Button2_KeyboardNumpad6,
        KeyboardNumpad7          = libphx.Button2_KeyboardNumpad7,
        KeyboardNumpad8          = libphx.Button2_KeyboardNumpad8,
        KeyboardNumpad9          = libphx.Button2_KeyboardNumpad9,
        KeyboardAbntC1           = libphx.Button2_KeyboardAbntC1,
        KeyboardAbntC2           = libphx.Button2_KeyboardAbntC2,
        KeyboardNumpadAdd        = libphx.Button2_KeyboardNumpadAdd,
        KeyboardApostrophe       = libphx.Button2_KeyboardApostrophe,
        KeyboardApps             = libphx.Button2_KeyboardApps,
        KeyboardAsterisk         = libphx.Button2_KeyboardAsterisk,
        KeyboardPlus             = libphx.Button2_KeyboardPlus,
        KeyboardAt               = libphx.Button2_KeyboardAt,
        KeyboardAx               = libphx.Button2_KeyboardAx,
        KeyboardBackslash        = libphx.Button2_KeyboardBackslash,
        KeyboardCalculator       = libphx.Button2_KeyboardCalculator,
        KeyboardCapital          = libphx.Button2_KeyboardCapital,
        KeyboardColon            = libphx.Button2_KeyboardColon,
        KeyboardComma            = libphx.Button2_KeyboardComma,
        KeyboardConvert          = libphx.Button2_KeyboardConvert,
        KeyboardNumpadDecimal    = libphx.Button2_KeyboardNumpadDecimal,
        KeyboardNumpadDivide     = libphx.Button2_KeyboardNumpadDivide,
        KeyboardEquals           = libphx.Button2_KeyboardEquals,
        KeyboardGrave            = libphx.Button2_KeyboardGrave,
        KeyboardKana             = libphx.Button2_KeyboardKana,
        KeyboardKanji            = libphx.Button2_KeyboardKanji,
        KeyboardAltLeft          = libphx.Button2_KeyboardAltLeft,
        KeyboardBracketLeft      = libphx.Button2_KeyboardBracketLeft,
        KeyboardControlLeft      = libphx.Button2_KeyboardControlLeft,
        KeyboardShiftLeft        = libphx.Button2_KeyboardShiftLeft,
        KeyboardSuperLeft        = libphx.Button2_KeyboardSuperLeft,
        KeyboardMail             = libphx.Button2_KeyboardMail,
        KeyboardMediaSelect      = libphx.Button2_KeyboardMediaSelect,
        KeyboardMediaStop        = libphx.Button2_KeyboardMediaStop,
        KeyboardMinus            = libphx.Button2_KeyboardMinus,
        KeyboardNumpadMultiply   = libphx.Button2_KeyboardNumpadMultiply,
        KeyboardMute             = libphx.Button2_KeyboardMute,
        KeyboardMyComputer       = libphx.Button2_KeyboardMyComputer,
        KeyboardNavigateForward  = libphx.Button2_KeyboardNavigateForward,
        KeyboardNavigateBackward = libphx.Button2_KeyboardNavigateBackward,
        KeyboardNextTrack        = libphx.Button2_KeyboardNextTrack,
        KeyboardNoConvert        = libphx.Button2_KeyboardNoConvert,
        KeyboardNumpadComma      = libphx.Button2_KeyboardNumpadComma,
        KeyboardNumpadEnter      = libphx.Button2_KeyboardNumpadEnter,
        KeyboardNumpadEquals     = libphx.Button2_KeyboardNumpadEquals,
        KeyboardOem102           = libphx.Button2_KeyboardOem102,
        KeyboardPeriod           = libphx.Button2_KeyboardPeriod,
        KeyboardPlayPause        = libphx.Button2_KeyboardPlayPause,
        KeyboardPower            = libphx.Button2_KeyboardPower,
        KeyboardPrevTrack        = libphx.Button2_KeyboardPrevTrack,
        KeyboardAltRight         = libphx.Button2_KeyboardAltRight,
        KeyboardBracketRight     = libphx.Button2_KeyboardBracketRight,
        KeyboardControlRight     = libphx.Button2_KeyboardControlRight,
        KeyboardShiftRight       = libphx.Button2_KeyboardShiftRight,
        KeyboardSuperRight       = libphx.Button2_KeyboardSuperRight,
        KeyboardSemicolon        = libphx.Button2_KeyboardSemicolon,
        KeyboardSlash            = libphx.Button2_KeyboardSlash,
        KeyboardSleep            = libphx.Button2_KeyboardSleep,
        KeyboardStop             = libphx.Button2_KeyboardStop,
        KeyboardNumpadSubtract   = libphx.Button2_KeyboardNumpadSubtract,
        KeyboardSysrq            = libphx.Button2_KeyboardSysrq,
        KeyboardTab              = libphx.Button2_KeyboardTab,
        KeyboardUnderline        = libphx.Button2_KeyboardUnderline,
        KeyboardUnlabeled        = libphx.Button2_KeyboardUnlabeled,
        KeyboardVolumeDown       = libphx.Button2_KeyboardVolumeDown,
        KeyboardVolumeUp         = libphx.Button2_KeyboardVolumeUp,
        KeyboardWake             = libphx.Button2_KeyboardWake,
        KeyboardWebBack          = libphx.Button2_KeyboardWebBack,
        KeyboardWebFavorites     = libphx.Button2_KeyboardWebFavorites,
        KeyboardWebForward       = libphx.Button2_KeyboardWebForward,
        KeyboardWebHome          = libphx.Button2_KeyboardWebHome,
        KeyboardWebRefresh       = libphx.Button2_KeyboardWebRefresh,
        KeyboardWebSearch        = libphx.Button2_KeyboardWebSearch,
        KeyboardWebStop          = libphx.Button2_KeyboardWebStop,
        KeyboardYen              = libphx.Button2_KeyboardYen,
        KeyboardCopy             = libphx.Button2_KeyboardCopy,
        KeyboardPaste            = libphx.Button2_KeyboardPaste,
        KeyboardCut              = libphx.Button2_KeyboardCut,
        MouseLeft                = libphx.Button2_MouseLeft,
        MouseMiddle              = libphx.Button2_MouseMiddle,
        MouseRight               = libphx.Button2_MouseRight,
        MouseX1                  = libphx.Button2_MouseX1,
        MouseX2                  = libphx.Button2_MouseX2,
        MouseDeltaX              = libphx.Button2_MouseDeltaX,
        MouseDeltaY              = libphx.Button2_MouseDeltaY,
        MouseScrollPixelX        = libphx.Button2_MouseScrollPixelX,
        MouseScrollPixelY        = libphx.Button2_MouseScrollPixelY,
        MouseScrollLineX         = libphx.Button2_MouseScrollLineX,
        MouseScrollLineY         = libphx.Button2_MouseScrollLineY,
        GamepadSouth             = libphx.Button2_GamepadSouth,
        GamepadEast              = libphx.Button2_GamepadEast,
        GamepadNorth             = libphx.Button2_GamepadNorth,
        GamepadWest              = libphx.Button2_GamepadWest,
        GamepadC                 = libphx.Button2_GamepadC,
        GamepadZ                 = libphx.Button2_GamepadZ,
        GamepadLeftTrigger       = libphx.Button2_GamepadLeftTrigger,
        GamepadLeftTrigger2      = libphx.Button2_GamepadLeftTrigger2,
        GamepadRightTrigger      = libphx.Button2_GamepadRightTrigger,
        GamepadRightTrigger2     = libphx.Button2_GamepadRightTrigger2,
        GamepadSelect            = libphx.Button2_GamepadSelect,
        GamepadStart             = libphx.Button2_GamepadStart,
        GamepadMode              = libphx.Button2_GamepadMode,
        GamepadLeftThumb         = libphx.Button2_GamepadLeftThumb,
        GamepadRightThumb        = libphx.Button2_GamepadRightThumb,
        GamepadDPadUp            = libphx.Button2_GamepadDPadUp,
        GamepadDPadDown          = libphx.Button2_GamepadDPadDown,
        GamepadDPadLeft          = libphx.Button2_GamepadDPadLeft,
        GamepadDPadRight         = libphx.Button2_GamepadDPadRight,
        GamepadLeftStickX        = libphx.Button2_GamepadLeftStickX,
        GamepadLeftStickY        = libphx.Button2_GamepadLeftStickY,
        GamepadLeftZ             = libphx.Button2_GamepadLeftZ,
        GamepadRightStickX       = libphx.Button2_GamepadRightStickX,
        GamepadRightStickY       = libphx.Button2_GamepadRightStickY,
        GamepadRightZ            = libphx.Button2_GamepadRightZ,
        TouchpadX                = libphx.Button2_TouchpadX,
        TouchpadY                = libphx.Button2_TouchpadY,
        TouchpadMagnifyDelta     = libphx.Button2_TouchpadMagnifyDelta,
        TouchpadRotateDelta      = libphx.Button2_TouchpadRotateDelta,

        ToString                 = libphx.Button2_ToString,
    }

    if onDef_Button2 then onDef_Button2(Button2, mt) end
    Button2 = setmetatable(Button2, mt)
end

return Button2
