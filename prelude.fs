\ Common macros

\ Stack operations
: ?dup dup if dup then ;
: tuck dup abc_cab ;

\ Basic math
: 2/ 2 / ;
: pow2 ( n -- n ) 1 swap lshift ;

\ Serial setup
: resets "x" puts ;
: rws puts 200 delay gets ;
: displays dup list_to_char . . ;

0 const DebugAddress
256 const MemoryAddress
: read_delay 100 delay gets ;

: memaddr MemoryAddress + ;
: debugaddr DebugAddress + ;

: ids "I" puts read_delay displays ;
: writes "W" puts puts puts read_delay ;
: reads "R" puts puts read_delay ;

: writeaddr memaddr writes ;
: readaddr memaddr reads ;

: writedebug debugaddr writes ;
: readdebug debugaddr reads ;

\ (n n msg --> None if n==n, throws error if not)
: assert_equal abc_cab == swap assert ;

"Connecting serial" .
115200 serial_list 0 access serial_start

"Ran prelude" .
