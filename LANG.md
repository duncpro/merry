# MD2
A structured markup language that is (1) dead-simple to parse,
(2) easy on the eyes, and (3) easy to write. 

## Directives
A **directive** is a command declared within a md2 source file,
that is executed during compilation.

The merry compiler has two built-in directives, namely `link` and `embed`,
however any program using the merry compiler as a library can easily
introduce more directives.

### `link`
The `link` directive expands qualified phrases to a hyperlinks.
```md2
The fox is a [mammal]{1}.

| link 1 /eco/taxonomy/mammal.md2
```

### `embed`
The `embed` directive embeds a rich asset file into the document at its invocation site.
The `embed` directive's most common use-cases is to embed images. 

```md2
| embed "red_fox.jpg"
````

An application using the merry compiler as a library can add support for other file types as well.

### Directive Syntax
An *n*-indented line beginning with a vertical bar is interpreted as a directive invocation. 

```text
| <command name> [args...]
```

Arguments are separated by spaces. If an argument contains a space it must be surrounded
by double quotes.

