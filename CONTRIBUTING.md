# Contributions

We discuss all features, fixes etc. to be contributed on Discord.

## Licensing Information

By contributing to Limit Theory Redux, you agree to the following licensing structure:

- **Original Limit Theory Content**: All content derived from Josh Parnell's original Limit Theory project remains under [The Unlicense](./UNLICENSE-ORIGINAL.txt), effectively placing it in the public domain.

- **New Contributions**: All new content and substantial modifications are dual-licensed under the [Apache License 2.0](./LICENSE-APACHE-2.0) and [MIT License](./LICENSE-MIT). You may choose either license at your option.

- **Mixed Content**: When modifying existing Unlicensed content from the original project, those specific portions remain under The Unlicense.

For more detailed information about our licensing approach, please see the [NOTICE](./NOTICE) file.

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

It´s highly recommended you document what you are doing in code. For more generalized documentation & design we use a seperately hosted [Wiki.js Instance](https://wiki.ltredux.org). For wiki editoral access contact [@IllustrisJack](https://github.com/IllustrisJack) directly or a maintainer. A backup repository of the wiki exists [here](https://github.com/Limit-Theory-Redux/wiki).

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
- LLVM: https://releases.llvm.org/download.html (download the latest Windows installer by going to the GitHub releases page, and downloading LLVM-xx.x.x-win64.exe)
- Rust: https://www.rust-lang.org/tools/install (use rustup-init.exe, then type 1 and press Enter in the terminal window)

Everything other than Visual Studio can be installed using [winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/) if you're on Windows 10 1709 or later:

- `winget install LLVM.LLVM Rustlang.Rustup`

### macOS

Users on macOS will need to install Git, Xcode, Rust and LLVM.

First, install Xcode using the Mac App store: https://apps.apple.com/us/app/xcode/id497799835

To install the remaining dependencies, we recommend first installing the Homebrew package manager if you haven't already: https://brew.sh/

Once Homebrew is installed, open a **Terminal** window and run the following one-liner:

- `brew install git rust llvm`

### Linux

You should install Git, Cargo, Rust, GCC and LLVM using your distro's package manager. OpenGL, GLU and ALSA development libraries are also required.

For example, if using Ubuntu 22.04, open a terminal and install the following packages:

- `sudo apt install -y git build-essential libgl1-mesa-dev libglu1-mesa-dev libasound2-dev cargo llvm-dev`

Arch: `sudo pacman -Syu --needed base-devel llvm glibc`

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

1. Download the latest binary release by going to the `latest` release on GitHub, and downloading the right `dev-binaries` package for your platform: https://github.com/Limit-Theory-Redux/ltheory/releases/tag/latest
   - Windows users should download `dev-binaries-windows.zip`, we recommend other users to skip to [Compiling libphx manually](#option-2-compiling-libphx-manually).
2. Navigate to the directory of the checked-out repository (e.g. `~/Desktop/ltheory` if you cloned to the desktop).
3. Create a new folder named `bin` if it does not exist already.
4. Extract the contents of the zip file downloaded in step 1 into `bin`. The `bin` directory should now contain a number of libraries and executable files, including `ltr`.

Once you've completed these steps, you can skip straight to [Running a Lua App](#running-a-lua-app)

### Option 2: Compiling libphx manually

> As mentioned in [Prerequisites](#prerequisites), the additional optional dependencies are required to compile libphx manually.

Limit Theory Redux is a Rust application, and therefore utilises the `cargo` toolchain for building. We also have a helper script `build.sh` written in Bash to do some of the heavy lifting. If you're on Windows, you'll need to use the Git Bash terminal (included when installing Git itself), it likely won't work from cmd.exe.

From a terminal in the directory of the checked-out repository, run:

- `./build.sh`

This will build the engine code and place it in the `bin` directory. `build.sh` is a helper script that does two things: runs `cargo build` then copies the binaries out of the `target` directory into `bin`. You can also run `./build.sh` with the `--debug` flag to disable optimizations and incorporate debug symbols, and `--run-tests` to run unit tests.

## Running a Lua App

If the compilation is successful, you now have `bin/ltr` (or `bin/ltr.exe` on Windows), which is the main executable. This program launches a Lua script. The intention is for Limit Theory (and all mods) to be broken into many Lua scripts, which would then implement the gameplay, using script functions exposed by the underlying engine.

To launch the default script ('LTheoryRedux'), you can run the launcher directly from a terminal / command prompt:

- Windows: `./bin/ltr.exe`
- macOS/Linux: `./bin/ltr`

To launch a specific script, add its name to the end:

- Windows: `./bin/ltr.exe <script name without extension>` (i.e. `./bin/ltr.exe PhysicsTest`)
- macOS/Linux: `./bin/ltr <script name without extension>` (i.e. `./bin/ltr PhysicsTest`)

All top-level scripts are in the `script/States/App` directory.

## Quickly iterate on engine changes

As we use the `cargo` ecosystem, if you're iterating on engine code, it might be easier to simply run:

- `cargo run`

This will essentially run `cargo build && ./target/debug/ltr` in one step. You can pass `--release` to `cargo run` to enable optimizations. If you'd like to launch with a specific script, you can pass it as an argument to `cargo run`:

- `cargo run -- <script name without extension>`

## Formatting

Engine code uses `cargo +nightly fmt` to format its code. If you get a linting error in CI, you'll need to run this first to ensure your changes are formatted correctly. The script `./format.sh` is provided for convenience.
