# Limit Theory

Limit Theory is a now-cancelled open world space simulation game.

This repository is the game (not engine) code for the second generation of LT's development, when all work was migrated to C and Lua. For the older, C++/LTSL Limit Theory, see https://github.com/JoshParnell/ltheory-old.

![LT Screenshot](./res/tex2d/screenshot.png)

# Prerequisites

To build Limit Theory, you'll need a few standard developer tools. All of them are available to download for free.

- Python: https://www.python.org/downloads/
- Git: https://git-scm.com/downloads
- Git LFS: https://git-lfs.github.com/
- Visual Studio Community: https://visualstudio.microsoft.com/vs/
- CMake: https://cmake.org/download/

## Ubuntu

If you're using Ubuntu, you will also need to install a few developer libraries. This one-liner will install all required dependencies:

- `sudo apt-get -y install python git git-lfs cmake libluajit-5.1-dev libglew-dev libsdl2-dev liblz4-dev libfreetype-dev libbullet-dev`

On some versions of Ubuntu, you may also need to install this if you get missing `python` errors:

- `sudo apt-get -y python-is-python3`

# Building

With the above prerequisites installed, open a **Git Bash terminal**.

## Checking out the Repository

First, use `cd` to change directories to the place where you want to download LT.
- `cd ~/Desktop/<path where you want to put the LT source>`

Before doing any other `git` commands, make sure LFS is installed:
- `git lfs install`

You should see `Git LFS initialized` or a similar message. **Important**: if you forget to install and initialize Git LFS, most of the resources will probably be broken, and the whole process will likely fail in strange and mysterious ways. This is a common gotcha with projects that use LFS. See the [Troubleshooting](#troubleshooting) section below for how to resolve this.

Now, you can download the repository:

- `git clone --recursive https://github.com/JoshParnell/ltheory.git ltheory`

## Compiling

Once you have the repository, the build process proceeds in two steps (as with other CMake builds): generating the build files, and then building. There is a Python script `configure.py` at the top level of the repository to help you do this easily.

From a terminal in the directory of the checked-out repository, run

- `python configure.py`

This runs CMake to generate the build files. Then, to compile,

- `python configure.py build`

## Running a Lua App

If the compilation is successful, you now have `bin/lt64.exe`, which is the main executable. This program launches a Lua script. The intention was for Limit Theory (and all mods) to be broken into many Lua scripts, which would then implement the gameplay, using script functions exposed by the underlying engine.

To launch a Lua script, you can run the executable:

- `python configure.py run`

To run the default script ('LTheory'), or

- `python configure.py run <script_name_without_extension>`

to run a specific script. All top-level scripts are in the `script/App` directory.

### Ubuntu

On Ubuntu, the python helper script doesn't run the executable correctly. Instead, to launch a Lua script, you can run the executable directly:

- `bin/lt64`

To run the default script ('LTheory'), or

- `bin/lt64 <script_name_without_extension>`

to run a specific script.

# Example of the Entire Process

An example of the entire sequence of commands to run LT, starting from nothing (but having the prerequisites installed):

Open Git Bash. Each line below is one command, some of which will take a while to complete:

```
cd ~/Desktop
git lfs install
git clone --recursive https://github.com/JoshParnell/ltheory.git ltheory
cd ltheory
python configure.py
python configure.py build
python configure.py run
```

# Troubleshooting

## Unable to load resources such as `Failed to load image from './res/tex2d/metal/01_d.jpg'`

This usually happens if you've cloned the repository without downloading the games resources via Git LFS. Open a terminal and run:

- `git lfs fetch`

then:

- `git lfs checkout`

## Debugging in Visual Studio

First, make sure that the CMake project is configured by running the steps above up to `python configure.py`.

Next, open the Visual Studio solution by navigating to `build/LTheory.sln` and opening it. Once the project has loaded, right click the `lt` project in the Solution Explorer, then select "Set as Startup Project".

To select a Lua script to run, right click the `lt` project, then select Properties, then Debugging, then change the value in "Command Line Arguments" to the desired Lua script. Leave this blank to launch the default Lua script (`LTheory`).
