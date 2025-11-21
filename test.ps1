# PowerShell test script for BogoKernel
# Runs the kernel with automated input and validates output

Write-Host "Starting QEMU with automated test..." -ForegroundColor Cyan

# Create a temporary script file for sending commands
$commands = @"
hello
shutdown
"@

# Start QEMU process
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = "qemu-system-riscv64"
$psi.Arguments = "-machine virt -m 128M -nographic -bios default -kernel target\riscv64gc-unknown-none-elf\debug\kernel"
$psi.UseShellExecute = $false
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true

$process = New-Object System.Diagnostics.Process
$process.StartInfo = $psi

# Event handlers for output
$outputBuilder = New-Object System.Text.StringBuilder
$errorBuilder = New-Object System.Text.StringBuilder

$outputHandler = {
    if (-not [String]::IsNullOrEmpty($EventArgs.Data)) {
        $Event.MessageData.AppendLine($EventArgs.Data)
        Write-Host $EventArgs.Data
    }
}

$errorHandler = {
    if (-not [String]::IsNullOrEmpty($EventArgs.Data)) {
        $Event.MessageData.AppendLine($EventArgs.Data)
        Write-Host $EventArgs.Data -ForegroundColor Red
    }
}

$outputEvent = Register-ObjectEvent -InputObject $process -EventName OutputDataReceived -Action $outputHandler -MessageData $outputBuilder
$errorEvent = Register-ObjectEvent -InputObject $process -EventName ErrorDataReceived -Action $errorHandler -MessageData $errorBuilder

try {
    $process.Start() | Out-Null
    $process.BeginOutputReadLine()
    $process.BeginErrorReadLine()
    
    # Wait for boot
    Write-Host "Waiting for shell to load..." -ForegroundColor Yellow
    Start-Sleep -Seconds 3
    
    # Send commands
    Write-Host "Sending 'hello' command..." -ForegroundColor Yellow
    $process.StandardInput.WriteLine("hello")
    $process.StandardInput.Flush()
    Start-Sleep -Seconds 2
    
    Write-Host "Sending 'shutdown' command..." -ForegroundColor Yellow
    $process.StandardInput.WriteLine("shutdown")
    $process.StandardInput.Flush()
    Start-Sleep -Seconds 2
    
    # Wait for shutdown or timeout
    $process.WaitForExit(5000) | Out-Null
    
} finally {
    if (-not $process.HasExited) {
        Write-Host "Forcing QEMU to stop..." -ForegroundColor Yellow
        $process.Kill()
    }
    
    Unregister-Event -SourceIdentifier $outputEvent.Name
    Unregister-Event -SourceIdentifier $errorEvent.Name
    $process.Dispose()
}

# Get full output
$fullOutput = $outputBuilder.ToString()

# Save to file
$fullOutput | Out-File -FilePath "test_output.txt" -Encoding UTF8

Write-Host "`n=== Test Results ===" -ForegroundColor Cyan

# Run tests
$tests = @(
    @{Name="Shell loaded"; Expected="Welcome to BogoShell!"},
    @{Name="Hello app ran"; Expected="Hello from C World!"},
    @{Name="Shutdown initiated"; Expected="Shutting down..."}
)

$passed = 0
$total = $tests.Count

foreach ($test in $tests) {
    if ($fullOutput -match [regex]::Escape($test.Expected)) {
        Write-Host "[PASS] $($test.Name)" -ForegroundColor Green
        $passed++
    } else {
        Write-Host "[FAIL] $($test.Name) - expected '$($test.Expected)'" -ForegroundColor Red
    }
}

Write-Host "`nPassed $passed/$total tests" -ForegroundColor $(if ($passed -eq $total) { "Green" } else { "Yellow" })
Write-Host "Full output saved to test_output.txt`n"

exit $(if ($passed -eq $total) { 0 } else { 1 })
