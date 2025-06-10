# About

This is a WonderSwan emulator I coded in Rust as an exercise. It is not very accurate and has very poor framerates.
Only some games are capable of running at a playable framerate and with no major glitches.
This project might be of interest to people working on similar projects, or as a base for a more accurate and better optimized emulator.

Special thanks to the SDL team.

# Running

The executable is meant to run from command-line. The first argument will be the name of the ROM file, if one is not provided the emulator will instead run a ROM made of all 0s.

The second argument can be either mute, which mutes the emulator or trace, in which case the CPU will print out a trace in addition to the program being muted.

# Resources used in testing, research or debugging:

[WSDev Wiki](https://ws.nesdev.org/wiki/WSdev_Wiki)

[WonderSwan - Sacred Tech Scroll](http://perfectkiosk.net/stsws.html)

[WonderSwan CPU test](https://github.com/FluBBaOfWard/WSCPUTest)

[WonderSwan test suite](https://github.com/asiekierka/ws-test-suite)

[Mesen](https://www.mesen.ca/)

[Ares](https://ares-emu.net/)
