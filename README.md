## Features
- PIT, RTC, HPET timer support
- APIC-based interrupts
- FPU, SSE, AVX support (not tested)
- Current time from RTC+CMOS
- RNG generator support
- kernel memory allocator
- basic future runtime
- data flow manager

## TODO
- [ ] Serial Log
- [X] HPET as a replacement for PIT/RTC
- [X] FPU support

- [ ] Processes/Threads
- [ ] User memory allocation
- [ ] ELF file reader
- [ ] Execute ELF file in userspace
- [ ] Syscalls (map memory pointers as kernel-access memory)