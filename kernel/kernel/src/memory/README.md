# Virtual memory layout

```
0000_1ff8_0000_0000 -> 0000_2000_0000_0000: 1MB x 32768 user stacks

// userspace is 32TB, ends at 0000_2000_0000_0000

// min higher half begins at ffff_8000_0000_0000 with 48 bit addresses

ffff_8000_0000_0000 -> ffff_8001_0000_0000: 8MB isr/irq kernel stacks x 512 cores
ffff_8001_0000_0000 -> ffff_8002_0000_0000: 128KB x 32768 kernel stacks for threads per process
ffff_9000_0000_0000 -> ffff_d000_0000_0000: 64TB physical memory mapping from 0
ffff_d000_0000_0000 -> ffff_d004_0000_0000: 16GB kernel heap

...

ffff_ffff_8000_0000 -> ffff_ffff_c000_0000: 1GB for kernel code mapping from physical 1MB
```