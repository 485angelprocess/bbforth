

: log2mod ( n -- mod n )
  ?dup not if abort"log2 can't compute 0" then
  dup 32 0 do ( orig n ) 2/ ?dup not if i break then loop ( orig n )
  tuck pow2 - swap ;
: log2 ( n -- n ) log2mod nip ;
: log2# ( n -- n ) log2mod swap if abort"not a power of 2" then ;

: roundup ( n div -- n ) tuck /+ * ;
: rounddown ( n div -- n ) tuck / * ;

\ Heron's method
: isqrt ( n -- n )
  dup 1 <= if exit then
  r! 2/ begin dup r@ over / + 2/ ( hi lo )
  2dup > while nip repeat rdrop drop ;

code abs ( n -- n ) 0 i) s<) if, 0 i) swap-, then exit,