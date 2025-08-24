needs lib/asm

\ andi x0, x0, 0
\ andi x1, x1, 0
\ addi x1, x1, v
\ sw x1, 0(x0)
: d_writechar
    "v" puts
    0 clearreg 0 writeaddr
    1 clearreg 4 writeaddr
    x21 1 1 addi_ 8 writeaddr
    0 0 1 sw_ 12 writeaddr
    32 0 0 addi_ 16 writeaddr
    0 20 writeaddr ;