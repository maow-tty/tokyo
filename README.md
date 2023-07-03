# :cityscape: tokyo

**東京**

A monolithic kernel for the x86_64 architecture, written in pure Rust,
and mainly developed to help better understand the inner workings of both CPUs and operating systems.

**Notice:** The kernel does not currently support UEFI due to limitations in the code as well as unpatched bugs.
Feel free to contribute bug fixes, features will *not* be accepted though.

## Features

- [x] Bootloader
- [x] Framebuffer
- [ ] Text Rendering
- [x] Serial Logging
- [x] Interrupts
- [x] Stack Switching
- [x] Hardware Interrupts
- [ ] Keyboard Input
- [x] Paging
- [x] Bitmap Frame Allocator
- [x] Double Buffering
- [ ] Shell
- [ ] Multitasking
- [ ] Threading
- [ ] User Management
- [ ] Filesystem
- [ ] ELF Executables
- [ ] ACPI
- [ ] USB Devices
- [ ] Networking
- [ ] System Calls

## Acknowledgements

This wouldn't be possible with the help of Philipp Oppermann's [Writing an OS in Rust](https://os.phil-opp.com/) series, or
the massive amount of OS development resources provided by the [OSDev wiki.](https://wiki.osdev.org/)

## License

This project is licensed under the [MIT License](https://opensource.org/license/mit).