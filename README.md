# voiceboard

CLI app: speak into your mic → local whisper transcription → auto-copied to clipboard.

[![CI](https://github.com/yanralapdy/voiceboard/actions/workflows/release.yml/badge.svg)](https://github.com/yanralapdy/voiceboard/actions/workflows/release.yml)

## Quick start

```bash
cargo install voiceboard
voiceboard
```

Press **Enter** to start recording, **Enter** again to stop. The transcript is copied to your clipboard automatically.

Or with auto-stop:

```bash
voiceboard --silence-timeout 1.5
```

Press **Enter** to start, stop speaking — it auto-stops after 1.5 seconds of silence.

## Install options

### 1. Cargo (if you have Rust)

```bash
cargo install voiceboard
```

### 2. Pre-built binaries

Download from the [releases page](https://github.com/yanralapdy/voiceboard/releases):

| Platform | File |
|---|---|
| Linux x86_64 | `voiceboard-x86_64-linux` |
| Windows x86_64 | `voiceboard-x86_64-windows.exe` |
| macOS x86_64 | `voiceboard-x86_64-macos` |
| macOS ARM64 | `voiceboard-aarch64-macos` |

Make the binary executable and put it in your `PATH`.

### 3. Docker (build only)

```bash
git clone https://github.com/yanralapdy/voiceboard.git
cd voiceboard
docker build -t voiceboard-builder .
docker create --name vb voiceboard-builder
docker cp vb:/voiceboard .
docker rm vb
./voiceboard --help
```

The binary is built statically — run it natively, no Docker runtime needed.

## Usage

```
voiceboard [OPTIONS]

Options:
      --device <ID>              Select audio input device by ID
      --list-devices             List available audio input devices and exit
      --model <SIZE>             Whisper model size [default: tiny]
                                 [possible: tiny, base, small, medium, large]
      --language <CODE>          Language code [default: en]
                                 (e.g. en, fr, de, ja, zh)
      --silence-timeout <SECS>   Auto-stop after N seconds of silence [default: 5]
                                 Set to 0 to disable and use Enter toggle instead
  -h, --help                     Print help
  -V, --version                  Print version
```

### Interactive mode (default)

```
$ voiceboard

▶ Press Enter to start recording.
(Enter)
▶ Recording… press Enter to stop
  Transcribing…
  ✓ "hello world"
  (copied to clipboard)
▶ Press Enter to start recording.
q
```

### Auto-stop mode

```
$ voiceboard --silence-timeout 2.0

▶ Press Enter to start recording.
(Enter)
  Recording… auto-stop after 2s of silence.
  (speak... pause...)
  ✓ "short meeting notes"
  (copied to clipboard)
```

### Select a specific mic

```bash
voiceboard --list-devices
#   0: Blue Yeti
#   1: Built-in Microphone

voiceboard --device 0
```

## How it works

```
Microphone ──▶ cpal (capture) ──▶ whisper.cpp (local) ──▶ arboard (clipboard)
                     │                    │
              ┌──────┘                    └──────┐
              │                                  │
        16-bit PCM,                      ggml-tiny.en.bin
        16 kHz mono                      (~75 MB, downloaded
        resampled from                    on first run via
        device rate                       wget or curl)
```

- **Audio**: captured via `cpal`, converted to mono, resampled to 16 kHz
- **Transcription**: local whisper.cpp via `whisper-rs` — no internet needed, fully private
- **Clipboard**: copied with `arboard` — works on X11, Wayland, and Windows
- **Model**: auto-downloaded from Hugging Face to `~/.cache/voiceboard/`

## Platform setup

### Linux

ALSA or PulseAudio required. Install:

```bash
# Debian/Ubuntu
sudo apt install libasound2-dev libpulse-dev

# Fedora
sudo dnf install alsa-lib-devel pulseaudio-libs-devel
```

### Windows (WSL2)

To use your Windows microphone from WSL:

1. **On Windows**: install PulseAudio — `choco install pulseaudio` (run as Admin)
2. **On Windows**: ensure PulseAudio service is running
3. **In WSL**: set `export PULSE_SERVER=tcp:host.docker.internal:4713`
4. Run `voiceboard` in WSL

> Full setup: [SETUP.md](./SETUP.md)

### macOS

Should work out of the box via CoreAudio. Requires `cmake` for whisper-rs build:

```bash
brew install cmake
```

## Build from source

```bash
git clone https://github.com/yanralapdy/voiceboard.git
cd voiceboard
cargo build --release
./target/release/voiceboard --help
```

System dependencies: `cmake`, `libclang-dev`, `libasound2-dev` (Linux) or `brew install cmake` (macOS).

## License

MIT
