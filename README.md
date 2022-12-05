# Limit Theory

Limit Theory is a now-cancelled open world space simulation game.

This repository is the game (not engine) code for the second generation of LT's development, when all work was migrated to C and Lua. For the older, C++/LTSL Limit Theory, see https://github.com/JoshParnell/ltheory-old.

![LT Screenshot](./res/tex2d/screenshot.png)


# Getting Started with Ltheory
## Prerequisites

To build Limit Theory, you'll need a few standard developer tools. All of them are available to download for free.

- Python: https://www.python.org/downloads/
- Git: https://git-scm.com/downloads
- Git LFS: https://git-lfs.github.com/

Below only required for Manually Compiling LT:
- Visual Studio Community: https://visualstudio.microsoft.com/vs/
- CMake: https://cmake.org/download/

## Checking out the Repository
First open a **Git Bash terminal**

Then, use `cd` to change directories to the place where you want to download LT.
- `cd ~/Desktop/<path where you want to put the LT source>`

Before doing any other `git` commands, make sure LFS is installed:
- `git lfs install`

You should see `Git LFS initialized` or a similar message. **Important**: if you forget to install and initialize Git LFS, most of the resources will probably be broken, and the whole process will likely fail in strange and mysterious ways. This is a common gotcha with projects that use LFS. Make sure you do the above step!

Now, you can download the repository:

- `git clone https://github.com/Limit-Theory-Redux/ltheory.git`

## Compiling (Option 1: Precompiled Bin)

You'll need need to download a precompiled bin folder.
1. Go to https://github.com/Limit-Theory-Redux/ltheory/actions/runs/3620791750
2. Under "Artifacts" Click on "ltheory" and download.
3. Navigate to the top level of the repository. (`~/Desktop/ltheory`)
4. Create a new folder named `bin` if it does not exist.
5. Extract ltheory.zip into `bin`
6. Your Bin folder should now look like this:

<details>
<summary> Example contents of `bin\` </summary>

```
fmodL64.dll
fmodstudioL64.dll
glew32.dll
liblz4.dll
libphx64.dll
libphx64.pdb
libphx64d.dll
libphx64d.pdb
lt64.exe
lt64.exp
lt64.lib
lt64.pdb
lt64d.exe
lt64d.exp
lt64d.lib
lt64d.pdb
lua51.dll
phx64.exp
phx64.lib
phx64d.exp
phx64d.lib
SDL2.dll
```

</details>

## Compiling (Option 2: Manually)

[CMake](https://cmake.org/download/) and [Visual Studio Community](https://visualstudio.microsoft.com/vs/) are Required for this Option.

Once you have the repository, the build process proceeds in two steps (as with other CMake builds): generating the build files, and then building. There is a Python script `configure.py` at the top level of the repository to help you do this easily.

From a terminal in the directory of the checked-out repository, run

- `python configure.py`

This runs CMake to generate the build files. Then, to compile,

- `python configure.py build`

## Running a Lua App

If the compilation is successful, you now have `bin/lt64.exe`, which is the main executable. This program launches a Lua script. The intention was for Limit Theory (and all mods) to be broken into many Lua scripts, which would then implement the gameplay, using script functions exposed by the underlying engine.

To launch a Lua script, you can again use the python helper:

- `python configure.py run`

To run the default script ('LTheory'), or

- `python configure.py run <script_name_without_extension>`

to run a specific script. All top-level scripts are in the `script/States/App` directory.

# Example of the Entire Process

## With Precompiled Bin
An example of the entire process to run LT, starting from nothing except the prerequisites. 
Using a Precompiled Bin

<details>
<summary> Example Full Run </summary>

Open Git Bash.
```
cd ~/Desktop
git lfs install
git clone https://github.com/Limit-Theory-Redux/ltheory.git
cd ltheory
mkdir bin
```
- Download ltheory from https://github.com/Limit-Theory-Redux/ltheory/actions/runs/3620791750
- Extract contents of ltheory.zip into `~/Desktop/ltheory/bin`
Now go back into Git Bash.
```
cd ~/Desktop/ltheory
python configure.py run
```

</details>

## With Manual Compilation
An example of the entire process to run LT, starting from nothing except every prerequisite (including Visual Studio Community and CMake):

<details>
<summary> Example Full Run </summary>

Open Git Bash. Each line below is one command, some of which will take a while to complete:

```
cd ~/Desktop
git lfs install
git clone https://github.com/Limit-Theory-Redux/ltheory.git
cd ltheory
python configure.py
python configure.py build
python configure.py run
```

</details>

# Debugging in Visual Studio

First, make sure that the CMake project is configured by running the steps above up to `python configure.py`.

Next, open the Visual Studio solution by navigating to `build/LTheory.sln` and opening it. Once the project has loaded, right click the `lt` project in the Solution Explorer, then select "Set as Startup Project".

To select a Lua script to run, right click the `lt` project, then select Properties, then Debugging, then change the value in "Command Line Arguments" to the desired Lua script. Leave this blank to launch the default Lua script (`LTheory`).
