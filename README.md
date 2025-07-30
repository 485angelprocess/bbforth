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