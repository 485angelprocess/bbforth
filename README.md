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

## RISC-V Loader

The goal of this project now is as a user interface for a RISC-V based synthesizer. I started a forth assember in `lib/asm.fs` which can build risc instructions.

My core has a serial bridge which accepts functions with char prefixes. This allows me to read and write to ram, and to a debugger.

```
Load Asm definitions
> needs lib/asm
Instruction for adding an immediate (x21) with source register 0 into register 1
> x21 1 0 add_
Writes to location at memory index 0
> 0 writeaddr
```

The core has access to a uart driver which can send replies over serial. Sending a "v" function disables the bridge's uart replies so that it's easier to parse what's from the bridge and whats from the cpu.

```
: clearreg 0 swap dup andi_ ;
: d_writechar
    "v" puts
    0 clearreg 0 writeaddr
    1 clearreg 4 writeaddr
    x21 1 1 addi_ 8 writeaddr
    0 0 1 sw_ 12 writeaddr
    32 0 0 addi_ 16 writeaddr
    0 20 writeaddr ;
```

This loads a program into memory which clears register 0 and 1, places x21 into register 1 and then writes to address 0 with that values. For now address 0 is the uart tx register, so data written there gets sent to back out of the CPU. This is essentially a hello world.