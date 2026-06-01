# Run this as Administrator in PowerShell on Windows
# Requires: Chocolatey (choco) installed

Write-Host "voiceboard — Windows PulseAudio setup" -ForegroundColor Cyan

# 1. Install PulseAudio
Write-Host "`n[1/3] Installing PulseAudio via Chocolatey..." -ForegroundColor Yellow
choco install pulseaudio -y
if (-not $?) {
    Write-Host "  Chocolatey install failed. Install manually from:" -ForegroundColor Red
    Write-Host "  https://www.freedesktop.org/wiki/Software/PulseAudio/Ports/Windows/"
    exit 1
}

# 2. Configure PulseAudio to load Windows mic
$pulseConfigDir = "$env:ProgramFiles\PulseAudio\etc\pulse"
$defaultPa = Join-Path $pulseConfigDir "default.pa"

if (Test-Path $defaultPa) {
    Write-Host "`n[2/3] Configuring default.pa for mic input..." -ForegroundColor Yellow
    $wasapiLine = "load-module module-wasapi source_name=input"
    Add-Content -Path $defaultPa -Value "`n$wasapiLine" -Force
    Write-Host "  Added: $wasapiLine"
} else {
    Write-Host "  default.pa not found at $defaultPa" -ForegroundColor Yellow
    Write-Host "  You may need to configure it manually."
}

# 3. Start PulseAudio service
Write-Host "`n[3/3] Starting PulseAudio service..." -ForegroundColor Yellow
Start-Service PulseAudio -ErrorAction SilentlyContinue
Set-Service PulseAudio -StartupType Automatic

Write-Host "`n✓ PulseAudio setup complete!" -ForegroundColor Green
Write-Host "  It's already running. Keep it running while using voiceboard."
Write-Host ""
Write-Host "Next step: Open a new WSL terminal and run:"
Write-Host "  cd ~/sites/personal/rust/voiceboard"
Write-Host "  cargo run --release"
