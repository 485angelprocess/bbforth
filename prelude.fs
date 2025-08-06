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

256 const MemoryAddress
: read_delay 100 delay gets ;
: memaddr MemoryAddress + ;
: ids "I" puts read_delay displays ;
: writes "W" puts puts puts read_delay ;
: reads "R" puts puts read_delay ;

: writeaddr memaddr writes ;
: readaddr memaddr reads ;

"Connecting serial" .
115200 "/dev/ttyUSB2" serial_start

"Ran prelude" .
