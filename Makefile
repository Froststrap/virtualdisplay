APP    = virtualdisplay
SIGN  ?= -

.PHONY: build clean install

build:
	swift build -c release
	codesign --force --sign "$(SIGN)" \
		--entitlements virtualdisplay.entitlements \
		.build/release/$(APP)

clean:
	swift package clean

install: build
	cp .build/release/$(APP) /usr/local/bin/$(APP)
