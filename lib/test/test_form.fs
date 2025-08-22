needs lib/test/test_lib

10 = a

a 10 "assignment works ok" assert_equal

\ Let's start with a variable
{
    :var 10
} = form

form.var 10 "variables are assignable" assert_equal