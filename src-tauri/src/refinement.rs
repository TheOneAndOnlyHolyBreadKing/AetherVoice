use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacementItem {
    pub spoken: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub custom_instructions: String,
    pub hotkey: String,
    pub activation_mode: String,
    pub model_id: String,
    pub vad_enabled: bool,
    pub strip_fillers: bool,
    pub spoken_punctuation: bool,
    pub auto_capitalize: bool,
    pub dictionary: Vec<String>,
    pub replacements: Vec<ReplacementItem>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            custom_instructions: r#"# Persona
Act as an expert software architect and prompt engineer. Transform raw spoken input into clear, structured, and precise instructions optimized for AI-driven software development.

# Core Dictation Processing
* **Remove Speech Artifacts:** Instantly strip out filler words ("um", "uh", "like"), conversational pleasantries, stutters, and self-corrections (retain only the final corrected thought).
* **Technical Translation:** Map non-technical or casual phrases to standard developer vocabulary (e.g., convert "place to hold user stuff" to "database schema for user profiles," or "button to send" to "form submission handler").
* **Infer Technical Context:** Explicitly define implied edge cases, type requirements, error handling, and architectural standards based on the spoken intent."#.to_string(),
            hotkey: "AltRight".to_string(),
            activation_mode: "push-to-talk".to_string(),
            model_id: "large-v3-turbo-q5_0".to_string(),
            vad_enabled: true,
            strip_fillers: true,
            spoken_punctuation: true,
            auto_capitalize: true,
            dictionary: vec![
                "AetherVoice".to_string(),
                "Tauri".to_string(),
                "Rust".to_string(),
                "TypeScript".to_string(),
                "Whisper".to_string(),
                "PostgreSQL".to_string(),
                "Kubernetes".to_string(),
                "GraphQL".to_string(),
                "GitHub".to_string(),
                "API".to_string(),
            ],
            replacements: vec![
                ReplacementItem {
                    spoken: "my email".to_string(),
                    replacement: "dev@example.com".to_string(),
                },
                ReplacementItem {
                    spoken: "k eight s".to_string(),
                    replacement: "k8s".to_string(),
                },
                ReplacementItem {
                    spoken: "smile emoji".to_string(),
                    replacement: "😊".to_string(),
                },
            ],
        }
    }
}

static PUNCTUATION_REGEXES: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
static FILLERS_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_punctuation_rules() -> &'static Vec<(Regex, &'static str)> {
    PUNCTUATION_REGEXES.get_or_init(|| {
        vec![
            (Regex::new(r"(?i)\b(new\s+line|next\s+line)\b").unwrap(), "\n"),
            (Regex::new(r"(?i)\b(new\s+paragraph)\b").unwrap(), "\n\n"),
            (Regex::new(r"(?i)\bperiod\b").unwrap(), "."),
            (Regex::new(r"(?i)\bcomma\b").unwrap(), ","),
            (Regex::new(r"(?i)\bquestion\s+mark\b").unwrap(), "?"),
            (Regex::new(r"(?i)\bexclamation\s+(mark|point)\b").unwrap(), "!"),
            (Regex::new(r"(?i)\bcolon\b").unwrap(), ":"),
            (Regex::new(r"(?i)\bsemicolon\b").unwrap(), ";"),
            (Regex::new(r"(?i)\bopen\s+quote\b").unwrap(), "\""),
            (Regex::new(r"(?i)\bclose\s+quote\b").unwrap(), "\""),
            (Regex::new(r"(?i)\bhyphen|dash\b").unwrap(), "-"),
            (Regex::new(r"(?i)\bellipsis\b").unwrap(), "..."),
        ]
    })
}

fn get_fillers_regex() -> &'static Regex {
    FILLERS_REGEX.get_or_init(|| {
        Regex::new(r"(?i)\b(um|uh|erm|ah|you\s+know|like\s+so)\b").unwrap()
    })
}

/// Refines raw transcription text with full configuration and custom instructions
pub fn refine_text_with_config(raw: &str, config: &AppConfig) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut text = trimmed.to_string();

    // 1. User custom macro replacements (case-insensitive phrase replacement)
    for rep in &config.replacements {
        if !rep.spoken.trim().is_empty() {
            let escaped = regex::escape(rep.spoken.trim());
            if let Ok(re) = Regex::new(&format!(r"(?i)\b{}\b", escaped)) {
                text = re.replace_all(&text, rep.replacement.as_str()).to_string();
            }
        }
    }

    // 2. Technical translation rules if indicated by custom instructions
    let instructions_lower = config.custom_instructions.to_lowercase();
    if instructions_lower.contains("technical translation") || instructions_lower.contains("software architect") {
        let tech_mappings = [
            (r"(?i)\bplace to hold user stuff\b", "database schema for user profiles"),
            (r"(?i)\bbutton to send\b", "form submission handler"),
            (r"(?i)\bdata base\b", "database"),
            (r"(?i)\bfront end\b", "frontend"),
            (r"(?i)\bback end\b", "backend"),
            (r"(?i)\bpull request\b", "PR"),
        ];
        for (pattern, replacement) in tech_mappings {
            if let Ok(re) = Regex::new(pattern) {
                text = re.replace_all(&text, replacement).to_string();
            }
        }
    }

    // 3. Strip vocal fillers if enabled
    if config.strip_fillers || instructions_lower.contains("remove speech artifacts") || instructions_lower.contains("strip") {
        text = get_fillers_regex().replace_all(&text, "").to_string();
    }

    // 4. Apply spoken punctuation macros if enabled
    if config.spoken_punctuation {
        for (re, replacement) in get_punctuation_rules() {
            text = re.replace_all(&text, *replacement).to_string();
        }
    }

    // 5. Normalize spacing around punctuation
    let re_spaces = Regex::new(r"\s+([.,!?:;])").unwrap();
    let cleaned_spacing = re_spaces.replace_all(&text, "$1").to_string();

    let re_multi_space = Regex::new(r"[ \t]+").unwrap();
    let mut normalized = re_multi_space.replace_all(&cleaned_spacing, " ").to_string();

    // 6. User Dictionary enforcement (proper casing for custom terms and proper nouns)
    for dict_word in &config.dictionary {
        let word_clean = dict_word.trim();
        if !word_clean.is_empty() {
            let escaped = regex::escape(word_clean);
            if let Ok(re) = Regex::new(&format!(r"(?i)\b{}\b", escaped)) {
                normalized = re.replace_all(&normalized, word_clean).to_string();
            }
        }
    }

    // 7. Custom Instructions Style Directives:
    // "Use all lowercase in Slack"
    if instructions_lower.contains("all lowercase") || instructions_lower.contains("lowercase in slack") {
        return normalized.trim().to_lowercase();
    }

    // Bullet points directive
    if instructions_lower.contains("bullet") || instructions_lower.contains("list") {
        let lines: Vec<&str> = normalized.split(|c| c == '\n' || c == '.').collect();
        let bulleted: Vec<String> = lines
            .into_iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| format!("• {}", capitalize_sentences(s)))
            .collect();
        if !bulleted.is_empty() {
            return bulleted.join("\n");
        }
    }

    // 8. Auto-capitalize sentences if enabled
    if config.auto_capitalize {
        capitalize_sentences(&normalized)
    } else {
        normalized.trim().to_string()
    }
}

/// Fallback refinement without custom config
pub fn refine_text(raw: &str) -> String {
    let default_config = AppConfig::default();
    refine_text_with_config(raw, &default_config)
}

fn capitalize_sentences(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut capitalize_next = true;

    for ch in text.chars() {
        if capitalize_next && ch.is_alphabetic() {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
            if ch == '.' || ch == '?' || ch == '!' || ch == '\n' {
                capitalize_next = true;
            }
        }
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spoken_punctuation() {
        let input = "hello world period this is a test comma right question mark new line next paragraph";
        let output = refine_text(input);
        assert_eq!(output, "Hello world. This is a test, right?\n\n");
    }

    #[test]
    fn test_custom_instructions_lowercase() {
        let mut config = AppConfig::default();
        config.custom_instructions = "Use all lowercase in Slack".to_string();
        let input = "Hello World, Testing SLACK Output!";
        let output = refine_text_with_config(input, &config);
        assert_eq!(output, "hello world, testing slack output!");
    }

    #[test]
    fn test_custom_replacements() {
        let mut config = AppConfig::default();
        config.replacements = vec![ReplacementItem {
            spoken: "brb".to_string(),
            replacement: "be right back".to_string(),
        }];
        let input = "I will brb period";
        let output = refine_text_with_config(input, &config);
        assert_eq!(output, "I will be right back.");
    }
}

