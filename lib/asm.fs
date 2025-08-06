\ imm rs rd code
\ immediate arithmetic
: imm swap 12 lshift + swap 7 lshift + swap 15 lshift + swap 20 lshift + ;
: imma b0010011 imm ;

\ Integer register immediate instructions
: addi  b000 imma ;
: slti  b010 imma ;
: sltiu b011 imma ;
: xori  b100 imma ;
: ori   b100 imma ;
: andi  b111 imma ;

\ Special
: ecall b1110011 ;