# chip-8

Yet another Rust implementation of the [Chip-8](https://en.wikipedia.org/wiki/CHIP-8) system.

## Description

This is a basic implementation of the [Chip-8](https://en.wikipedia.org/wiki/CHIP-8) system, used as a "first-step" into making larger-scale emulators.

This implementation passes all test ROMs I could find, and plays games such as PONG / Tetris without issue. The only caveat is that the display to terminal can scroll / flash a bit, and my goal in this project was the emulation itself, not the nuance of terminal output.

## Running

Using [Cargo](https://crates.io/)...

```shell
cargo run PATH-TO-ROM
```

The keymap used is the left-hand-side of a standard QWERTY keyboard, but can be remapped in [src/main.rs](src/main.rs).

The default keymap is:

```
+---+---+---+---+
| 1 | 2 | 3 | 4 |
+---+---+---+---+
| Q | W | E | R |
+---+---+---+---+
| A | S | D | F |
+---+---+---+---+
| Z | X | C | V |
+---+---+---+---+
```

## Acknowledgements

- [Cowgod's Chip-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [tobiasvl's Awesome CHIP-8](https://github.com/tobiasvl/awesome-chip-8#testing)
