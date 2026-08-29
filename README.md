# virtualdisplay

Creates a virtual 240Hz display and mirrors your main screen to it. Useful for unlocking higher refresh rates on displays that support it through software.

## How it works

Uses macOS's private `CGVirtualDisplay` API to create a virtual monitor at your main screen's resolution with a 240Hz refresh rate, then configures the system to mirror your physical display to it via `CGConfigureDisplayMirrorOfDisplay`.

## Build

Requires Xcode Command Line Tools.

```sh
swift build -c release
```

## Usage

```sh
virtualdisplay           # start with menu bar icon
virtualdisplay --no-menu # start headless, stop with Ctrl-C or SIGTERM
virtualdisplay --help
```

When running, a display icon appears in the menu bar. Click it and select **Stop** to disable mirroring and remove the virtual display. The original display resolution is restored on exit.

## Signing

By default the binary is ad-hoc signed. If the virtual display fails to initialize, sign with a Developer ID instead:

```sh
make SIGN="Developer ID Application: Your Name (TEAMID)"
```
