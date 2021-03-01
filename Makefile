.PHONY: build
build:
	scons

.PHONY: test
test:
	scons test

.PHONY: run
run:
	scons run headless=1

.PHONY: debug
debug:
	scons debug headless=1
