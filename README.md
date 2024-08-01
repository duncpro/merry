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
- Italicize, embolden, and underline text.
- Hyperlinks
- Unordered lists
- Paragraphs

## Differences with Markdown
- Unordered list declarator

    Unlike markdown which has a number of functionally equivalent unordered list item declarators,
    Merry has only one. That is the double hyphen. For example in Merry one would write...

    ```md2
    -- Is This It
    -- Room on Fire
    -- First Impressions of Earth
    ```
- Lists

    In Markdown a block is attached to a list item by indenting the subsequent lines +4 spaces.
    Instead the Merry compiler interprets list items as blocks themselves. 

    The following markup will be rendered as `<li><div><p>Is This It</p></div></li>`
    
    ```md2
    -- Is
       This
       It
    ```

    But if we do not indent +3 as the compiler expects, the list item will be broken.

    ```md2
    -- Is
    This
    It
    ````

    This markup will be rendered as `<li><div><p>Is</p></div></li><p>This It</p>`.

-
