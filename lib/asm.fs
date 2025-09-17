x00 const zero
x01 const ra
x02 const sp
x03 const gp
x04 const tp
x05 const t0
x06 const t1
x07 const t2
x08 const s0
x08 const fp
x09 const s1
x10 const a0
x11 const a1

x12 const %eax

x18 const %wp
x19 const %sp
x20 const %se
x21 const %sm

: noop_ 0 ;

\ imm rs rd f3 code
\ immediate arithmetic
: imm_ 
    swap b111 & 12 lshift + \ f3
    swap x1F & 7 lshift + \ rd 
    swap x1F & 15 lshift + \ rs
    swap xFFF & 20 lshift + \ imm
;
: imma_ b0010011 imm_ ;

\ Integer register immediate instructions
: addi_  b000 imma_ ;
: slti_  b010 imma_ ;
: sltiu_ b011 imma_ ;
: xori_  b100 imma_ ;
: ori_   b100 imma_ ;
: andi_  b111 imma_ ;

\ imm rs rd
: lw_  b010 b0000011 imm_ ;
: lh_  b001 b0000011 imm_ ;
: lb_  b000 b0000011 imm_ ;

\ r type
\ s2 s1 d1 f op
: rtype_ 
    swap 12 lshift + \ function
    swap 7 lshift +  \ d1
    swap 15 lshift + \ s1
    swap 20 lshift + \ s2
;

: rarith_ b0110011 rtype_ ;

\ Register arithmetic
: add_ b000 rarith_ ;

\ u type
: utype_ swap 7 lshift + swap 12 lshift + ;
\ imm rd --> inst
: lui_   b0110111 utype_ ;
: auipc_ b0010111 utype_ ;

\ s type opcode
\ split immediate int --> lower upper
: simm_ dup b11111 & swap 5 rshift b1111111 & ;
: stype_ 
      12 lshift + \ function code 
      swap 20 lshift + \ rs2
      swap 15 lshift + 
      swap simm_ 
      25 lshift 
      swap 7 lshift + + ;
\ store word imm s1 s2 --> instruction 
: sw_ b0100011 b010 stype_ ;
: sh_ b0100011 b001 stype_ ;
: sb_ b0100011 b000 stype_ ;

\ Special
: ecall_ b1110011 ;

\ offset rs1 rd
: jalr_ 7 lshift b1100111 + 
    swap 15 lshift +
    swap 20 lshift +
    ;

: abc_bca
  abc_cab
  abc_cab
;

\ offset -> mapped value
: jal_imm_map_ 
  dup
  12 rshift b11111111 & \ 19:12
  swap
  dup
  11 rshift b1 & \ 11
  8 lshift
  abc_bca
  +
  swap dup
  1 rshift b1111111111 & \ 10:1
  9 lshift
  abc_bca
  +
  swap
  20 rshift b1 & 19 lshift
  +
;
\ offset rd
: jal_ 
  7 lshift
  b1101111 +
  swap
  jal_imm_map_
  12 lshift
  +
;

: branch_imm_map_
  dup 11 rshift x1 & 7 lshift
  swap
  dup 1 rshift x1F & 8 lshift
  swap
  dup 5 rshift x3F & 25 lshift 
  swap
  12 rshift x1 & 31 lshift
  +
  +
  +
;

\ imm s1 s2 f3 
: branch_
  b1100011
  swap b111 & 12 lshift + \ f3
  swap x1F & 15 lshift + \ s2
  swap x1F & 20 lshift + \ s1
  swap branch_imm_map_
  +
;

: beq_ b000 branch_ ;

: clearreg 0 swap dup andi_ ;

: write_reply_off "v" puts ;
: write_reply_on "V" puts ;

: d_reset
  1 34 writedebug ;

: d_stop
  0 32 writedebug ;

: d_start
  write_reply_off
  d_stop
  d_reset
  1 32 writedebug ;
  
: readpc
  33 readdebug getdata ;
  
: writepc
  33 writedebug ;
  
: readreg
  readdebug getdata ;
  
: readen
  32 readdebug getdata ;
  
: goto% zero jal% ;
  
\ TODO have this load from regs not from hardcoded here
40 const MUTEX
48 const LOCK \ may have lock not in ddr3
56 const HEAP
60 const FP

256 const UART

: wm MemoryAddress + write ;
: rm MemoryAddress + read ;

: helloworld
  512 zero jal_ 0 wm
  x10000000 UART wm
  512 HEAP wm \ allocated bytes essentially
  `start
  x80000 t0 lui_
  UART t0 t0 addi_
  0 t0 t0 lw_
  62 zero t1 addi_ \ '>'
  0 zero zero andi_
  0 t0 t1 sb_
  8 zero s0 addi_
  4 t0 t1 lw_
  `loop
  1 t1 t2 andi_ \ loop:
  48 t2 t2 addi_
  0 t0 t2 sb_
  1 t1 t1 sltiu_
  -1 s0 s0 addi_
  8 zero s0 beq_ \ break
  `loop goto%  \ goto loop
  68 zero t1 addi_
  0 t0 t1 sb_
  `end
  `end goto% \ loop at end
;

"loading hello" .
helloworld stack_to_list #= hello
"data.bin" write_bin

\ TODO clean up this thing
\ should be one function
: push
  0 %sp %eax sw_
  -4 %sp %sp addi_
;
push #= &push

: pop
  4 %sp %sp addi_
  0 %sp %eax lw_
;
pop #= &pop