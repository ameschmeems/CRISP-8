# CRISP-8
CRISP-8 is a CHIP-8 language interpreter built in rust.
CHIP-8 is commonly used as a "Hello World" project, for devs entering the world of emulator programming.
My main source of information was [this article](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/).
The project is structured with a crisp8_core library crate, containing the backend engine for the emulator, and a frontend desktop binary crate, using sdl2.

## Potential improvements
I currently have no plans on updating this project as it was only meant as a weekend thing, but I could see myself potentially adding a web frontend, working sound, or configuration for more ambiguous opcodes (if I have nothing better to do).
