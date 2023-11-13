# Contributions
We discuss all features, fixes etc. to be contributed on Discord.

## Discussing Gamedesign
Discussions regarding game design are held in the ltheory-crafting channel:

**Open on:**
\
\
<a href="https://discord.com/channels/695088786702336000/1021252323663691826" style="display: block;">
  <img style="height: 36px; display: block;" src="https://assets-global.website-files.com/6257adef93867e50d84d30e2/636e0b5061df29d55a92d945_full_logo_blurple_RGB.svg"/>
</a>


## Discussing Gamesystems
Discussions regarding game systems / programming are held in the programming-discussion channel:

**Open on:**
\
\
<a href="https://discord.com/channels/695088786702336000/1021816893629272174" style="display: block;">
  <img style="height: 36px; display: block;" src="https://assets-global.website-files.com/6257adef93867e50d84d30e2/636e0b5061df29d55a92d945_full_logo_blurple_RGB.svg"/>
</a>

## Documentation
It´s highly recommended you document what you are doing in code. For more generalized documentation & design we use a seperately hosted [Wiki.js Instance](https://wiki.ltredux.org). For wiki editoral access contact @IllustrisJack . A backup repository of the wiki exists [here](https://github.com/Limit-Theory-Redux/wiki).

# Getting Started
In this file we´ll go over the whole process of getting started & the the workflow of Limit Theory Redux development.

## Prerequisites

To build Limit Theory, you'll need a few standard developer tools. All of them are available to download for free.

### Windows

To work on any of the Lua scripts, the following tools are required:
- Git: https://git-scm.com/downloads

You may want to install a GUI for Git, such as GitHub for Desktop: https://desktop.github.com/

#### Optional dependencies

Optionally, if you would like to made changes to the libphx engine, you will also need:
- Visual Studio Community: https://visualstudio.microsoft.com/vs/
- CMake: https://cmake.org/download/
- LLVM: https://releases.llvm.org/download.html (download the latest Windows installer by going to the GitHub releases page, and downloading LLVM-xx.x.x-win64.exe)
- Rust: https://www.rust-lang.org/tools/install (use rustup-init.exe, then type 1 and press Enter in the terminal window)

Everything other than Visual Studio can be installed using [winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/) if you're on Windows 10 1709 or later:

- `winget install Kitware.CMake LLVM.LLVM Rustlang.Rustup`

### macOS

Users on macOS will need to install Git, CMake, Xcode, Rust and LLVM.

First, install Xcode using the Mac App store: https://apps.apple.com/us/app/xcode/id497799835

To install the remaining dependencies, we recommend first installing the Homebrew package manager if you haven't already: https://brew.sh/

Once Homebrew is installed, open a **Terminal** window and run the following one-liner:

- `brew install git cmake rust llvm`

### Linux

You should install Git, CMake, a C++ toolchain, Cargo, Rust and LLVM using your distro's package manager. OpenGL, GLU and ALSA development libraries are also required.

For example, if using Ubuntu 22.04, open a terminal and install the following packages:

- `sudo apt install -y git build-essential cmake libgl1-mesa-dev libglu1-mesa-dev libasound2-dev cargo llvm-dev libclang-dev clang`

# Setting up

With the above prerequisites installed, let's get the `libphx` engine set up and ready to run a Lua application. First, open a terminal window (**Git Bash** on Windows).

## Check out the Repository

Use `cd` to change directories to the place where you want to download LTR.

- `cd <path where you want to put the LTR source>` (for example: `~/Desktop` for the desktop)

Now, you can download the repository:

- `git clone https://github.com/Limit-Theory-Redux/ltheory.git`

## Set up libphx

Next, we will need to get the engine ready to run Lua applications. There are two options, downloading a precompiled binary package, or compiling libphx manually.

### Option 1: Using precompiled binaries

1. Download the latest binary release by going to the `latest` release on GitHub, and downloading the right `bin` package for your platform: https://github.com/Limit-Theory-Redux/ltheory/releases/tag/latest
   * Windows users should download `ltheory-bin-win32.zip`, we recommend other users to skip to [Compiling libphx manually](#option-2-compiling-libphx-manually).
2. Navigate to the directory of the checked-out repository (e.g. `~/Desktop/ltheory` if you cloned to the desktop).
3. Create a new folder named `bin` if it does not exist already.
4. Extract the contents of the zip file downloaded in step 1 into `bin`. The `bin` directory should now contain a number of libraries and executable files, including `lt64`.

Once you've completed these steps, you can skip straight to [Running a Lua App](#running-a-lua-app)

### Option 2: Compiling libphx manually

> As mentioned in [Prerequisites](#prerequisites), the additional optional dependencies are required to compile libphx manually.

Once you have the repository, the build process consists of two steps (as with other CMake projects): generating the build files, and then building. There is a Python script `configure.py` at the top level of the repository to help you do this easily.

From a terminal in the directory of the checked-out repository, run

- `cmake -B build`

This runs CMake to generate the build files and places them in 'build`. Then, to compile

- Windows: `cmake --build ./build --config RelWithDebInfo`
- macOS/Linux: `cmake --build ./build`

## Running a Lua App

If the compilation is successful, you now have `bin/lt64` (or `bin/lt64.exe` on Windows), which is the main executable. This program launches a Lua script. The intention is for Limit Theory (and all mods) to be broken into many Lua scripts, which would then implement the gameplay, using script functions exposed by the underlying engine.

To launch the default script ('LTheoryRedux'), you can run the launcher directly from a terminal / command prompt:

- Windows: `./bin/lt64.exe`
- macOS/Linux: `./bin/lt64`

To launch a specific script, add its name to the end:

- Windows: `./bin/lt64.exe <script name without extension>` (i.e. `./bin/lt64.exe PlanetTest`)
- macOS/Linux: `./bin/lt64 <script name without extension>` (i.e. `./bin/lt64 PlanetTest`)

All top-level scripts are in the `script/States/App` directory.

# Troubleshooting

## Debugging in Visual Studio

First, make sure that the CMake project is configured by running the steps above up to `cmake -B build`.

Next, open the Visual Studio solution by navigating to `build/LTheory.sln` and opening it. Once the project has loaded, right-click the `lt` project in the Solution Explorer, then select "Set as Startup Project".

To select a Lua script to run, right-click the `lt` project, then select Properties, then Debugging, then change the value in "Command Line Arguments" to the desired Lua script. Leave this blank to launch the default Lua script (`LTheoryRedux`).


