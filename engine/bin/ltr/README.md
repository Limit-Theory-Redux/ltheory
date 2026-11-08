# Limit Theory Redux

## Log output

To see more logging run application with **RUST_LOG** environment variable:

```bash
RUST_LOG=debug ltr
```
Possible parameters: trace, debug, info, warn, error

## Exit code

`ltr` exits with the code the app ends with, so a script or CI job can run an app to completion:
- `0` when the window is closed, or the app calls `Application:quit()` (or `Engine:exit()`).
- The given code when the app calls `Application:quit(code)` (or `Engine:exitWithCode(code)`).
- `1` (`Core.ExitCode.Failure`) when a Lua error reaches `Core.Call`, including an unknown app name.
  The engine then runs no more frames, still calls the app's exit handler, and shuts down
  normally, so the log file is complete.

If several codes are requested, the first non-zero one wins. A panic in engine code exits with `1`
at once, without a normal shutdown.
