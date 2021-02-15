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
mkdir build
cd build
cmake ..
make run
```
