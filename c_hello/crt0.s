.section .text.entry
.global _start
_start:
    # Setup stack pointer if needed (kernel usually provides it)
    # Call main
    call main
    # Exit with return value from main
    mv a0, a0
    call exit
    # Should not reach here
    wfi
