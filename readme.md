# Rust Router
An argument parsing library that makes it easy to model your API as a hierarchy. The goal is to be easy to reason about and have the absolute minimum amount of overhead at runtime.

## About
A CLI program consists of options and operands. Most programs have a concept of "commands", which are just operands that perform an action. This library structures your program as a tree of "segments", like path segments separated by spaces, where the end of the path represents an action to perform.

## Goals of 1.0
[x] Support invalid UTF-8
[ ] Does it handle, or at least not hinder, the requirements of a CLI application
  - Non-interactive programs
  - TUI programs
  - Continuous input
  - Asynchronous programs
  - etc.
[ ] Flexible documentation output that can be formatted as help text, a shorter "tl;dr" option, a MAN page, Web page output, etc.
  - The core library needs just enough to facilitate it, while the rest would likely be an add-on
  - It'd be nice if it can use compile-time checking of name references and other things that could get out of sync with code
[ ] A validation strategy that can handle as many constraints as the program needs
[ ] Smaller than other options while offering the important features, and perhaps having other features as add-ons
[ ] Allow programs that can be extended, similar to Cargo
  - Might be achieved with *Path Parameters*, but what if people want to override existing commands?
  - How can an extension's documentation interact with the host program?
[x] Single-hyphon-long-options (optional)
   Allows some old programs to be built with this library without breaking the API. Also, there shouldn't be any implicit style restrictions on parsing
[x] Support optional '=' separator to Separate options from option-arguments (Optional)... I've never said "option" so many times..

## Options
Options will have a full name prefixed with "--", and can optionally be aliased by a shorthand that's prefixed with "-". These prefixes are parsed out before they're made available to your program.

> Note: When a shorthand is given as an argument, it is resolved to its full name. You won't know the shorthand was used instead of the full name.

### The Hyphen-Only Argument (`-`)
While this character is used as a prefix for option shorthands, it can also be given as an option-argument, or as an operand. The convention is for programs to check for this when they expect a filename as an option's value or an operand, which means, "read from stdin instead of a file". But, it's just a convention. This library interprets it as a regular argument.

### Rule Groups
You don't have to specify which options a segment expects, but when you do you put them in `OptGroup`s.

`OptGroup`s allow specifying for each segment:
- A number of options allowed, and whether one is required or they're all optional
- A number of options allowed, but are mutually exclusive

If groups are specified, unknown options will display an error.

If no groups are specified, unknown options will be ignored.

## Path Parameters
When a segment is defined with a ':' prefixing its name, it will match any string passed to it. They can then be used in the action.

## Limits
**Segments** - A router can have up to `u16::MAX` segments.

**Tree Depth** - The deepest nesting level a tree of segments can be is 16. If you have an example of a real-world API that goes beyond that I would be interested to hear about it, and consider bumping the limit.

**Options**- A router can have up to `u16::MAX` options.

**Option Groups** - A router can have up to 8,190 `OptGroup`s, and a segment can have up to 15.