# bbforth

Small forth interpreter implemented in rust. Supports a small dictionary of built in words, and new words can be defined. 
Based on zforth, as well as Charle Moore's book on problem-based programming.

## Running

Runs in terminal as an interactive prompt. Example:

```
> 5 3 + .
8
```

The dictionary is compiled at run time. I'm putting together a standard library, but the goal is to be able to use the same framework for multiple hardware/software targets. A small number of types with a extensible dictionary should make this possible.

## Lazy Lists

Adding in support for lazy lists as data types. The basic support can be done by adding the natural numbers to the the stack.

```
> natural .
{0, 1, 2, 3, 4, 5, 6, 7, 8,  ... }
```

Values are calculated as needed. Operations can be done on lazy lists/generators:

```
> natural 5 + .
{5, 6, 7, 8, 9, 10, 11, 12, 13,  ... }
```