## Features
- PIT, RTC, HPET timer support
- APIC-based interrupts
- FPU, SSE, AVX support (not tested)
- Current time from RTC+CMOS
- RNG generator support
- kernel memory allocator
- basic future runtime
- data flow manager
- PS/2 device support
- PCI support
- very basic terminal
- SMBios support

## TODO
- [ ] Serial Log
- [X] HPET as a replacement for PIT/RTC
- [X] FPU support

- [ ] Processes/Threads
- [ ] User memory allocation
- [ ] ELF file reader
- [ ] Execute ELF file in userspace
- [ ] Syscalls (map memory pointers as kernel-access memory)

## External TODO
- [ ] Add scroll wheel support to `ps2` crate

## Global flow system
- `/dev` - dev flow system
- `/dev/ps2` - PS/2 devices
- `/dev/ps2/keyboard` - PS/2 keyboard (sends `KeyboardMessage`)
- `/dev/ps2/mouse` - PS/2 mouse (sends `MouseMessage`)
- `/dev/syslog` - System log (sends `SyslogMessage`)
- `/dev/tty` - TTY devices
- `/dev/tty/vga` - TTY VGA device (sends/receives `TtyMessage`)
- `/dev/pci/{bus}/{device}/{function}` - PCI device information
- `/dev/pci/{bus}/{device}/{function}/bar/{id}` - PCI device BAR information
- `/dev/smbios` - SMBios information
- `/dev/kernel_heap/info` - Kernel heap info