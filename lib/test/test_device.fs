needs prelude

\ Hardware in loop testing

\ (n n msg --> None if n==n, throws error if not)
: assert_equal abc_cab == swap assert ;

\ Check memory read write
\ Should be able to read and write to memory
: test_direct_rw 11 0 writeaddr 0 readaddr 11 "Memory read write" assert_equal ;

"Resetting" .
resets

"Test Direct RW" .
test_direct_rw
