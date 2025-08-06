44100 const SampleRate
: freqtosample SampleRate swap / ;

\ amp freq phase square
: normalramp natural + swap freqtosample dup swap abc_cab % swap tofloat / ;
: ramp normalramp swap tofloat * ;
: square normalramp swap tofloat swap 0.5 > * ;