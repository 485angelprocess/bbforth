esc

x00 const zero
x01 const ra
x02 const sp
x03 const gp
x04 const tp
x06 const t1
x07 const t2

: noop_ 0 ;

\ imm rs rd code
\ immediate arithmetic
: imm_ swap xFFF & 12 lshift + swap 7 lshift + swap 15 lshift + swap 20 lshift + ;
: imma_ b0010011 imm_ ;

\ Integer register immediate instructions
: addi_  b000 imma_ ;
: slti_  b010 imma_ ;
: sltiu_ b011 imma_ ;
: xori_  b100 imma_ ;
: ori_   b100 imma_ ;
: andi_  b111 imma_ ;

: lw_  b010 b0000011 imm_ ;

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
  
: readreg
  readdebug getdata ;
  
: readen
  32 readdebug getdata ;