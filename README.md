# bbforth

Small forth interpreter implemented in rust. Supports a small dictionary of built in words, and new words can be defined. 
Based on zforth.

Basic square

```rust
let mut ctx = Context::new();

let program = "5 dup * ." // push 5 to stack, duplicate, multiply and print top of stack

run_program(program, &mut ctx);
```

This program prints 25

Defining a word:

```rust
let mut ctx = Context::new();

run_program(": square dup * ;", &mut ctx); // define square
run_program("5 square .", &mut ctx); // use word 
```

This program also prints 25.

## Editor

I added a very simple editor with color highlighting.

![Screenshot 2025-05-21 125712](https://github.com/user-attachments/assets/0fda1884-bddf-4c60-a3e8-eb1e6a55817d)
