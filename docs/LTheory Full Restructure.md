# Limit Theory Completed Restructure Status

## Status of all Legacy "Apps"

- BSPTest.lua
- - *Functional*
- - Test 3 and Test 5
- - - Hotfixed by changing calls to "rand()" to "math.random()". I cannot find any place where "rand()" is defined.
- - Test 6
- - - Pressing right does not display anything. Cannot be sure if this is a bug or intentional.
- CoordTest
- - *Complete* Passes Every Test.
- AudioTest
- - *Does Not Work* Uses deprecated Key Input functions
- GenTex2D
- - *Complete* Fully Functional
- InputTest
- - *Does Not Work* Is an Incomplete App, Errors on Launch.
- PhysicsTest
- - *Functional* Unclear if all functionality is working, but appears fine.
- ShipTest
- - *Complete* Fully Functional
- TestEcon
- - *Complete* Fully Functional
- TestHmGui
- - *Complete* Fully Functional
- TestIcon
- - *Does Not Work* Relies on Unimplemented Icon UI Element.
- TestImGui
- - *Does Not Work* Relies on ```'./screenshot/wp2.png'``` which does not exist
- Todo
- - *Does Not Work* Relies on ```'./screenshot/wp2.png'``` which does not exist

# New Structure Overview

## Config
- All Configuration Files needed for game/engine initialization.
  - Aliases.lua
    - Function Aliases for ease of use.
  - App.lua
    - Set all app initial state variables
  - JIT.lua
    - Set all relevant LuaJIT variables
  - Local.lua
    - Extra App and JIT variable setting. Could be integrated into other config

## Core
- Files inside of Core are Global functions, classes, and class extensions which add additional functionality and convenience to lua
  - These files do not directly relate to Gameplay functionality
### CFFI
- Lua to C Foreign Function Interface Files and Helpers.
  - Gives lua the ability to more conveniently interact with libphx.
### LuaExtensions
- Lua function and class extensions and overrides
### Structures
- Custom General Purpose Structures and Data Structures.
- Settings.lua
  - Stores global settings during game runtime. These settings are primarily set and read by Renderer.lua.
### Util
- All other General Purpose Utility Functions that are in the Global Namespace.
  - Each file is either:
    - A file with one or more General Purpose utility functions
    - A specific use-case structure or set of Functions which are specific to running the engine, not gameplay related.

## GameObjects
- All GameObjects and GameObjects Specific Scripts are here.
### "Floating Files" (Ones that are in the base folder)
- Action.lua, Entity.lua, Job.lua, material.lua
  - The Base Class Definitions for these script types.
### Actions
- Actions are AI Exclusive scripts that define the actions and how they function for all AI
### Components
- Components are scripts which are attached to Entities that allow them further functionality.
#### Core Components
- Components which are applied to a wide variety of different Entities
#### Economy Components
- Components which relate specifically to Economic Activity
#### Material Components
- Components which relate to adding Materials to Entities
#### NPC Components
- Components which are used only on NPC's
#### Object Components
- Components which are used only on Objects (I.E. Not NPC's, Not Player)
#### Other Components
- Components which do not fit within other groups.
- This folder is questionable, but for now it works. Things could be moved to other folders or subdivided later if needed.
#### Ship Components
- Components which relate to Ship Construction
### Entities
- All Entities which can be placed in the Gamespace. Components are Attached to Entities
#### Effect Entities
- Entities which are visual effects
  - Pulse could be placed somewhere else, as it is an actual turret shot. But as of now it is the only of its kind therefore fits here.
#### Objects
- Entities that are General Objects. Could be subdivided once more objects are created
#### Ship
- All Entities which (are/are placed on) Ships
#### Test
- Entities which are used to test functionality or larger implementation
- NOTE: System.lua is extremely important. This is what is used to create large systems at the moment. This File should be replaced later on. It handles far too much functionality on its own, it is essentially an App, but functions as an Entity currently. This is highly inefficient but will require further development to replace.
#### Files inside Entities but not in folders
- Player.lua
  - Used for creating the Player
- Trigger.lua
  - A General Trigger that can be placed in the gamespace for something to happen when entered.
- Zone.lua
  - An invisible Zone that can be entered, and can have something happen if that happens.
### Jobs
- Jobs which AI uses. This functions in conjunction with Actions. Unsure what the clear distinction between this an actions is.
### Materials
- Unused Folder, which will later be used for more specific Material definitions as that gets more developed.

## Render
- Contains all Files used to render the gamespace visually. This is Globally Scoped
## Scenes
- Empty Folder which would be implemented if Statemanagement were to be developed further
## States
- Contains All Apps which define the gamespace.
- POTENTIAL NAMING CONFUSION: This is Named States for the potential development of a State Management System. This would allow for more configurable and deep Applications to be made. Currently Apps are monolithic in their implementation. These files will spiral in length as more development is done.
### "Floating Files" (Files in the base folder)
- Application
  - Base File for that all Applications are built ontop of
- ApplicationBindings
  - General Bindings which all Applications can use
- State.lua and StateMachine.lua
  - Empty Files which would be the base scripts for State Creation and Management.
### App
- Contains all Apps created by Josh for Testing Features and Proof of Concepts
### Stable
- Empty folder which would contain all Stable and working States
### Test
- Empty folder which would contain all States for Testing purposes
## Systems
- All Systems that are used throughout the game.
  - Example: Economy contains all scripts which actively track the activity happening in the Economy.
### Camera
- Camera Scripts which define different Functionality for the Gamespace Camera.
  - Note: This could be considered a GameObject. But I believe its scope is wider than most Gameobjects are. Willing to Change.
### CommandView
- Less clear System. This Contains Tracking all entities in a system, and showing a full systemMap.
- Honestly these files could be moved elsewhere. But currently I believe this makes most sense. Will move if needed.
### Controls
- All Scripts which deal with the Controls a player can use. These are imported to add specific Controller Contexts to an app or state.
#### "Floating Files" (Files in the Base Folder)
- Control.lua
  - Base Script for all Controls
- Gamepads.lua
  - Base Script for adding Gamepad functionality
#### Bindings
- Specific Key Bindings to different Controls Contexts
#### Controls
- General Definitions for different Controls Contexts.
- Example: MasterControl can be added to any app to give specific Controller/Keyboard actions
### Economy
- Controls the Economy and Defines base Economic items/systems
### Events
- This System allows for Events to be sent and read by systems and gameobjects
### Gen
- For Procedural Generation of Ships, Asteroids, Systems, Shapes.
### Menu
- Unused Folder, For potential Development of Menu Systems. This is similar to UI, but would be more specific definitions of how the UI scripts come together.
### Overlay
- For Displaying any overlays, both Debug and actual HUD
## UI
- All UI Related Elements and Functionality is here. I never touched this folder. It is a little messy, and could use further subdivision.
## Init.lua
- Used to Initialize Global Table. This includes JIT, FFI, Core/*
## Main.lua
- Used to Initialize the full game, including the "App"

# How Does Init.lua work?
1. Set Flags if not defined in Main.cpp
2. Import ffi and jit
3. Import all functions in "math" into Global Table
  - Unclear exactly why this definition happens. Legacy from Josh.
4. Initialize Core, Render, Config for their addition to Global Table Later.
5. Importing and Requiring Aliases and ToString
   1. Import Config.Aliases.lua so that all Aliases are globally accessible. Required for ToString
   2. Require Core.LuaExtensions.ToString.lua for globally accessible improved ToString
6. Requiring LuaExtensions and RequireAll
   1. Require both LuaExtensions IOEx and StringEx. Required for Core.util.RequireAll
   2. Require Core.util.RequireAll Giving Us the ability to use the requireAll Function Globally
7. Finish Loading Util and Structures into Core and Global Namespace
   1. Require Core.Structures and Core.Util
   2. Add Core.Util and Core.Struct into Core Namespace
   3. Require Systems.Events and Add it into Core Namespace
      -   Note: Events is within Systems. While this is technically a Core System, it is also a General Game System which requires Global Accessibility.
8. Load FFI into Core. I do not fully understand this, but it is used for Global FFI usage.
9. Additional Lua Configuration
   1.  A bunch of Typedef stuff I do not understand. Has to do with FFI
10. Require Renderer into the Global Space.
    1.  RequireAll(Render) and then add it into the "Render" Object
11. Core.Call(fn) - Used for running Main.lua with ErrorHandling.

# How Does Main.lua work?
1. require('Init') run the Init.lua file for all Global Space Definitions
2. local app - Either the CLI argument or start with the "ltheory" app.
3. GlobalRestrict.On() Disallow for anything else to be added to the Global Table.
4. Run Config.App.lua - Initialize app configuration variables
5. Run Config.Local.lua - Initialize other app configuration variables
6. Namespace.Load("UI") - Loads in all of the UI elements
7. Namespace.LoadInline("Systems") - Loads all of the Systems that have not be Loaded yet.
8. Namespace.LoadInline("GameObjects") - Loads all of the GameObjects.
9. Set Initial JIT Varaibles
10. Run the App defined by "local app"
11. After App is done running turn off GlobalRestrict.