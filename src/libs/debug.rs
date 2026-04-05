use chrono::Local;
use colored::*;

// LogType enum
#[derive(Debug)]
pub enum LogType {
    Success,
    Failed,
    Debug,
    LLM,
    Action,
}

// Tüm log tipleri
fn all_tags() -> Vec<&'static str> {
    vec!["SUCCESS", "FAILED!", "DEBUG", "LLM", "ACTION"]
}

// Ortala ve maksimum genişlik bul
fn center_tag(tag: &str, max_width: usize) -> String {
    let len = tag.len();
    if len >= max_width {
        tag.to_string()
    } else {
        let padding = max_width - len;
        let pad_left = padding / 2;
        let pad_right = padding - pad_left;
        format!("{}{}{}", " ".repeat(pad_left), tag, " ".repeat(pad_right))
    }
}

pub fn printd(message: &str, log_type: LogType) {
    let time = Local::now().format("%H:%M:%S").to_string().bright_black();

    let (tag, color_fn): (&str, fn(&str) -> ColoredString) = match log_type {
        LogType::Success => ("SUCCESS", |s: &str| s.green()),
        LogType::Failed => ("FAILED!", |s: &str| s.red()),
        LogType::Debug => ("DEBUG", |s: &str| s.bright_black()),
        LogType::LLM => ("LLM", |s: &str| s.cyan()),
        LogType::Action => ("ACTION", |s: &str| s.yellow()),

    };

    // Maksimum tag uzunluğunu bul
    let max_tag_len = all_tags().iter().map(|t| t.len()).max().unwrap_or(tag.len());
    let tag_centered = center_tag(tag, max_tag_len);

    // Sabit sütun -> işareti için padding
    let arrow_column = 20; // Saat + [tag] + boşluk toplam
    let current_len = time.len() + 2 + tag_centered.len() + 2;
    let padding = if arrow_column > current_len {
        " ".repeat(arrow_column - current_len)
    } else {
        " ".to_string()
    };

    println!(
        "[{}] [{}]{}-> {}",
        time,
        color_fn(&tag_centered),
        padding,
        message.white()
    );
}

// Macro
#[macro_export]
macro_rules! printd {
    ($msg:expr, $ty:ident) => {
        $crate::printd($msg, $crate::LogType::$ty)
    };
}