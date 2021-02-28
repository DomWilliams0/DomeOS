# Virtual memory layout

```
0000_0000_0000_0000 -> 0000_2000_0000_0000: 32TB for userspace

// min higher half begins at ffff_8000_0000_0000 with 48 bit addresses

ffff_8000_0000_0000 -> ffff_8000_0080_0000: 8MB kernel stacks x 512 cores
ffff_9000_0000_0000 -> ffff_d000_0000_0000: 64TB physical memory mapping from 0

...

ffff_ffff_8000_0000 -> ffff_ffff_c000_0000: 1GB for kernel code mapping from physical 1MB
```