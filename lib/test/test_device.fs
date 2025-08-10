needs prelude

\ Check memory read write
\ Should be able to read and write to memory
: test_direct_rw 11 0 writedata 0 readdata "Memory read write" assert_equal ;

: test_debug_rw 11 0 writedebug getdata 0 readdebug getdata "Debug writing should be reflected" assert_equal ;

"Resetting" .
resets

"Test Direct RW" .
test_direct_rw
test_debug_rw
