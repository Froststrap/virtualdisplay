# virtualdisplay

Creates a virtual 240Hz display and mirrors your main screen to it. Useful for unlocking higher refresh rates on displays that support it through software.

## How it works

Uses macOS's private `CGVirtualDisplay` API to create a virtual monitor at your main screen's resolution with a 240Hz refresh rate, then configures the system to mirror your physical display to it via `CGConfigureDisplayMirrorOfDisplay`.

## Build

Requires Xcode Command Line Tools.

```sh
swift build -c release
```
