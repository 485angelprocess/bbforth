####################
## Entry point #####
####################
_start:
andi zero, zero, 0
lui gp, 0x80000 # Memory base
lw t1, 36(gp)
andi t1, t1, 1
bne t1, zero, main
# Setup
lui  sp, 0x100 # Size of memory
add sp, gp, sp
sw  sp, 56(gp) # Store location of immediate stack pointer
addi t1, zero, 1
sw   t1, 36(gp) # set init flag
main:
addi t1, zero, 1
lw   t1, 48(gp) # load immediate flag
beq  t1, zero, skipimm # If flag set

# Run immedatiates
runimmediate:
lw    t1, 52(gp) # load immediate function from register
lw    sp, 56(gp) # Load location of stack pointer
jalr  ra, 0(t1) # run immediate function
jalr  zero, 4(ra) # return
lw    zero, 48(gp) # clear immediate flag

# TODO: run buffer functions like video or audio

skipimm:
lw t1 36(gp)
andi t1, t1, 2 # Check if loop flag is set
bne  t1, zero, end
j _start # Loop to start of kernel
end:
ecall # otherwise end