# RustyChip
A CHIP-8 emulator made in Rust. Used as a learning project to start developing in Rust.

## Prerequisites
This project uses SDL2 for the display. Please ensure that you have followed the steps provided [for that crate](https://crates.io/crates/sdl2#user-content-requirements) before building.
Since I developed this on Windows using WSL and RustRover, there were two ways to go about it.

### WSL
Please keep in mind that this will result in a Linux-style for the window.
- Set RustRover to use the WSL toolchain and standard library.
- Follow [the instructions]() for Linux.
  - Run `sudo apt-get install libsdl2-dev` to get the libraries needed.
    - You can also copy the files yourself to the toolchain folder but this is nicer.

### Windows
- Set RustRover to use the Windows toolchain and standard library.
- Follow [the instructions](https://crates.io/crates/sdl2#user-content-windows-msvc) for Windows.
  - [Download](https://github.com/libsdl-org/SDL/releases/latest) the latest VC development release.
  - Extract the zip.
  - Copy all of the `.lib` files from `SDL2-devel-2.0.x-VC\SDL2-2.0.x\lib\x64\` to `C:\Users\{Your Username}\.rustup\toolchains\{current toolchain}\lib\rustlib\{current toolchain}\lib`.

## Running
As expected, the standard `cargo` command are all that's necessary. To simply get it up and running, run `cargo run`.

## Controls
Aside from the actual game controls, you may close the window or press ESC to stop the emulator.

When it comes to the game controls, I have put the mapping I used down below, but each game has its own controls and I'm sad to say your guess is as good as mine there.

### Original CHIP-8
|     |     |     |     |
|:---:|:---:|:---:|:---:|
|  1  |  2  |  3  |  C  |
|  4  |  5  |  6  |  D  |
|  7  |  8  |  9  |  E  |
|  A  |  0  |  B  |  F  |

### Keyboard Mapping
|     |     |     |     |
|:---:|:---:|:---:|:---:|
|  1  |  2  |  3  |  4  |
|  Q  |  W  |  E  |  R  |
|  A  |  S  |  D  |  F  |
|  Z  |  X  |  C  |  V  |

## Games
I have included the public domain games which I could find in a directory in the project.  The file picker should automatically start inside there.  Have fun!

If I am mistaken and any games within are not a part of the public domain, please let me know and I will take them down immediately.

## Reference Material
[Wikipedia CHIP-8 Page](https://en.wikipedia.org/wiki/CHIP-8)  
[Cowgod's CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)

