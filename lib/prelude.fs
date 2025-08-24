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
: writes "W" puts puts puts ;
: reads "R" puts puts read_delay ;

: writeaddr memaddr writes ;
: readaddr memaddr reads ;

: writedebug debugaddr writes ;
: readdebug debugaddr reads ;

\ (n n msg --> None if n==n, throws error if not)
: assert_equal abc_cab == swap assert ;

\ Math for getting response
: slicedata dup 8 access swap dup 7 access swap dup 6 access swap 5 access ;
: 8to32 24 lshift swap 16 lshift + swap 8 lshift + + ;
: getdata slicedata 8to32 ;

: writedata writeaddr read_delay getdata ;
: readdata readaddr getdata ;

"Connecting serial" .
115200 serial_list 0 access serial_start

needs lib/device/control

"Ran prelude" .
