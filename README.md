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

This requires Rust.

```sh
cargo b -r
```
