# Merry
The fast, flexible, and fun wiki tool.

## Writing Articles
Articles are written in a markdown-inspired markup langauge md2.
The language aims to be consistent, extendable, dead-simple to parse, 
and easy on the eyes.

Merry provides a language server which supports syntax highlighting,
refactoring, and auto-complete for `md2` files.

## Filesystem Structure
A wiki is described by a filesystem directory whose structure
mirrors the structure of the wiki itself. Every article corresponds
to an `md2` file located somewhere beneath the wiki's root directory.

A subdirectory containing a group of related articles is called a *category*.
Categories appear alongside articles in the parent index. The articles
within a category are accessible by the category-specific index which
is accessible from the category's parent index.

Every article file begins with a top-level heading. This heading declares
the *title* of the article. The *title* appears at the top article and
in the parent index. If two articles share the same title, a *disambiguation
index* is generated containing the list of identically-titled articles and
their short introductions.

Unless explicitly specified in an `intro.md2` file, categories are titled
by the name of their directory. If an `intro.md2` file is specified, then
the category's title becomes the top-level heading  of the `intro.md2` file.

Directories and files should be given abbreviated, easy-to-remember names, so as
not to interrupt the flow of text in the source document too much.

An `.snippet.md2` file can be embedded in article but it not an article itself.
Like article files, snippets should begin with a top-level heading. 
When a snippet is embedded in another article, its section tree
is relative to the expansion site.

## The md2 Language
Mostly md2 is a stricter markdown.

### Headings
Like markdown, headings  in md2 are declared by a newline followed by
some number of pound symbols. The number of leading pound symbols
indicates the relationship of a heading to its predecessor. 
A heading declaration has either...

1. The same number of pounds as its predecessor in which case it
begins a *sibling section*.
2. One more pound than its predecessor, in which case it is a *child section*.
3. Less pounds than its predecessor, in which case it is an *ancestor's subling section*.

It is an error to skip a level of nesting. Meaning the subsequent header *can not*
2 or more pounds. It can at most have 1 more pound.

### Directives
Directives are block level elements.

#### Embed
`embed` takes the path to a static asset and embeds it into the 
document at the invocation site.

```md2
| embed("fox.jpg")
```


#### Custom Directives
In the root directory of the wiki create a `directives.js` file.
Declare your macros as Javascript functions here.
There is an implicit `stream` property which is an HTML
async output stream. There is also a `srcpos` containing
information about the macro element invocation site. 
