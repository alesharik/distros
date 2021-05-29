## Features
- PIT, RTC, HPET timer support
- APIC-based interrupts
- FPU, SSE, AVX support (not tested)
- Current time from RTC+CMOS
- RNG generator support
- kernel memory allocator

## TODO
- [ ] Serial Log
- [X] HPET as a replacement for PIT/RTC
- [X] FPU support

- [ ] User memory allocation
- [ ] ELF file reader
- [ ] Execute ELF file in userspace
- [ ] Syscalls (map memory pointers as kernel-access memory)
- [ ] Processes/Threads