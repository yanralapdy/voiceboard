use anyhow::{Context, Result};
use std::path::PathBuf;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const HUB_BASE: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

pub struct Config {
    pub model: ModelSize,
    pub language: String,
}

pub enum ModelSize {
    Tiny,
    Base,
    Small,
    Medium,
    Large,
}

impl std::str::FromStr for ModelSize {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tiny" => Ok(Self::Tiny),
            "base" => Ok(Self::Base),
            "small" => Ok(Self::Small),
            "medium" => Ok(Self::Medium),
            "large" => Ok(Self::Large),
            _ => Err(format!(
                "unknown model '{}' — choose tiny, base, small, medium, large",
                s
            )),
        }
    }
}

impl ModelSize {
    fn filename(&self, lang: &str) -> String {
        if lang == "en" {
            format!("ggml-{}.en.bin", self.name())
        } else {
            format!("ggml-{}.bin", self.name())
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Tiny => "tiny",
            Self::Base => "base",
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large-v3",
        }
    }
}

pub fn ensure_model(config: &Config) -> Result<PathBuf> {
    let path = model_path(&config.model, &config.language);
    if path.exists() {
        return Ok(path);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let fname = config.model.filename(&config.language);
    let url = format!("{HUB_BASE}/{fname}");
    let size_str = match config.model {
        ModelSize::Tiny => "~75 MB",
        ModelSize::Base => "~150 MB",
        ModelSize::Small => "~500 MB",
        ModelSize::Medium => "~1.5 GB",
        ModelSize::Large => "~3 GB",
    };

    println!("  Downloading {fname} ({size_str})...");
    let status = std::process::Command::new("wget")
        .args(["-O", path.to_str().unwrap(), "--progress=dot:giga", &url])
        .status()
        .or_else(|_| {
            std::process::Command::new("curl")
                .args(["-L", "-o", path.to_str().unwrap(), &url])
                .status()
        })
        .context("need wget or curl to download model")?;

    if !status.success() {
        std::fs::remove_file(&path).ok();
        anyhow::bail!("model download failed");
    }
    Ok(path)
}

fn model_path(model: &ModelSize, lang: &str) -> PathBuf {
    let cache = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".cache")
        });
    cache
        .join("voiceboard")
        .join(model.filename(lang))
}

pub fn transcribe(samples: &[f32], config: &Config) -> Result<String> {
    let model_path = ensure_model(config)?;

    let ctx = WhisperContext::new_with_params(
        model_path.to_str().context("invalid model path")?,
        WhisperContextParameters::default(),
    )
    .context("failed to load whisper model")?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 5 });
    params.set_language(Some(&config.language));

    let mut state = ctx.create_state()?;
    state
        .full(params, samples)
        .context("whisper transcription failed")?;

    let mut result = String::new();
    for segment in state.as_iter() {
        result.push_str(&segment.to_string());
    }
    Ok(result)
}
