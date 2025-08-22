needs lib/asm

\ Define in riscv context
: # 1 library_set : ;
: esc 0 library_set ;

\ Writing to device:
\ [1 2] [3 4] ["W" "W"] list_group list_group puts

\ (len addr) -> (list)
: program_offset natural 4 * + collect to_int ;
\ (stack addr) -> ()

0x04 const IMM_BUFFER_ADDRESS

256 const KERNEL_START

\ kernel
: load_kernel
    "v" puts
    KERNEL_START 0 1 jalr_ 0 write_addr
    1 clearreg            KERNEL_START writeaddr
    0 clearreg          4 KERNEL_START + writeaddr
    x21 1 1 addi_       8 KERNEL_START + writeaddr
    0 0 1 sw_          12 KERNEL_START +  writeaddr
    KERNEL_START 0 1 jalr_ 16 KERNEL_START + writeaddr
;