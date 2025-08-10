needs lib/asm

\ Assemble
: test_andi 15 r1 r2 andi_ x00f0f113 "and immediate" assert_equal ;
: test_addi 15 r1 r2 addi_ x00f08113 "add immediate" assert_equal ;
: test_lui xABCDEF12 r2 lui_ xdef12137 "load upper immediate" assert_equal ;

"Assemble immediates" .
test_andi
test_addi

: test_addi 15 r1 r2 addi_ 
  0 writedata 0 readdata x00f08113
  "add immediate" assert_equal ;
  
test_addi

"Ok" .