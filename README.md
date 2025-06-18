# CHIP-8 Emulator

A barebones CHIP-8 emulator, built in Rust. Most of the code is based on [this
CHIP-8 book](https://github.com/aquova/chip8-book). 

## Building and Running

The codebase requires the 2024 edition of Rust. The windowing/graphics use
SDL2, which needs to be [installed
beforehand](https://github.com/Rust-SDL2/rust-sdl2?tab=readme-ov-file#sdl20-development-libraries).

```
$ git clone https://github.com/arnavb/chip8_emu.git
$ cd chip8_emu
$ cargo build
$ cargo run path/to/rom
```

## Notes

Most of the instructions here follow [Cowgod's
specification](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM), though there
may be some discrepancies. Sound isn't supported.
