# DomeOS
A toy x86_64 OS, third time lucky.

### Building
Only tested on Linux with the Rust nightly specified in `rust-toolchain`.

Dependencies:

* mtools
* grub
* xorriso
* qemu
* ld

```
$ scons                # builds only
$ scons run            # builds and runs in qemu
$ scons run headless=1 # builds and runs in qemu with no graphical window

```
