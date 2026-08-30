#!/bin/sh

MACOS_MAJOR=$(sw_vers -productVersion | cut -d. -f1)

if [ "$MACOS_MAJOR" -ge 27 ]; then
    swift build -c release --arch arm64
else
    swift build -c release --arch arm64 --arch x86_64
fi
