#!/usr/bin/env nu

let major: int = (sw_vers -productVersion | split row "." | get 0 | into int)

if $major >= 27 {
    swift build -c release --arch arm64
} else {
    swift build -c release --arch arm64 --arch x86_64
}
