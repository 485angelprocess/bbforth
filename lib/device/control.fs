needs lib/asm

\ Define in riscv context
: # 1 library_set : ;
: esc 0 library_set ;
: riscv 1 library_set ;
: local 0 library_set ;

\ Writing to device:
\ [1 2] [3 4] ["W" "W"] list_group list_group puts

\ (len addr) -> (list)
: program_offset natural 4 * + collect to_int ;
\ (program_n..program_0 offset) -> (0)
\ Uploads program from current stack
\ Uses the top of the stack as the program offset
: upload_program 
    stack_to_list dup 0 access
    MemoryAddress +
    swap remove_from_list dup len 
    abc_cab abc_cab program_offset 
    dup len "W" repeat 
    list_group
    list_group
    puts
    ;

x04 const IMM_BUFFER_ADDRESS

256 const KERNEL_START
264 const IMM_START
32 const IMM_LOCK
256 const MEM

128 const IMM_START

1024 const SP_BASE

\ Program starts here and jumps to offset
\ This area could be used for interrupt vector
: load_startup
    KERNEL_START 0 1 jalr_ \ init
    IMM_START 0 1 jalr_ \ run immediate
    0
    upload_program
;

\ kernel
: load_kernel
    d_reset
    d_stop
    clear
    load_startup \ load first jump
    \ Start of main loop
    zero clearreg \ clear zero register
    \ Load number of immediate values
    \ TODO have this be an initializer
    SP_BASE MemoryAddress + zero sp addi_
    MEM x0 t1 addi_
    IMM_START t1 t1 addi_
    0 t1 t2 lw_
    x0 t1 ra jalr_ \ go to immediate program
    IMM_LOCK x0 t1 addi_
    0 t1 zero sw_ \ clear imm lock
    0 \ noop
    KERNEL_START upload_program
;

load_kernel
"Loaded kernel" .

: run_program
    0
    512 upload_program
    512 4 IMM_START + writeaddr \ Store number
    d_reset
    
    4 writepc \ Set to immediate vector table
    d_start
;

# run d_start ;
# stop d_stop ;

\ TODO pop and push
# push_
    0 sp t1 sw_
    -4 sp sp addi_
;

# pop_
    4 sp sp addi_
    0 sp t1 lw_
;

# pushint_
   zero t1 addi_
    push_
;

# ex
    run_program
;

# st
    pop_
    0 zero zero addi_ \ no op needed? have to check what's going on here
    10 zero t1 sw_
    run_program
    10 delay
    gets
    0 access
    .
;

esc