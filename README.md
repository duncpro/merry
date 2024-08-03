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
- Monospace Code Block
- Monospace Code Span

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

- Italics

    In Markdown text is italicized by surrounding it with asterisks.
    ```md
    This is *italicized* in markdown.
    ```

    In Merry text is italicized by surrounding it with tildes.
    ```md2
    This is ~italicized~ in Merry.
    ```

- Bold

    In Markdown text is emboldened by surrounding it with double asterisks.
    ```md
    This is **emboldened** in markdown.
    ```

    In Merry text is emboldened by surrounding it with asterisks.
    ```md2
    This is *emboldened*.
    ```
    
- Hyperlinks

    In Merry there is no specific language construct for hyperlinks. Instead, hyperlinks are
    achieved using two language features. Namely, qualified spans, and directives.

    ```md2
    [Google]{1} is the most popular search engine, but I prefer [DuckDuckGo]{2} myself.
    
    | href 1 https://google.com
    | href 2 https://duckduckgo.com
    ````

    The `href` directive expands all spans which are qualified with a given tag into hyperlinks
    pointing to a given url.

    There is no way to write a hyperlink inline like in Markdown.

- Code Blocks

    In Merry the triple backtick block is a generic escaped block of text called a *verbatim*.
    By default verbatims are rendered similarly to paragraphs. The only difference being that
    one can use symbols that would otherwise have a special meaning, such as asterisks, tildes,
    etc. However, like bracketed spans, *verbatims* can be qualified with tags. The builtin `m`
    tag will rewrite the verbatim to be a generic monospace code block.

    ````md2
    ```
    println!("Hello World");    
    ```{m}
    ````

- Inline Code

    In Merry the backtick span is a generic escaped block of text called an *inline verbatim*.
    By default these are rendered just like any other text. The only difference being that
    one can use symbols that would otherwise have a special meaning, such as asterisks, tildes, etc.
    However, *inline verbatims* can be qualified with tags. The builtin `m` tag will rewrite
    the inline verbatim to be a generic monospace code span.

    ```md2
    Consider using `std::mem::swap`{m} instead of cloning when possible.
    ```

- Mathematics

    Merry does not support mathematics out of the box unlike some popular flavors of Markdown.
    However, it is possible to typeset math via an external tool like [KaTeX](https://katex.org),
    using the builtin `rewrite` directive.

    ````md2
    ```
    a^2 + b^2 = c^2
    ```{math}

    | rewrite math npx katex -d -F mathml
    ````

    The `rewrite` directive replaces all verbatim blocks marked with a given tag,
    by piping the contents to an external process and then piping that process' output
    into the finished file.

    The external process should produce valid HTML.
