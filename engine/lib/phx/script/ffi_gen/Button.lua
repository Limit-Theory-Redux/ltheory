-- Button ----------------------------------------------------------------------


local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 Button;
    ]]

    return 2, 'Button'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Button

    do -- C Definitions
        ffi.cdef [[
            Button Button_KeyboardBackquote;
            Button Button_KeyboardBackslash;
            Button Button_KeyboardBracketLeft;
            Button Button_KeyboardBracketRight;
            Button Button_KeyboardComma;
            Button Button_KeyboardKey0;
            Button Button_KeyboardKey1;
            Button Button_KeyboardKey2;
            Button Button_KeyboardKey3;
            Button Button_KeyboardKey4;
            Button Button_KeyboardKey5;
            Button Button_KeyboardKey6;
            Button Button_KeyboardKey7;
            Button Button_KeyboardKey8;
            Button Button_KeyboardKey9;
            Button Button_KeyboardEqual;
            Button Button_KeyboardIntlBackslash;
            Button Button_KeyboardIntlRo;
            Button Button_KeyboardIntlYen;
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
            Button Button_KeyboardMinus;
            Button Button_KeyboardPeriod;
            Button Button_KeyboardQuote;
            Button Button_KeyboardSemicolon;
            Button Button_KeyboardSlash;
            Button Button_KeyboardAltLeft;
            Button Button_KeyboardAltRight;
            Button Button_KeyboardBackspace;
            Button Button_KeyboardCapsLock;
            Button Button_KeyboardContextMenu;
            Button Button_KeyboardControlLeft;
            Button Button_KeyboardControlRight;
            Button Button_KeyboardEnter;
            Button Button_KeyboardSuperLeft;
            Button Button_KeyboardSuperRight;
            Button Button_KeyboardShiftLeft;
            Button Button_KeyboardShiftRight;
            Button Button_KeyboardSpace;
            Button Button_KeyboardTab;
            Button Button_KeyboardConvert;
            Button Button_KeyboardKanaMode;
            Button Button_KeyboardLang1;
            Button Button_KeyboardLang2;
            Button Button_KeyboardLang3;
            Button Button_KeyboardLang4;
            Button Button_KeyboardLang5;
            Button Button_KeyboardNonConvert;
            Button Button_KeyboardDelete;
            Button Button_KeyboardEnd;
            Button Button_KeyboardHelp;
            Button Button_KeyboardHome;
            Button Button_KeyboardInsert;
            Button Button_KeyboardPageDown;
            Button Button_KeyboardPageUp;
            Button Button_KeyboardDown;
            Button Button_KeyboardLeft;
            Button Button_KeyboardRight;
            Button Button_KeyboardUp;
            Button Button_KeyboardNumLock;
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
            Button Button_KeyboardNumpadAdd;
            Button Button_KeyboardNumpadBackspace;
            Button Button_KeyboardNumpadClear;
            Button Button_KeyboardNumpadClearEntry;
            Button Button_KeyboardNumpadComma;
            Button Button_KeyboardNumpadDecimal;
            Button Button_KeyboardNumpadDivide;
            Button Button_KeyboardNumpadEnter;
            Button Button_KeyboardNumpadEqual;
            Button Button_KeyboardNumpadHash;
            Button Button_KeyboardNumpadMemoryAdd;
            Button Button_KeyboardNumpadMemoryClear;
            Button Button_KeyboardNumpadMemoryRecall;
            Button Button_KeyboardNumpadMemoryStore;
            Button Button_KeyboardNumpadMemorySubtract;
            Button Button_KeyboardNumpadMultiply;
            Button Button_KeyboardNumpadParenLeft;
            Button Button_KeyboardNumpadParenRight;
            Button Button_KeyboardNumpadStar;
            Button Button_KeyboardNumpadSubtract;
            Button Button_KeyboardEscape;
            Button Button_KeyboardFn;
            Button Button_KeyboardFnLock;
            Button Button_KeyboardPrintScreen;
            Button Button_KeyboardScrollLock;
            Button Button_KeyboardPause;
            Button Button_KeyboardBrowserBack;
            Button Button_KeyboardBrowserFavorites;
            Button Button_KeyboardBrowserForward;
            Button Button_KeyboardBrowserHome;
            Button Button_KeyboardBrowserRefresh;
            Button Button_KeyboardBrowserSearch;
            Button Button_KeyboardBrowserStop;
            Button Button_KeyboardEject;
            Button Button_KeyboardLaunchApp1;
            Button Button_KeyboardLaunchApp2;
            Button Button_KeyboardLaunchMail;
            Button Button_KeyboardMediaPlayPause;
            Button Button_KeyboardMediaSelect;
            Button Button_KeyboardMediaStop;
            Button Button_KeyboardMediaTrackNext;
            Button Button_KeyboardMediaTrackPrevious;
            Button Button_KeyboardPower;
            Button Button_KeyboardSleep;
            Button Button_KeyboardAudioVolumeDown;
            Button Button_KeyboardAudioVolumeMute;
            Button Button_KeyboardAudioVolumeUp;
            Button Button_KeyboardWakeUp;
            Button Button_KeyboardMeta;
            Button Button_KeyboardHyper;
            Button Button_KeyboardTurbo;
            Button Button_KeyboardAbort;
            Button Button_KeyboardResume;
            Button Button_KeyboardSuspend;
            Button Button_KeyboardAgain;
            Button Button_KeyboardCopy;
            Button Button_KeyboardCut;
            Button Button_KeyboardFind;
            Button Button_KeyboardOpen;
            Button Button_KeyboardPaste;
            Button Button_KeyboardProps;
            Button Button_KeyboardSelect;
            Button Button_KeyboardUndo;
            Button Button_KeyboardHiragana;
            Button Button_KeyboardKatakana;
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
            Button Button_KeyboardF25;
            Button Button_KeyboardF26;
            Button Button_KeyboardF27;
            Button Button_KeyboardF28;
            Button Button_KeyboardF29;
            Button Button_KeyboardF30;
            Button Button_KeyboardF31;
            Button Button_KeyboardF32;
            Button Button_KeyboardF33;
            Button Button_KeyboardF34;
            Button Button_KeyboardF35;
            Button Button_MouseLeft;
            Button Button_MouseMiddle;
            Button Button_MouseRight;
            Button Button_MouseForward;
            Button Button_MouseBack;
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
            KeyboardBackquote            = libphx.Button_KeyboardBackquote,
            KeyboardBackslash            = libphx.Button_KeyboardBackslash,
            KeyboardBracketLeft          = libphx.Button_KeyboardBracketLeft,
            KeyboardBracketRight         = libphx.Button_KeyboardBracketRight,
            KeyboardComma                = libphx.Button_KeyboardComma,
            KeyboardKey0                 = libphx.Button_KeyboardKey0,
            KeyboardKey1                 = libphx.Button_KeyboardKey1,
            KeyboardKey2                 = libphx.Button_KeyboardKey2,
            KeyboardKey3                 = libphx.Button_KeyboardKey3,
            KeyboardKey4                 = libphx.Button_KeyboardKey4,
            KeyboardKey5                 = libphx.Button_KeyboardKey5,
            KeyboardKey6                 = libphx.Button_KeyboardKey6,
            KeyboardKey7                 = libphx.Button_KeyboardKey7,
            KeyboardKey8                 = libphx.Button_KeyboardKey8,
            KeyboardKey9                 = libphx.Button_KeyboardKey9,
            KeyboardEqual                = libphx.Button_KeyboardEqual,
            KeyboardIntlBackslash        = libphx.Button_KeyboardIntlBackslash,
            KeyboardIntlRo               = libphx.Button_KeyboardIntlRo,
            KeyboardIntlYen              = libphx.Button_KeyboardIntlYen,
            KeyboardA                    = libphx.Button_KeyboardA,
            KeyboardB                    = libphx.Button_KeyboardB,
            KeyboardC                    = libphx.Button_KeyboardC,
            KeyboardD                    = libphx.Button_KeyboardD,
            KeyboardE                    = libphx.Button_KeyboardE,
            KeyboardF                    = libphx.Button_KeyboardF,
            KeyboardG                    = libphx.Button_KeyboardG,
            KeyboardH                    = libphx.Button_KeyboardH,
            KeyboardI                    = libphx.Button_KeyboardI,
            KeyboardJ                    = libphx.Button_KeyboardJ,
            KeyboardK                    = libphx.Button_KeyboardK,
            KeyboardL                    = libphx.Button_KeyboardL,
            KeyboardM                    = libphx.Button_KeyboardM,
            KeyboardN                    = libphx.Button_KeyboardN,
            KeyboardO                    = libphx.Button_KeyboardO,
            KeyboardP                    = libphx.Button_KeyboardP,
            KeyboardQ                    = libphx.Button_KeyboardQ,
            KeyboardR                    = libphx.Button_KeyboardR,
            KeyboardS                    = libphx.Button_KeyboardS,
            KeyboardT                    = libphx.Button_KeyboardT,
            KeyboardU                    = libphx.Button_KeyboardU,
            KeyboardV                    = libphx.Button_KeyboardV,
            KeyboardW                    = libphx.Button_KeyboardW,
            KeyboardX                    = libphx.Button_KeyboardX,
            KeyboardY                    = libphx.Button_KeyboardY,
            KeyboardZ                    = libphx.Button_KeyboardZ,
            KeyboardMinus                = libphx.Button_KeyboardMinus,
            KeyboardPeriod               = libphx.Button_KeyboardPeriod,
            KeyboardQuote                = libphx.Button_KeyboardQuote,
            KeyboardSemicolon            = libphx.Button_KeyboardSemicolon,
            KeyboardSlash                = libphx.Button_KeyboardSlash,
            KeyboardAltLeft              = libphx.Button_KeyboardAltLeft,
            KeyboardAltRight             = libphx.Button_KeyboardAltRight,
            KeyboardBackspace            = libphx.Button_KeyboardBackspace,
            KeyboardCapsLock             = libphx.Button_KeyboardCapsLock,
            KeyboardContextMenu          = libphx.Button_KeyboardContextMenu,
            KeyboardControlLeft          = libphx.Button_KeyboardControlLeft,
            KeyboardControlRight         = libphx.Button_KeyboardControlRight,
            KeyboardEnter                = libphx.Button_KeyboardEnter,
            KeyboardSuperLeft            = libphx.Button_KeyboardSuperLeft,
            KeyboardSuperRight           = libphx.Button_KeyboardSuperRight,
            KeyboardShiftLeft            = libphx.Button_KeyboardShiftLeft,
            KeyboardShiftRight           = libphx.Button_KeyboardShiftRight,
            KeyboardSpace                = libphx.Button_KeyboardSpace,
            KeyboardTab                  = libphx.Button_KeyboardTab,
            KeyboardConvert              = libphx.Button_KeyboardConvert,
            KeyboardKanaMode             = libphx.Button_KeyboardKanaMode,
            KeyboardLang1                = libphx.Button_KeyboardLang1,
            KeyboardLang2                = libphx.Button_KeyboardLang2,
            KeyboardLang3                = libphx.Button_KeyboardLang3,
            KeyboardLang4                = libphx.Button_KeyboardLang4,
            KeyboardLang5                = libphx.Button_KeyboardLang5,
            KeyboardNonConvert           = libphx.Button_KeyboardNonConvert,
            KeyboardDelete               = libphx.Button_KeyboardDelete,
            KeyboardEnd                  = libphx.Button_KeyboardEnd,
            KeyboardHelp                 = libphx.Button_KeyboardHelp,
            KeyboardHome                 = libphx.Button_KeyboardHome,
            KeyboardInsert               = libphx.Button_KeyboardInsert,
            KeyboardPageDown             = libphx.Button_KeyboardPageDown,
            KeyboardPageUp               = libphx.Button_KeyboardPageUp,
            KeyboardDown                 = libphx.Button_KeyboardDown,
            KeyboardLeft                 = libphx.Button_KeyboardLeft,
            KeyboardRight                = libphx.Button_KeyboardRight,
            KeyboardUp                   = libphx.Button_KeyboardUp,
            KeyboardNumLock              = libphx.Button_KeyboardNumLock,
            KeyboardNumpad0              = libphx.Button_KeyboardNumpad0,
            KeyboardNumpad1              = libphx.Button_KeyboardNumpad1,
            KeyboardNumpad2              = libphx.Button_KeyboardNumpad2,
            KeyboardNumpad3              = libphx.Button_KeyboardNumpad3,
            KeyboardNumpad4              = libphx.Button_KeyboardNumpad4,
            KeyboardNumpad5              = libphx.Button_KeyboardNumpad5,
            KeyboardNumpad6              = libphx.Button_KeyboardNumpad6,
            KeyboardNumpad7              = libphx.Button_KeyboardNumpad7,
            KeyboardNumpad8              = libphx.Button_KeyboardNumpad8,
            KeyboardNumpad9              = libphx.Button_KeyboardNumpad9,
            KeyboardNumpadAdd            = libphx.Button_KeyboardNumpadAdd,
            KeyboardNumpadBackspace      = libphx.Button_KeyboardNumpadBackspace,
            KeyboardNumpadClear          = libphx.Button_KeyboardNumpadClear,
            KeyboardNumpadClearEntry     = libphx.Button_KeyboardNumpadClearEntry,
            KeyboardNumpadComma          = libphx.Button_KeyboardNumpadComma,
            KeyboardNumpadDecimal        = libphx.Button_KeyboardNumpadDecimal,
            KeyboardNumpadDivide         = libphx.Button_KeyboardNumpadDivide,
            KeyboardNumpadEnter          = libphx.Button_KeyboardNumpadEnter,
            KeyboardNumpadEqual          = libphx.Button_KeyboardNumpadEqual,
            KeyboardNumpadHash           = libphx.Button_KeyboardNumpadHash,
            KeyboardNumpadMemoryAdd      = libphx.Button_KeyboardNumpadMemoryAdd,
            KeyboardNumpadMemoryClear    = libphx.Button_KeyboardNumpadMemoryClear,
            KeyboardNumpadMemoryRecall   = libphx.Button_KeyboardNumpadMemoryRecall,
            KeyboardNumpadMemoryStore    = libphx.Button_KeyboardNumpadMemoryStore,
            KeyboardNumpadMemorySubtract = libphx.Button_KeyboardNumpadMemorySubtract,
            KeyboardNumpadMultiply       = libphx.Button_KeyboardNumpadMultiply,
            KeyboardNumpadParenLeft      = libphx.Button_KeyboardNumpadParenLeft,
            KeyboardNumpadParenRight     = libphx.Button_KeyboardNumpadParenRight,
            KeyboardNumpadStar           = libphx.Button_KeyboardNumpadStar,
            KeyboardNumpadSubtract       = libphx.Button_KeyboardNumpadSubtract,
            KeyboardEscape               = libphx.Button_KeyboardEscape,
            KeyboardFn                   = libphx.Button_KeyboardFn,
            KeyboardFnLock               = libphx.Button_KeyboardFnLock,
            KeyboardPrintScreen          = libphx.Button_KeyboardPrintScreen,
            KeyboardScrollLock           = libphx.Button_KeyboardScrollLock,
            KeyboardPause                = libphx.Button_KeyboardPause,
            KeyboardBrowserBack          = libphx.Button_KeyboardBrowserBack,
            KeyboardBrowserFavorites     = libphx.Button_KeyboardBrowserFavorites,
            KeyboardBrowserForward       = libphx.Button_KeyboardBrowserForward,
            KeyboardBrowserHome          = libphx.Button_KeyboardBrowserHome,
            KeyboardBrowserRefresh       = libphx.Button_KeyboardBrowserRefresh,
            KeyboardBrowserSearch        = libphx.Button_KeyboardBrowserSearch,
            KeyboardBrowserStop          = libphx.Button_KeyboardBrowserStop,
            KeyboardEject                = libphx.Button_KeyboardEject,
            KeyboardLaunchApp1           = libphx.Button_KeyboardLaunchApp1,
            KeyboardLaunchApp2           = libphx.Button_KeyboardLaunchApp2,
            KeyboardLaunchMail           = libphx.Button_KeyboardLaunchMail,
            KeyboardMediaPlayPause       = libphx.Button_KeyboardMediaPlayPause,
            KeyboardMediaSelect          = libphx.Button_KeyboardMediaSelect,
            KeyboardMediaStop            = libphx.Button_KeyboardMediaStop,
            KeyboardMediaTrackNext       = libphx.Button_KeyboardMediaTrackNext,
            KeyboardMediaTrackPrevious   = libphx.Button_KeyboardMediaTrackPrevious,
            KeyboardPower                = libphx.Button_KeyboardPower,
            KeyboardSleep                = libphx.Button_KeyboardSleep,
            KeyboardAudioVolumeDown      = libphx.Button_KeyboardAudioVolumeDown,
            KeyboardAudioVolumeMute      = libphx.Button_KeyboardAudioVolumeMute,
            KeyboardAudioVolumeUp        = libphx.Button_KeyboardAudioVolumeUp,
            KeyboardWakeUp               = libphx.Button_KeyboardWakeUp,
            KeyboardMeta                 = libphx.Button_KeyboardMeta,
            KeyboardHyper                = libphx.Button_KeyboardHyper,
            KeyboardTurbo                = libphx.Button_KeyboardTurbo,
            KeyboardAbort                = libphx.Button_KeyboardAbort,
            KeyboardResume               = libphx.Button_KeyboardResume,
            KeyboardSuspend              = libphx.Button_KeyboardSuspend,
            KeyboardAgain                = libphx.Button_KeyboardAgain,
            KeyboardCopy                 = libphx.Button_KeyboardCopy,
            KeyboardCut                  = libphx.Button_KeyboardCut,
            KeyboardFind                 = libphx.Button_KeyboardFind,
            KeyboardOpen                 = libphx.Button_KeyboardOpen,
            KeyboardPaste                = libphx.Button_KeyboardPaste,
            KeyboardProps                = libphx.Button_KeyboardProps,
            KeyboardSelect               = libphx.Button_KeyboardSelect,
            KeyboardUndo                 = libphx.Button_KeyboardUndo,
            KeyboardHiragana             = libphx.Button_KeyboardHiragana,
            KeyboardKatakana             = libphx.Button_KeyboardKatakana,
            KeyboardF1                   = libphx.Button_KeyboardF1,
            KeyboardF2                   = libphx.Button_KeyboardF2,
            KeyboardF3                   = libphx.Button_KeyboardF3,
            KeyboardF4                   = libphx.Button_KeyboardF4,
            KeyboardF5                   = libphx.Button_KeyboardF5,
            KeyboardF6                   = libphx.Button_KeyboardF6,
            KeyboardF7                   = libphx.Button_KeyboardF7,
            KeyboardF8                   = libphx.Button_KeyboardF8,
            KeyboardF9                   = libphx.Button_KeyboardF9,
            KeyboardF10                  = libphx.Button_KeyboardF10,
            KeyboardF11                  = libphx.Button_KeyboardF11,
            KeyboardF12                  = libphx.Button_KeyboardF12,
            KeyboardF13                  = libphx.Button_KeyboardF13,
            KeyboardF14                  = libphx.Button_KeyboardF14,
            KeyboardF15                  = libphx.Button_KeyboardF15,
            KeyboardF16                  = libphx.Button_KeyboardF16,
            KeyboardF17                  = libphx.Button_KeyboardF17,
            KeyboardF18                  = libphx.Button_KeyboardF18,
            KeyboardF19                  = libphx.Button_KeyboardF19,
            KeyboardF20                  = libphx.Button_KeyboardF20,
            KeyboardF21                  = libphx.Button_KeyboardF21,
            KeyboardF22                  = libphx.Button_KeyboardF22,
            KeyboardF23                  = libphx.Button_KeyboardF23,
            KeyboardF24                  = libphx.Button_KeyboardF24,
            KeyboardF25                  = libphx.Button_KeyboardF25,
            KeyboardF26                  = libphx.Button_KeyboardF26,
            KeyboardF27                  = libphx.Button_KeyboardF27,
            KeyboardF28                  = libphx.Button_KeyboardF28,
            KeyboardF29                  = libphx.Button_KeyboardF29,
            KeyboardF30                  = libphx.Button_KeyboardF30,
            KeyboardF31                  = libphx.Button_KeyboardF31,
            KeyboardF32                  = libphx.Button_KeyboardF32,
            KeyboardF33                  = libphx.Button_KeyboardF33,
            KeyboardF34                  = libphx.Button_KeyboardF34,
            KeyboardF35                  = libphx.Button_KeyboardF35,
            MouseLeft                    = libphx.Button_MouseLeft,
            MouseMiddle                  = libphx.Button_MouseMiddle,
            MouseRight                   = libphx.Button_MouseRight,
            MouseForward                 = libphx.Button_MouseForward,
            MouseBack                    = libphx.Button_MouseBack,
            MouseX1                      = libphx.Button_MouseX1,
            MouseX2                      = libphx.Button_MouseX2,
            MouseDeltaX                  = libphx.Button_MouseDeltaX,
            MouseDeltaY                  = libphx.Button_MouseDeltaY,
            MouseScrollX                 = libphx.Button_MouseScrollX,
            MouseScrollY                 = libphx.Button_MouseScrollY,
            MouseScrollPixelX            = libphx.Button_MouseScrollPixelX,
            MouseScrollPixelY            = libphx.Button_MouseScrollPixelY,
            GamepadSouth                 = libphx.Button_GamepadSouth,
            GamepadEast                  = libphx.Button_GamepadEast,
            GamepadNorth                 = libphx.Button_GamepadNorth,
            GamepadWest                  = libphx.Button_GamepadWest,
            GamepadC                     = libphx.Button_GamepadC,
            GamepadZ                     = libphx.Button_GamepadZ,
            GamepadLeftTrigger           = libphx.Button_GamepadLeftTrigger,
            GamepadLeftTrigger2          = libphx.Button_GamepadLeftTrigger2,
            GamepadRightTrigger          = libphx.Button_GamepadRightTrigger,
            GamepadRightTrigger2         = libphx.Button_GamepadRightTrigger2,
            GamepadSelect                = libphx.Button_GamepadSelect,
            GamepadStart                 = libphx.Button_GamepadStart,
            GamepadMode                  = libphx.Button_GamepadMode,
            GamepadLeftThumb             = libphx.Button_GamepadLeftThumb,
            GamepadRightThumb            = libphx.Button_GamepadRightThumb,
            GamepadDPadUp                = libphx.Button_GamepadDPadUp,
            GamepadDPadDown              = libphx.Button_GamepadDPadDown,
            GamepadDPadLeft              = libphx.Button_GamepadDPadLeft,
            GamepadDPadRight             = libphx.Button_GamepadDPadRight,
            GamepadLeftStickX            = libphx.Button_GamepadLeftStickX,
            GamepadLeftStickY            = libphx.Button_GamepadLeftStickY,
            GamepadLeftZ                 = libphx.Button_GamepadLeftZ,
            GamepadRightStickX           = libphx.Button_GamepadRightStickX,
            GamepadRightStickY           = libphx.Button_GamepadRightStickY,
            GamepadRightZ                = libphx.Button_GamepadRightZ,
            TouchpadX                    = libphx.Button_TouchpadX,
            TouchpadY                    = libphx.Button_TouchpadY,
            TouchpadMagnifyDelta         = libphx.Button_TouchpadMagnifyDelta,
            TouchpadRotateDelta          = libphx.Button_TouchpadRotateDelta,
            SystemExit                   = libphx.Button_SystemExit,

            ToString                     = libphx.Button_ToString,
        }

        if onDef_Button then onDef_Button(Button, mt) end
        Button = setmetatable(Button, mt)
    end

    return Button
end

return Loader
