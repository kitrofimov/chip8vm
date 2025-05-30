![banner](.github/banner.png)

![demo](.github/demo.gif)

<div align="center">

![build_status](https://img.shields.io/github/actions/workflow/status/kitrofimov/chip8vm/rust.yml)
![license_mit](https://img.shields.io/badge/License-MIT-blue.svg)
![language_rust](https://img.shields.io/badge/language-rust-B7410E)

</div>

# CHIP-8 Toolchain

- Interpreter:
    - full support for original CHIP-8 instructions (no "quirks"!)
    - graphical rendering using SDL2
- Assembler:
    - consistent & well-defined syntax
    - user-friendly error messages
    - all instructions & useful directives
- Disassembler:
    - translates bytecode to assembler instructions
    - makes it easier to decode ROMs 

## TODO (if I ever want to take on this project again)
- [ ] Multiple versions of CHIP-8 (quirks?)
- [ ] Maybe, an IDE for this thing? Something like [this](https://internet-janitor.itch.io/octo) or [this](https://github.com/gulrak/cadmium)?
- [ ] [This](https://github.com/glouw/c8c) looks like fun

## Building
### Requirements
- Rust (latest stable version recommended)
- SDL2 development libraries

### Steps
1. Install Rust: [https://www.rust-lang.org/](https://www.rust-lang.org/)
2. Install SDL2 development libraries:
    - Debian/Ubuntu: `sudo apt install libsdl2-dev`
    - macOS: `brew install sdl2`
    - Windows: Use [vcpkg](https://github.com/microsoft/vcpkg) or download SDL2 manually.
3. ```bash
   git clone https://www.github.com/kitrofimov/chip8vm
   cd chip8vm
   cargo build --release
   ```
4. Find the binaries in `target/release`!

## Usage
```bash
interpreter <rom.ch8>
assembler <input.asm> <output.ch8>
disassembler <input.ch8> <output.asm>
```

## Key Mapping
The CHIP-8 keypad is mapped to the keyboard as follows:
```
CHIP-8:  1 2 3 C       Keyboard:  1 2 3 4
         4 5 6 D                  Q W E R
         7 8 9 E                  A S D F
         A 0 B F                  Z X C V
```

## Acknowledgments
- `Timendus` for his wonderful [ROM test suite](https://github.com/Timendus/chip8-test-suite)
- `gulrak` for the [opcode table](https://chip8.gulrak.net/)
- `mittmikolay` and `tobiasvl` for their technical references ([this](https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Technical-Reference) and [this](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/))
- `Cowgod` for assembler syntax [reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
