needs prelude

\ Hardware in loop testing

\ (n n msg --> None if n==n, throws error if not)
: assert_equal abc_cab == swap assert ;

\ Math for getting response
: slicedata dup 8 access swap dup 7 access swap dup 6 access swap 5 access ;
: 8to32 24 lshift swap 16 lshift + swap 8 lshift + + ;
: getdata slicedata 8to32 ;

: writedata writeaddr getdata ;
: readdata readaddr getdata ;

\ Check memory read write
\ Should be able to read and write to memory
: test_direct_rw 11 0 writedata 0 readdata "Memory read write" assert_equal ;

: test_debug_rw 11 0 writedebug getdata 0 readdebug getdata "Debug writing should be reflected" assert_equal ;

"Resetting" .
resets

"Test Direct RW" .
test_direct_rw
test_debug_rw
