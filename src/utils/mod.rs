use std::error::Error;
use std::fs;
use std::path::Path;

pub fn ensure_directory(path: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(path);
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn save_results_to_file(
    path: &str,
    content: &str,
    format: &str,
) -> Result<(), Box<dyn Error>> {
    let path = Path::new(path);
    ensure_directory(path.parent().unwrap().to_str().unwrap())?;
    fs::write(path, content)?;
    Ok(())
}

pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{} seconds", secs)
    } else if secs < 3600 {
        format!("{} minutes {} seconds", secs / 60, secs % 60)
    } else {
        format!(
            "{} hours {} minutes {} seconds",
            secs / 3600,
            (secs % 3600) / 60,
            secs % 60
        )
    }
}

pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => c,
            _ => '_',
        })
        .collect()
}

pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
} 