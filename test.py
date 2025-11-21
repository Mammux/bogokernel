import subprocess
import time
import sys

def run_test():
    """Run BogoKernel with automated input and capture output"""
    
    print("Starting QEMU...")
    
    # Start QEMU process
    proc = subprocess.Popen(
        [
            'qemu-system-riscv64',
            '-machine', 'virt',
            '-m', '128M',
            '-nographic',
            '-bios', 'default',
            '-kernel', 'target/riscv64gc-unknown-none-elf/debug/kernel'
        ],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=0  # Unbuffered
    )
    
    output_lines = []
    
    def read_until_prompt(timeout=5):
        """Read output until we see the shell prompt"""
        start = time.time()
        buffer = ""
        while time.time() - start < timeout:
            try:
                char = proc.stdout.read(1)
                if not char:
                    break
                buffer += char
                output_lines.append(char)
                sys.stdout.write(char)
                sys.stdout.flush()
                
                # Check for prompt
                if buffer.endswith("> "):
                    return True
            except:
                break
        return False
    
    try:
        # Wait for initial boot and shell prompt
        print("Waiting for shell...")
        if not read_until_prompt(timeout=10):
            print("\nERROR: Shell prompt not found")
            proc.kill()
            return False
        
        # Send "hello" command
        print("\nSending 'hello' command...")
        proc.stdin.write("hello\n")
        proc.stdin.flush()
        time.sleep(2)
        
        # Read output
        read_until_prompt(timeout=5)
        
        # Send "shutdown" command
        print("\nSending 'shutdown' command...")
        proc.stdin.write("shutdown\n")
        proc.stdin.flush()
        time.sleep(2)
        
        # Wait for process to end
        proc.wait(timeout=5)
        
    except subprocess.TimeoutExpired:
        print("\nQEMU did not shut down, killing...")
        proc.kill()
    except KeyboardInterrupt:
        print("\nInterrupted, killing QEMU...")
        proc.kill()
    
    # Save output to file
    full_output = ''.join(output_lines)
    with open('test_output.txt', 'w') as f:
        f.write(full_output)
    
    print("\n\n=== Test Results ===")
    
    # Check for expected strings
    tests_passed = 0
    tests_total = 0
    
    tests = [
        ("Shell loaded", "Welcome to BogoShell!"),
        ("Hello app ran", "Hello from C World!"),
        ("Shutdown initiated", "Shutting down..."),
    ]
    
    for test_name, expected_string in tests:
        tests_total += 1
        if expected_string in full_output:
            print(f"[PASS] {test_name}")
            tests_passed += 1
        else:
            print(f"[FAIL] {test_name} - expected '{expected_string}'")
    
    print(f"\nPassed {tests_passed}/{tests_total} tests")
    print("Full output saved to test_output.txt")
    
    return tests_passed == tests_total

if __name__ == "__main__":
    success = run_test()
    sys.exit(0 if success else 1)
