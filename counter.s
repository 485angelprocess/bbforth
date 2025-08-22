; pointer to start of generator

; Counter is a struct
; 0 - counter
; 1 - rate
; 2 - loop
COUNTER:
pop ;
lw x10, 0(xSTACK) ; counter
lw x11, 1(xSTACK) ; rate
lw x12, 2(xSTACK) ; loop
add x10, x10, x11 ; get new value
blt x10, x12, 12
andi xSTACK, x0, 0
push
ret
addi xSTACK, x10, 0
push
ret

VCA:
pop ; address into usable register say x5
jalr xRET, 0(xSTACK) ; jump to get value of input

MAIN:
; push pointer of source to stack
push &INPUT
; run vca