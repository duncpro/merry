# Merry
Compiler for my markdown-esque markup language.

> [!CAUTION]
> This compiler is tested only on macOS. Some parts of the code assume that linebreaks
> are only a single character long. This will not work on Windows.

> [!CAUTION]
> This lanaguage and compiler are an experiment. It does work, but it is not nearly
> as tested as the mainstream markdown compilers. I recommend using 
> [commonmark.js](https://github.com/commonmark/commonmark.js/). Furthermore, this language
> deviates from markdown in fundamental ways both syntactically and semantically.

## Usage
```
cargo run input.md2 output.html
```

