# LC-3_rs
[LC-3](https://en.wikipedia.org/wiki/LC-3) virtual machine implementation in Rust. Currently it should be able to run all programs written for LC-3 but I haven't tested it with other programs than [2048 game](https://justinmeiners.github.io/lc3-vm/supplies/2048.obj).

# TODO
* Get rid of all c functions and unsafe blocks
* Better abstraction on memory

# Usage 
`<program name> [file1] ...` \
Run the program with the desired `.obj` file as argument

# Crates
## enum-primitive-derive
Allows using enums in c style
## libc
I was lazy and used c functions to implement parts that are not related to emulation

# Credits
This project followed this [tutorial](https://justinmeiners.github.io/lc3-vm/).
