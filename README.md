# TWOCB - JS live coding environment for LED strips.

**twocb** is an in-development live coding environment for writing "patterns" for lights. Specifically targetted towards volumetric LED installations where iteration speed and performance matters. The project is inspired by the absolutely fantastic [Pixelblaze](https://www.bhencke.com/pixelblaze) by Ben Hecke, providing the same rapid development cycle although on much higher performance CPUs. **twocb** uses [rusty_v8](https://github.com/denoland/rusty_v8), providing the speed of the V8 engine for patterns which mean that you can do a surprising amount of patterns on a normal CPU while maintaining 200 FPS for PoV applications.

The project also aims to ship with audio analysis primitives and as of today, **twocb** comes with beat tracking (using [aubio](https://github.com/katyo/aubio-rs)) and ["colorchord2"](https://github.com/cnlohr/colorchord) (using a custom Rust binding). This means that you have access to a rich set of analayzed data sources to draw on for writing patterns.

It's still in development and more info on how to run it will come in a bit.

### Special thanks

-   [ben0x539](https://github.com/ben0x539) for all the Rust help
-   [Love Olsson](https://github.com/loveolsson) for the initial design of the workflow + electronics consulting
-   [cnlohr](https://github.com/cnlohr) for the inspiration
-   [pajlada](https://github.com/pajlada) gachiGASM
