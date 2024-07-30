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
> past, and it works well-enough.
 
## Usage
```
cargo run input.md2 output.html
```

