# virtualdisplay

Creates a virtual 240Hz display and mirrors your main screen to it.
Useful for unlocking higher refresh rates on displays that support it through software.

## How it works

Uses macOS's private `CGVirtualDisplay` API to create a virtual monitor
at your main screen's resolution with a 240Hz refresh rate, then configures
the system to mirror your physical display to it via `CGConfigureDisplayMirrorOfDisplay`.

## Build

Note: You need to clone the Froststrap repo to be able to get the workspace Cargo.toml
as this project is intended to only work apart of the backend dir, but to keep maintainability
it's still using the registered fork repo.

This requires Rust and Swift currently.

```sh
cargo b -r
```


## Notice

This project is being rewritten in Rust due to the fact the Swift compiler is so
incredibly bad (majority being speed and behviour depending on what it is compiled on)
that it's better just to port the thing over to another language, since it's integrated
with Froststrap, most of the dependencies are also used by other rust modules/libraries
making compile time faster than with Swift.
