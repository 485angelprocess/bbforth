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

## Adding words

Words can be added as compilations of existing words

```
> ; square dup * ;
> 5 square .
25
```

## Lists

List data types can be containers of other data, such as ints or floats. There may be some way to implement better vector math, but currently are done pretty naively.

Push list to stack

```
> [1 2 3] .
[1 2 3]
```

Operation on a list with an int

```
> [1 2 3] 5 + .
[6 7 8]
```

Operation between lists

```
> [1 2 3] [4 5 6] + .
[5 7 9]
```

## Lazy Lists/Generators

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

## Loading files

Files can be loaded using the needs word. File `lib/math.fs` can be loaded:

```
needs lib/math
```

Forth files can contain definitions and functions. I may add some support for file caching, but currently files are just reloaded when asked for.

## UART Communication

I am using this as a interface method for my RISC-V softcore. Support is added for serial in/out.
