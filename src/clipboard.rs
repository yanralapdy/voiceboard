use anyhow::Result;

pub fn copy(text: &str) -> Result<()> {
    let mut clip = arboard::Clipboard::new()?;
    clip.set_text(text)?;
    Ok(())
}
