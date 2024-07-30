# Merry
Compiler for my markdown-esque markup language.

> [!CAUTION]
> This compiler is tested only on macOS. Some parts of the code assume that linebreaks
> are only a single character long. This will not work on Windows.

> [!CAUTION]
> This lanaguage and compiler are an experiment. It does work, but it is not nearly
> as tested as the mainstream markdown compilers. At this point in development, you
> should continue using standard markdown and a well-supported open-source markdown compiler
> instead. I have used [commonmark.js](https://github.com/commonmark/commonmark.js/) in the
> past, and it works well enough.
 
## Builing the Compiler
```
git clone https://github.com/duncpro/merry
git cd merry
cargo build --release
```
The executable binary will be created at `./target/release/merryc`.

## Compiling your first `.md2` source file
```
touch input.md2
merryc input.md2 output.html
open output.html
```

## Current Features
- Rich error messages complete with annotated source quotes visually indicating the source of the error.
- Produces semantic HTML `<section>` elements.
- Builtin linter which enforces consistent source text formatting.
