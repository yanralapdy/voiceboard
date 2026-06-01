# voiceboard — Windows + WSL Setup

For using your **Windows microphone** from `voiceboard` running in WSL2.

## Architecture

```
Windows                           WSL
┌──────────────┐    TCP 4713    ┌─────────────────────┐
│ PulseAudio   │◄──────────────│ voiceboard           │
│ server.exe   │               │ (cpal → PulseAudio)  │
│   ↑          │               │                      │
│ WASAPI/wave  │               │                      │
│ (mic)        │               │                      │
└──────────────┘               └─────────────────────┘
```

## Windows: Install PulseAudio

### Option A: Chocolatey (easiest)

Run PowerShell **as Administrator**:

```powershell
choco install pulseaudio -y
```

### Option B: Manual

Download from https://www.freedesktop.org/wiki/Software/PulseAudio/Ports/Windows/ and extract to `C:\pulseaudio`.

### Configure

Edit `C:\Program Files\PulseAudio\etc\pulse\default.pa` (or the config dir from your install). Add:

```
load-module module-wasapi source_name=input
```

If WASAPI isn't available, try:

```
load-module module-waveout source_name=input record=1
```

### Start

```powershell
Start-Service PulseAudio
```

Or manually: `C:\pulseaudio\bin\pulseaudio.exe`

## WSL: Configure

Already done if you used the provided setup. Otherwise:

```bash
# Install pulseaudio client libs
sudo apt install libpulse-dev

# Add to ~/.zshrc or ~/.bashrc
export PULSE_SERVER=tcp:host.docker.internal:4713
```

## Verify

```bash
pactl info
# Server String: host.docker.internal:4713

# List sources
pactl list sources short | grep input
```

## Troubleshooting

| Problem | Fix |
|---|---|
| `pactl info` fails | PulseAudio not running on Windows |
| `Connection refused` | Windows firewall blocking port 4713 |
| No input sources | `default.pa` missing `source_name=input` module |
| ALSA errors but works | Ignore — cpal falls back to PulseAudio |
