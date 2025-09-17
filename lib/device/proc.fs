needs lib/asm

: obtain MUTEX read ;
: release 0 MUTEX write ;

: writekernel
    \ Initial jump
    512 zero jal_ 0 wm
    obtain
    \ Set front pointer to non value
    -1 FP wm
    \ clear heap
    512 HEAP wm
    release
;

14 MemoryAddress + const UARTPTR

UARTPTR zero t0 lw_ \ Load peripheral base 
0 t0 t1 lw_     \ Load size of uart
-8 zero t1 beq_ \ TODO make this go to next process
4 t0 t2 lw_     \ Load rx value of uart
8 t0 t2 sw_     \ Echo
-20 zero jal_   \ Go to next process
#= _ECHO