mod audio;
mod clipboard;
mod transcribe;

use anyhow::Result;
use clap::Parser;
use std::io::Write;

#[derive(Parser)]
#[command(name = "voiceboard", version, about = "Mic → whisper transcription → clipboard")]
struct Args {
    #[arg(long, help = "Select audio input device by ID")]
    device: Option<usize>,

    #[arg(long, help = "List available audio input devices and exit")]
    list_devices: bool,

    #[arg(long, default_value = "tiny", help = "Whisper model size [tiny, base, small, medium, large]")]
    model: String,

    #[arg(long, default_value = "en", help = "Language code (e.g. en, fr, de, ja)")]
    language: String,

    #[arg(long, default_value_t = 2.0, help = "Auto-stop after N seconds of silence (0 = disabled, use Enter toggle)")]
    silence_timeout: f32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.list_devices {
        let devices = audio::list_devices()?;
        if devices.is_empty() {
            println!("No input devices found.");
        } else {
            println!("Available input devices:");
            for (id, name) in &devices {
                println!("  {id}: {name}");
            }
        }
        return Ok(());
    }

    let model: transcribe::ModelSize = args
        .model
        .parse()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let transcribe_cfg = transcribe::Config {
        model,
        language: args.language.clone(),
    };

    println!("voiceboard v{} — mic → clipboard", env!("CARGO_PKG_VERSION"));
    println!("  Model: {}", transcribe_cfg.model.name());
    println!("  Language: {}", transcribe_cfg.language);
    if args.silence_timeout > 0.0 {
        println!("  Silence timeout: {}s", args.silence_timeout);
    }
    println!();

    transcribe::ensure_model(&transcribe_cfg)?;

    loop {
        if args.silence_timeout > 0.0 {
            println!("▶ Press Enter to start recording.");
        } else {
            println!("▶ Press Enter to start recording (or q + Enter to quit).");
        }

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let line = input.trim();

        if args.silence_timeout <= 0.0 && (line == "q" || line == "quit") {
            break;
        }

        let silence = if args.silence_timeout > 0.0 {
            Some(args.silence_timeout)
        } else {
            None
        };

        match audio::record(args.device, silence) {
            Ok(samples) => {
                print!("  Transcribing…");
                std::io::stdout().flush()?;
                match transcribe::transcribe(&samples, &transcribe_cfg) {
                    Ok(text) => {
                        let text = text.trim().to_string();
                        if text.is_empty() {
                            println!("\r  (silence detected — no speech)");
                            continue;
                        }
                        clipboard::copy(&text)?;
                        println!("\r  ✓ \"{text}\"");
                        println!("  (copied to clipboard)");
                    }
                    Err(e) => {
                        println!("\r  ✗ {e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("  ✗ {e}");
            }
        }
    }

    Ok(())
}
