# libc/src/crt0.s - C runtime startup code for RISC-V
.section .text.entry
.global _start
_start:
    # a0 = argc
    # a1 = argv
    # a2 = envp
    # Stack pointer is already set by kernel
    
    # Call main(argc, argv, envp)
    call main
    
    # Exit with return value from main
    # a0 already contains return value
    call exit
    
    # Should not reach here
1:  wfi
    j 1b
