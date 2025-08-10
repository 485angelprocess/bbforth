x00 const r0
x01 const r1
x02 const r2
x03 const r3
x04 const r4
x05 const r5
x06 const r6
x07 const r7

\ imm rs rd code
\ immediate arithmetic
: imm_ swap 12 lshift + swap 7 lshift + swap 15 lshift + swap 20 lshift + ;
: imma_ b0010011 imm_ ;

\ Integer register immediate instructions
: addi_  b000 imma_ ;
: slti_  b010 imma_ ;
: sltiu_ b011 imma_ ;
: xori_  b100 imma_ ;
: ori_   b100 imma_ ;
: andi_  b111 imma_ ;

\ u type
: utype_ swap 7 lshift + swap 12 rshift 12 lshift + ;
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

: write_reply_off "v" puts ;
: write_reply_on "V" puts ;

: d_stop
  0 34 writedebug ;

: d_start
  "Starting" .
  write_reply_off
  d_stop
  0 33 writedebug
  1 32 writedebug ;