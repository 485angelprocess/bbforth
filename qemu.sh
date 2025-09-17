#!/bin/bash

PROGRAM=${1:-data.bin}

echo "Running QEMU with program $PROGRAM"

qemu-system-riscv32 -M virt -nographic -serial mon:stdio -bios none -device loader,addr=0x80000000,file=$PROGRAM

#qemu-system-riscv32 -M virt -nographic -serial /dev/null -bios none -device loader,addr=0x80000000,file=$PROGRAM -monitor stdio