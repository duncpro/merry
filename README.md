# Merry
Compiler for my markdown-esq markup language.

## Caveats
- This is only tested on macOS and I doubt it will work on Windows as there
  are parts of the code that rely on the fact that newline delimiters are 1 byte
  long. 
- This is very much a work in progress. There are likely some bugs I've yet to find.
  However it is functional in a barebones sense, and in its current state it can compile
  an md2 file into an HTML file.

## Usage
```
cargo run input.md2 output.html
```
