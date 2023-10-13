-- Button ----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local Button

function declareType()
    ffi.cdef [[
        typedef uint8 Button;
    ]]

    return 2, 'Button'
end

do -- C Definitions
    ffi.cdef [[
        Button Button_KeyboardKey1;
        Button Button_KeyboardKey2;
        Button Button_KeyboardKey3;
        Button Button_KeyboardKey4;
        Button Button_KeyboardKey5;
        Button Button_KeyboardKey6;
        Button Button_KeyboardKey7;
        Button Button_KeyboardKey8;
        Button Button_KeyboardKey9;
        Button Button_KeyboardKey0;
        Button Button_KeyboardA;
        Button Button_KeyboardB;
        Button Button_KeyboardC;
        Button Button_KeyboardD;
        Button Button_KeyboardE;
        Button Button_KeyboardF;
        Button Button_KeyboardG;
        Button Button_KeyboardH;
        Button Button_KeyboardI;
        Button Button_KeyboardJ;
        Button Button_KeyboardK;
        Button Button_KeyboardL;
        Button Button_KeyboardM;
        Button Button_KeyboardN;
        Button Button_KeyboardO;
        Button Button_KeyboardP;
        Button Button_KeyboardQ;
        Button Button_KeyboardR;
        Button Button_KeyboardS;
        Button Button_KeyboardT;
        Button Button_KeyboardU;
        Button Button_KeyboardV;
        Button Button_KeyboardW;
        Button Button_KeyboardX;
        Button Button_KeyboardY;
        Button Button_KeyboardZ;
        Button Button_KeyboardEscape;
        Button Button_KeyboardF1;
        Button Button_KeyboardF2;
        Button Button_KeyboardF3;
        Button Button_KeyboardF4;
        Button Button_KeyboardF5;
        Button Button_KeyboardF6;
        Button Button_KeyboardF7;
        Button Button_KeyboardF8;
        Button Button_KeyboardF9;
        Button Button_KeyboardF10;
        Button Button_KeyboardF11;
        Button Button_KeyboardF12;
        Button Button_KeyboardF13;
        Button Button_KeyboardF14;
        Button Button_KeyboardF15;
        Button Button_KeyboardF16;
        Button Button_KeyboardF17;
        Button Button_KeyboardF18;
        Button Button_KeyboardF19;
        Button Button_KeyboardF20;
        Button Button_KeyboardF21;
        Button Button_KeyboardF22;
        Button Button_KeyboardF23;
        Button Button_KeyboardF24;
        Button Button_KeyboardSnapshot;
        Button Button_KeyboardScroll;
        Button Button_KeyboardPause;
        Button Button_KeyboardInsert;
        Button Button_KeyboardHome;
        Button Button_KeyboardDelete;
        Button Button_KeyboardEnd;
        Button Button_KeyboardPageDown;
        Button Button_KeyboardPageUp;
        Button Button_KeyboardLeft;
        Button Button_KeyboardUp;
        Button Button_KeyboardRight;
        Button Button_KeyboardDown;
        Button Button_KeyboardBack;
        Button Button_KeyboardReturn;
        Button Button_KeyboardSpace;
        Button Button_KeyboardCompose;
        Button Button_KeyboardCaret;
        Button Button_KeyboardNumlock;
        Button Button_KeyboardNumpad0;
        Button Button_KeyboardNumpad1;
        Button Button_KeyboardNumpad2;
        Button Button_KeyboardNumpad3;
        Button Button_KeyboardNumpad4;
        Button Button_KeyboardNumpad5;
        Button Button_KeyboardNumpad6;
        Button Button_KeyboardNumpad7;
        Button Button_KeyboardNumpad8;
        Button Button_KeyboardNumpad9;
        Button Button_KeyboardAbntC1;
        Button Button_KeyboardAbntC2;
        Button Button_KeyboardNumpadAdd;
        Button Button_KeyboardApostrophe;
        Button Button_KeyboardApps;
        Button Button_KeyboardAsterisk;
        Button Button_KeyboardPlus;
        Button Button_KeyboardAt;
        Button Button_KeyboardAx;
        Button Button_KeyboardBackslash;
        Button Button_KeyboardCalculator;
        Button Button_KeyboardCapital;
        Button Button_KeyboardColon;
        Button Button_KeyboardComma;
        Button Button_KeyboardConvert;
        Button Button_KeyboardNumpadDecimal;
        Button Button_KeyboardNumpadDivide;
        Button Button_KeyboardEquals;
        Button Button_KeyboardGrave;
        Button Button_KeyboardKana;
        Button Button_KeyboardKanji;
        Button Button_KeyboardAltLeft;
        Button Button_KeyboardBracketLeft;
        Button Button_KeyboardControlLeft;
        Button Button_KeyboardShiftLeft;
        Button Button_KeyboardSuperLeft;
        Button Button_KeyboardMail;
        Button Button_KeyboardMediaSelect;
        Button Button_KeyboardMediaStop;
        Button Button_KeyboardMinus;
        Button Button_KeyboardNumpadMultiply;
        Button Button_KeyboardMute;
        Button Button_KeyboardMyComputer;
        Button Button_KeyboardNavigateForward;
        Button Button_KeyboardNavigateBackward;
        Button Button_KeyboardNextTrack;
        Button Button_KeyboardNoConvert;
        Button Button_KeyboardNumpadComma;
        Button Button_KeyboardNumpadEnter;
        Button Button_KeyboardNumpadEquals;
        Button Button_KeyboardOem102;
        Button Button_KeyboardPeriod;
        Button Button_KeyboardPlayPause;
        Button Button_KeyboardPower;
        Button Button_KeyboardPrevTrack;
        Button Button_KeyboardAltRight;
        Button Button_KeyboardBracketRight;
        Button Button_KeyboardControlRight;
        Button Button_KeyboardShiftRight;
        Button Button_KeyboardSuperRight;
        Button Button_KeyboardSemicolon;
        Button Button_KeyboardSlash;
        Button Button_KeyboardSleep;
        Button Button_KeyboardStop;
        Button Button_KeyboardNumpadSubtract;
        Button Button_KeyboardSysrq;
        Button Button_KeyboardTab;
        Button Button_KeyboardUnderline;
        Button Button_KeyboardUnlabeled;
        Button Button_KeyboardVolumeDown;
        Button Button_KeyboardVolumeUp;
        Button Button_KeyboardWake;
        Button Button_KeyboardWebBack;
        Button Button_KeyboardWebFavorites;
        Button Button_KeyboardWebForward;
        Button Button_KeyboardWebHome;
        Button Button_KeyboardWebRefresh;
        Button Button_KeyboardWebSearch;
        Button Button_KeyboardWebStop;
        Button Button_KeyboardYen;
        Button Button_KeyboardCopy;
        Button Button_KeyboardPaste;
        Button Button_KeyboardCut;
        Button Button_MouseLeft;
        Button Button_MouseMiddle;
        Button Button_MouseRight;
        Button Button_MouseX1;
        Button Button_MouseX2;
        Button Button_MouseDeltaX;
        Button Button_MouseDeltaY;
        Button Button_MouseScrollX;
        Button Button_MouseScrollY;
        Button Button_MouseScrollPixelX;
        Button Button_MouseScrollPixelY;
        Button Button_GamepadSouth;
        Button Button_GamepadEast;
        Button Button_GamepadNorth;
        Button Button_GamepadWest;
        Button Button_GamepadC;
        Button Button_GamepadZ;
        Button Button_GamepadLeftTrigger;
        Button Button_GamepadLeftTrigger2;
        Button Button_GamepadRightTrigger;
        Button Button_GamepadRightTrigger2;
        Button Button_GamepadSelect;
        Button Button_GamepadStart;
        Button Button_GamepadMode;
        Button Button_GamepadLeftThumb;
        Button Button_GamepadRightThumb;
        Button Button_GamepadDPadUp;
        Button Button_GamepadDPadDown;
        Button Button_GamepadDPadLeft;
        Button Button_GamepadDPadRight;
        Button Button_GamepadLeftStickX;
        Button Button_GamepadLeftStickY;
        Button Button_GamepadLeftZ;
        Button Button_GamepadRightStickX;
        Button Button_GamepadRightStickY;
        Button Button_GamepadRightZ;
        Button Button_TouchpadX;
        Button Button_TouchpadY;
        Button Button_TouchpadMagnifyDelta;
        Button Button_TouchpadRotateDelta;
        Button Button_SystemExit;

        cstr   Button_ToString(Button);
    ]]
end

do -- Global Symbol Table
    Button = {
        KeyboardKey1             = libphx.Button_KeyboardKey1,
        KeyboardKey2             = libphx.Button_KeyboardKey2,
        KeyboardKey3             = libphx.Button_KeyboardKey3,
        KeyboardKey4             = libphx.Button_KeyboardKey4,
        KeyboardKey5             = libphx.Button_KeyboardKey5,
        KeyboardKey6             = libphx.Button_KeyboardKey6,
        KeyboardKey7             = libphx.Button_KeyboardKey7,
        KeyboardKey8             = libphx.Button_KeyboardKey8,
        KeyboardKey9             = libphx.Button_KeyboardKey9,
        KeyboardKey0             = libphx.Button_KeyboardKey0,
        KeyboardA                = libphx.Button_KeyboardA,
        KeyboardB                = libphx.Button_KeyboardB,
        KeyboardC                = libphx.Button_KeyboardC,
        KeyboardD                = libphx.Button_KeyboardD,
        KeyboardE                = libphx.Button_KeyboardE,
        KeyboardF                = libphx.Button_KeyboardF,
        KeyboardG                = libphx.Button_KeyboardG,
        KeyboardH                = libphx.Button_KeyboardH,
        KeyboardI                = libphx.Button_KeyboardI,
        KeyboardJ                = libphx.Button_KeyboardJ,
        KeyboardK                = libphx.Button_KeyboardK,
        KeyboardL                = libphx.Button_KeyboardL,
        KeyboardM                = libphx.Button_KeyboardM,
        KeyboardN                = libphx.Button_KeyboardN,
        KeyboardO                = libphx.Button_KeyboardO,
        KeyboardP                = libphx.Button_KeyboardP,
        KeyboardQ                = libphx.Button_KeyboardQ,
        KeyboardR                = libphx.Button_KeyboardR,
        KeyboardS                = libphx.Button_KeyboardS,
        KeyboardT                = libphx.Button_KeyboardT,
        KeyboardU                = libphx.Button_KeyboardU,
        KeyboardV                = libphx.Button_KeyboardV,
        KeyboardW                = libphx.Button_KeyboardW,
        KeyboardX                = libphx.Button_KeyboardX,
        KeyboardY                = libphx.Button_KeyboardY,
        KeyboardZ                = libphx.Button_KeyboardZ,
        KeyboardEscape           = libphx.Button_KeyboardEscape,
        KeyboardF1               = libphx.Button_KeyboardF1,
        KeyboardF2               = libphx.Button_KeyboardF2,
        KeyboardF3               = libphx.Button_KeyboardF3,
        KeyboardF4               = libphx.Button_KeyboardF4,
        KeyboardF5               = libphx.Button_KeyboardF5,
        KeyboardF6               = libphx.Button_KeyboardF6,
        KeyboardF7               = libphx.Button_KeyboardF7,
        KeyboardF8               = libphx.Button_KeyboardF8,
        KeyboardF9               = libphx.Button_KeyboardF9,
        KeyboardF10              = libphx.Button_KeyboardF10,
        KeyboardF11              = libphx.Button_KeyboardF11,
        KeyboardF12              = libphx.Button_KeyboardF12,
        KeyboardF13              = libphx.Button_KeyboardF13,
        KeyboardF14              = libphx.Button_KeyboardF14,
        KeyboardF15              = libphx.Button_KeyboardF15,
        KeyboardF16              = libphx.Button_KeyboardF16,
        KeyboardF17              = libphx.Button_KeyboardF17,
        KeyboardF18              = libphx.Button_KeyboardF18,
        KeyboardF19              = libphx.Button_KeyboardF19,
        KeyboardF20              = libphx.Button_KeyboardF20,
        KeyboardF21              = libphx.Button_KeyboardF21,
        KeyboardF22              = libphx.Button_KeyboardF22,
        KeyboardF23              = libphx.Button_KeyboardF23,
        KeyboardF24              = libphx.Button_KeyboardF24,
        KeyboardSnapshot         = libphx.Button_KeyboardSnapshot,
        KeyboardScroll           = libphx.Button_KeyboardScroll,
        KeyboardPause            = libphx.Button_KeyboardPause,
        KeyboardInsert           = libphx.Button_KeyboardInsert,
        KeyboardHome             = libphx.Button_KeyboardHome,
        KeyboardDelete           = libphx.Button_KeyboardDelete,
        KeyboardEnd              = libphx.Button_KeyboardEnd,
        KeyboardPageDown         = libphx.Button_KeyboardPageDown,
        KeyboardPageUp           = libphx.Button_KeyboardPageUp,
        KeyboardLeft             = libphx.Button_KeyboardLeft,
        KeyboardUp               = libphx.Button_KeyboardUp,
        KeyboardRight            = libphx.Button_KeyboardRight,
        KeyboardDown             = libphx.Button_KeyboardDown,
        KeyboardBack             = libphx.Button_KeyboardBack,
        KeyboardReturn           = libphx.Button_KeyboardReturn,
        KeyboardSpace            = libphx.Button_KeyboardSpace,
        KeyboardCompose          = libphx.Button_KeyboardCompose,
        KeyboardCaret            = libphx.Button_KeyboardCaret,
        KeyboardNumlock          = libphx.Button_KeyboardNumlock,
        KeyboardNumpad0          = libphx.Button_KeyboardNumpad0,
        KeyboardNumpad1          = libphx.Button_KeyboardNumpad1,
        KeyboardNumpad2          = libphx.Button_KeyboardNumpad2,
        KeyboardNumpad3          = libphx.Button_KeyboardNumpad3,
        KeyboardNumpad4          = libphx.Button_KeyboardNumpad4,
        KeyboardNumpad5          = libphx.Button_KeyboardNumpad5,
        KeyboardNumpad6          = libphx.Button_KeyboardNumpad6,
        KeyboardNumpad7          = libphx.Button_KeyboardNumpad7,
        KeyboardNumpad8          = libphx.Button_KeyboardNumpad8,
        KeyboardNumpad9          = libphx.Button_KeyboardNumpad9,
        KeyboardAbntC1           = libphx.Button_KeyboardAbntC1,
        KeyboardAbntC2           = libphx.Button_KeyboardAbntC2,
        KeyboardNumpadAdd        = libphx.Button_KeyboardNumpadAdd,
        KeyboardApostrophe       = libphx.Button_KeyboardApostrophe,
        KeyboardApps             = libphx.Button_KeyboardApps,
        KeyboardAsterisk         = libphx.Button_KeyboardAsterisk,
        KeyboardPlus             = libphx.Button_KeyboardPlus,
        KeyboardAt               = libphx.Button_KeyboardAt,
        KeyboardAx               = libphx.Button_KeyboardAx,
        KeyboardBackslash        = libphx.Button_KeyboardBackslash,
        KeyboardCalculator       = libphx.Button_KeyboardCalculator,
        KeyboardCapital          = libphx.Button_KeyboardCapital,
        KeyboardColon            = libphx.Button_KeyboardColon,
        KeyboardComma            = libphx.Button_KeyboardComma,
        KeyboardConvert          = libphx.Button_KeyboardConvert,
        KeyboardNumpadDecimal    = libphx.Button_KeyboardNumpadDecimal,
        KeyboardNumpadDivide     = libphx.Button_KeyboardNumpadDivide,
        KeyboardEquals           = libphx.Button_KeyboardEquals,
        KeyboardGrave            = libphx.Button_KeyboardGrave,
        KeyboardKana             = libphx.Button_KeyboardKana,
        KeyboardKanji            = libphx.Button_KeyboardKanji,
        KeyboardAltLeft          = libphx.Button_KeyboardAltLeft,
        KeyboardBracketLeft      = libphx.Button_KeyboardBracketLeft,
        KeyboardControlLeft      = libphx.Button_KeyboardControlLeft,
        KeyboardShiftLeft        = libphx.Button_KeyboardShiftLeft,
        KeyboardSuperLeft        = libphx.Button_KeyboardSuperLeft,
        KeyboardMail             = libphx.Button_KeyboardMail,
        KeyboardMediaSelect      = libphx.Button_KeyboardMediaSelect,
        KeyboardMediaStop        = libphx.Button_KeyboardMediaStop,
        KeyboardMinus            = libphx.Button_KeyboardMinus,
        KeyboardNumpadMultiply   = libphx.Button_KeyboardNumpadMultiply,
        KeyboardMute             = libphx.Button_KeyboardMute,
        KeyboardMyComputer       = libphx.Button_KeyboardMyComputer,
        KeyboardNavigateForward  = libphx.Button_KeyboardNavigateForward,
        KeyboardNavigateBackward = libphx.Button_KeyboardNavigateBackward,
        KeyboardNextTrack        = libphx.Button_KeyboardNextTrack,
        KeyboardNoConvert        = libphx.Button_KeyboardNoConvert,
        KeyboardNumpadComma      = libphx.Button_KeyboardNumpadComma,
        KeyboardNumpadEnter      = libphx.Button_KeyboardNumpadEnter,
        KeyboardNumpadEquals     = libphx.Button_KeyboardNumpadEquals,
        KeyboardOem102           = libphx.Button_KeyboardOem102,
        KeyboardPeriod           = libphx.Button_KeyboardPeriod,
        KeyboardPlayPause        = libphx.Button_KeyboardPlayPause,
        KeyboardPower            = libphx.Button_KeyboardPower,
        KeyboardPrevTrack        = libphx.Button_KeyboardPrevTrack,
        KeyboardAltRight         = libphx.Button_KeyboardAltRight,
        KeyboardBracketRight     = libphx.Button_KeyboardBracketRight,
        KeyboardControlRight     = libphx.Button_KeyboardControlRight,
        KeyboardShiftRight       = libphx.Button_KeyboardShiftRight,
        KeyboardSuperRight       = libphx.Button_KeyboardSuperRight,
        KeyboardSemicolon        = libphx.Button_KeyboardSemicolon,
        KeyboardSlash            = libphx.Button_KeyboardSlash,
        KeyboardSleep            = libphx.Button_KeyboardSleep,
        KeyboardStop             = libphx.Button_KeyboardStop,
        KeyboardNumpadSubtract   = libphx.Button_KeyboardNumpadSubtract,
        KeyboardSysrq            = libphx.Button_KeyboardSysrq,
        KeyboardTab              = libphx.Button_KeyboardTab,
        KeyboardUnderline        = libphx.Button_KeyboardUnderline,
        KeyboardUnlabeled        = libphx.Button_KeyboardUnlabeled,
        KeyboardVolumeDown       = libphx.Button_KeyboardVolumeDown,
        KeyboardVolumeUp         = libphx.Button_KeyboardVolumeUp,
        KeyboardWake             = libphx.Button_KeyboardWake,
        KeyboardWebBack          = libphx.Button_KeyboardWebBack,
        KeyboardWebFavorites     = libphx.Button_KeyboardWebFavorites,
        KeyboardWebForward       = libphx.Button_KeyboardWebForward,
        KeyboardWebHome          = libphx.Button_KeyboardWebHome,
        KeyboardWebRefresh       = libphx.Button_KeyboardWebRefresh,
        KeyboardWebSearch        = libphx.Button_KeyboardWebSearch,
        KeyboardWebStop          = libphx.Button_KeyboardWebStop,
        KeyboardYen              = libphx.Button_KeyboardYen,
        KeyboardCopy             = libphx.Button_KeyboardCopy,
        KeyboardPaste            = libphx.Button_KeyboardPaste,
        KeyboardCut              = libphx.Button_KeyboardCut,
        MouseLeft                = libphx.Button_MouseLeft,
        MouseMiddle              = libphx.Button_MouseMiddle,
        MouseRight               = libphx.Button_MouseRight,
        MouseX1                  = libphx.Button_MouseX1,
        MouseX2                  = libphx.Button_MouseX2,
        MouseDeltaX              = libphx.Button_MouseDeltaX,
        MouseDeltaY              = libphx.Button_MouseDeltaY,
        MouseScrollX             = libphx.Button_MouseScrollX,
        MouseScrollY             = libphx.Button_MouseScrollY,
        MouseScrollPixelX        = libphx.Button_MouseScrollPixelX,
        MouseScrollPixelY        = libphx.Button_MouseScrollPixelY,
        GamepadSouth             = libphx.Button_GamepadSouth,
        GamepadEast              = libphx.Button_GamepadEast,
        GamepadNorth             = libphx.Button_GamepadNorth,
        GamepadWest              = libphx.Button_GamepadWest,
        GamepadC                 = libphx.Button_GamepadC,
        GamepadZ                 = libphx.Button_GamepadZ,
        GamepadLeftTrigger       = libphx.Button_GamepadLeftTrigger,
        GamepadLeftTrigger2      = libphx.Button_GamepadLeftTrigger2,
        GamepadRightTrigger      = libphx.Button_GamepadRightTrigger,
        GamepadRightTrigger2     = libphx.Button_GamepadRightTrigger2,
        GamepadSelect            = libphx.Button_GamepadSelect,
        GamepadStart             = libphx.Button_GamepadStart,
        GamepadMode              = libphx.Button_GamepadMode,
        GamepadLeftThumb         = libphx.Button_GamepadLeftThumb,
        GamepadRightThumb        = libphx.Button_GamepadRightThumb,
        GamepadDPadUp            = libphx.Button_GamepadDPadUp,
        GamepadDPadDown          = libphx.Button_GamepadDPadDown,
        GamepadDPadLeft          = libphx.Button_GamepadDPadLeft,
        GamepadDPadRight         = libphx.Button_GamepadDPadRight,
        GamepadLeftStickX        = libphx.Button_GamepadLeftStickX,
        GamepadLeftStickY        = libphx.Button_GamepadLeftStickY,
        GamepadLeftZ             = libphx.Button_GamepadLeftZ,
        GamepadRightStickX       = libphx.Button_GamepadRightStickX,
        GamepadRightStickY       = libphx.Button_GamepadRightStickY,
        GamepadRightZ            = libphx.Button_GamepadRightZ,
        TouchpadX                = libphx.Button_TouchpadX,
        TouchpadY                = libphx.Button_TouchpadY,
        TouchpadMagnifyDelta     = libphx.Button_TouchpadMagnifyDelta,
        TouchpadRotateDelta      = libphx.Button_TouchpadRotateDelta,
        SystemExit               = libphx.Button_SystemExit,

        ToString                 = libphx.Button_ToString,
    }

    if onDef_Button then onDef_Button(Button, mt) end
    Button = setmetatable(Button, mt)
end

return Button
