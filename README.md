# :cityscape: tokyo

A monolithic kernel for the x86_64 architecture, written in pure Rust,
and mainly developed to help better understand the inner workings of both CPUs and operating systems.

## Features

- [x] Bootloader
- [x] Framebuffer
- [x] Text Rendering
- [x] Interrupts
- [x] Stack Switching
- [x] Hardware Interrupts
- [ ] Keyboard Input
- [ ] Paging
- [ ] Allocation
- [ ] Double Buffering
- [ ] Shell
- [ ] Multitasking
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

As of right now, the project uses these crates:

- [`bootloader`](https://github.com/rust-osdev/bootloader)
- [`ovmf_prebuilt`](https://github.com/rust-osdev/ovmf-prebuilt)
- [`x86_64`](https://github.com/rust-osdev/x86_64)
- [`pic8259`](https://github.com/rust-osdev/pic8259)
- [`font8x8`](https://gitlab.com/saibatizoku/font8x8-rs)
- [`line_drawing`](https://github.com/expenses/line_drawing) (with no_std fix by `andyblarblar`)
- [`bitvec`](https://github.com/ferrilab/bitvec)
- [`spin`](https://github.com/mvdnes/spin-rs)
- [`unchecked_index`](https://github.com/bluss/unchecked-index)

## License

This project is licensed under the [MIT License](https://opensource.org/license/mit).