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
    #[serde(default = "default_llm_model")]
    pub llm_model: String,
    pub audio_device: String,
    pub mic_gain: f32,
    pub noise_suppression: bool,
    pub echo_cancellation: bool,
    pub vad_enabled: bool,
    pub strip_fillers: bool,
    pub spoken_punctuation: bool,
    pub auto_capitalize: bool,
    pub dictionary: Vec<String>,
    pub replacements: Vec<ReplacementItem>,
}

fn default_llm_model() -> String {
    "gemma2:2b".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            custom_instructions: r#"# Persona
Act as an intelligent, high-fidelity voice dictation assistant. Transform spoken speech into clean, well-formatted, and accurate text.

# Core Dictation Processing
* **Content Preservation:** Preserve all spoken thoughts, sentences, and context without summarizing, dropping, or truncating anything.
* **Smart Structuring:** When a list, sequence, or set of steps is spoken, format it cleanly with newlines and bullet points or numbers while keeping surrounding text intact.
* **Remove Speech Artifacts:** Strip out filler words ("um", "uh", "like") and stutters while retaining the full meaning of every statement.
* **Polished Writing:** Ensure correct punctuation, capitalization, and smooth readability."#.to_string(),
            hotkey: "AltRight".to_string(),
            activation_mode: "push-to-talk".to_string(),
            model_id: "large-v3-turbo-q5_0".to_string(),
            llm_model: default_llm_model(),
            audio_device: "Default".to_string(),
            mic_gain: 1.0,
            noise_suppression: true,
            echo_cancellation: true,
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

    // 7. Intelligent Spoken List Detection & Formatting
    // Automatically detect spoken lists (e.g. "one, ... two, ... three" or "first, ... second, ...")
    let list_formatted = format_spoken_lists(&normalized);
    let final_candidate = if list_formatted != normalized {
        list_formatted
    } else {
        normalized
    };

    // 8. Custom Instructions Style Directives:
    // "Use all lowercase in Slack"
    if instructions_lower.contains("all lowercase") || instructions_lower.contains("lowercase in slack") {
        return final_candidate.trim().to_lowercase();
    }

    // Bullet points directive
    if instructions_lower.contains("bullet") || (instructions_lower.contains("list") && !final_candidate.contains('\n')) {
        let lines: Vec<&str> = final_candidate.split(|c| c == '\n' || c == '.').collect();
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

    // 9. Auto-capitalize sentences if enabled
    if config.auto_capitalize {
        capitalize_sentences(&final_candidate)
    } else {
        final_candidate.trim().to_string()
    }
}

/// Fallback refinement without custom config
pub fn refine_text(raw: &str) -> String {
    let default_config = AppConfig::default();
    refine_text_with_config(raw, &default_config)
}

/// Applies user dictionary casing and phrase replacements to text without destructively re-formatting lists or sentences
pub fn apply_dictionary_and_replacements(text: &str, config: &AppConfig) -> String {
    let mut result = text.to_string();

    // 1. User custom macro replacements (case-insensitive phrase replacement)
    for rep in &config.replacements {
        if !rep.spoken.trim().is_empty() {
            let escaped = regex::escape(rep.spoken.trim());
            if let Ok(re) = Regex::new(&format!(r"(?i)\b{}\b", escaped)) {
                result = re.replace_all(&result, rep.replacement.as_str()).to_string();
            }
        }
    }

    // 2. User Dictionary enforcement (proper casing for custom terms and proper nouns)
    for dict_word in &config.dictionary {
        let word_clean = dict_word.trim();
        if !word_clean.is_empty() {
            let escaped = regex::escape(word_clean);
            if let Ok(re) = Regex::new(&format!(r"(?i)\b{}\b", escaped)) {
                result = re.replace_all(&result, word_clean).to_string();
            }
        }
    }

    result.trim().to_string()
}

/// Asynchronously invokes local Gemma 2 (via Ollama) with the user's custom instructions
pub async fn refine_with_llm(raw: &str, config: &AppConfig) -> String {
    let chosen_model = config.llm_model.trim();
    if chosen_model == "none" || chosen_model == "disabled" {
        return refine_text_with_config(raw, config);
    }

    let model_to_use = if chosen_model.is_empty() {
        "gemma2:2b"
    } else {
        chosen_model
    };

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(15000))
        .build()
    {
        Ok(c) => c,
        Err(_) => return refine_text_with_config(raw, config),
    };

    let instructions = config.custom_instructions.trim();
    let custom_instruction_block = if !instructions.is_empty() {
        format!(
            "\nUSER'S CUSTOM INSTRUCTIONS FOR STRUCTURING & STYLING TEXT (HIGHEST PRIORITY):\n\"\"\"\n{}\n\"\"\"\n\
            Always apply the user's styling, phrasing, capitalization, and structural guidelines above to format their spoken words.\n",
            instructions
        )
    } else {
        String::new()
    };

    let system_prompt = format!(
        "You are the voice dictation editing and polishing engine of AetherVoice (like Aqua Voice).\n\
        Your job is to transform raw, rambling spoken speech into clean, articulate, and well-structured written text according to the user's instructions.\n\n\
        CRITICAL RULES:\n\
        1. Speech-to-Text Polishing: Spoken speech is naturally wordy and repetitive. Strip out verbal crutches ('so basically', 'um', 'uh', 'like', 'you know') and rambling filler clauses. Rewrite the thoughts into crisp, professional, and natural written prose.\n\
        2. First-Person Dictation Invariant: The user is speaking their thoughts or messages. Never converse, never advise, and never address the user as an assistant (e.g. never say 'Please provide...'). Keep the user's voice and perspective ('I need...', 'We need...').\n\
        3. Meaning & Facts: Preserve all core requirements, facts, and intent completely intact.\n\
        4. Lists & Steps: When items, steps, or sequences are spoken, structure them cleanly with bullet points (-) or numbers (1., 2., 3.).\n\
        5. Output: Return ONLY the polished text ready to paste. No markdown codeblocks, no explanations.{}",
        custom_instruction_block
    );

    println!(
        "[AetherVoice] Dictation LLM ({}) processing with custom instructions ({} chars):\n---\n{}\n---",
        model_to_use,
        instructions.len(),
        if instructions.is_empty() { "(Default: Verbatim formatting)" } else { instructions }
    );

    let body = serde_json::json!({
        "model": model_to_use,
        "messages": [
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user",
                "content": "Rewrite dictation: so basically what we want to do is make sure that the server restarts automatically whenever there is a crash so that our users don't see any downtime"
            },
            {
                "role": "assistant",
                "content": "We need to ensure the server automatically restarts upon crashing to prevent user downtime."
            },
            {
                "role": "user",
                "content": "Rewrite dictation: I am going to the store and this is my shopping list eggs grits watermelon sugar then after that I will come back and call you back"
            },
            {
                "role": "assistant",
                "content": "I am going to the store and this is my shopping list:\n\n1. Eggs\n2. Grits\n3. Watermelon\n4. Sugar\n\nAfter that, I will come back and call you back."
            },
            {
                "role": "user",
                "content": format!("Rewrite dictation: {}", raw)
            }
        ],
        "stream": false,
        "keep_alive": "24h",
        "options": {
            "temperature": 0.0,
            "top_p": 0.9
        }
    });

    // Explicitly target IPv4 loopback 127.0.0.1
    match client.post("http://127.0.0.1:11434/api/chat").json(&body).send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                if let Ok(json_val) = resp.json::<serde_json::Value>().await {
                    if let Some(content) = json_val["message"]["content"].as_str() {
                        let cleaned = content.trim();
                        // Strip markdown code block wrappers if model wrapped response in ``` or ```markdown
                        let stripped = if cleaned.starts_with("```") {
                            if let Some(first_newline) = cleaned.find('\n') {
                                let inner = &cleaned[first_newline + 1..];
                                if inner.ends_with("```") {
                                    inner[..inner.len() - 3].trim()
                                } else {
                                    inner.trim()
                                }
                            } else {
                                cleaned
                            }
                        } else {
                            cleaned
                        };

                        // Strip leading command prefixes or surrounding quotes if echoed
                        let mut final_text = stripped.trim();
                        if final_text.to_lowercase().starts_with("rewrite dictation:") {
                            final_text = final_text["rewrite dictation:".len()..].trim();
                        } else if final_text.to_lowercase().starts_with("transcribe and format:") {
                            final_text = final_text["transcribe and format:".len()..].trim();
                        }
                        if final_text.starts_with('"') && final_text.ends_with('"') && final_text.len() >= 2 {
                            final_text = final_text[1..final_text.len() - 1].trim();
                        }

                        if !final_text.is_empty() {
                            println!("[AetherVoice] Local LLM rephrased: '{}'", final_text);
                            // Apply dictionary casing and user replacements without mangling LLM's list formatting
                            return apply_dictionary_and_replacements(final_text, config);
                        }
                    }
                }
            }
            eprintln!("[AetherVoice] Local LLM returned non-success HTTP status ({}), falling back to deterministic rules", status);
            refine_text_with_config(raw, config)
        }
        Err(e) => {
            eprintln!("[AetherVoice] Local LLM connection failed: {}. Falling back to deterministic rules.", e);
            refine_text_with_config(raw, config)
        }
    }
}

/// Automatically detects and formats spoken enumerations into structured markdown lists
fn format_spoken_lists(text: &str) -> String {
    // Check if the text sounds like a numbered sequence:
    // e.g. "one, I need to be like this. Two, also like this, and three, finally like that"
    // or "1. I need to be... 2. Also..."
    let list_item_regex = match Regex::new(r"(?i)(?:^|[\.\,\;\n\s]+)(?:and\s+)?(?:number\s+)?(one|two|three|four|five|six|seven|eight|nine|ten|1|2|3|4|5|6|7|8|9|10)[\:\,\.\s\-]+") {
        Ok(re) => re,
        Err(_) => return text.to_string(),
    };

    // If there are at least two sequence triggers, format into a clean numbered list
    let word_to_num = |w: &str| -> Option<u32> {
        match w.to_lowercase().as_str() {
            "one" | "1" => Some(1),
            "two" | "2" => Some(2),
            "three" | "3" => Some(3),
            "four" | "4" => Some(4),
            "five" | "5" => Some(5),
            "six" | "6" => Some(6),
            "seven" | "7" => Some(7),
            "eight" | "8" => Some(8),
            "nine" | "9" => Some(9),
            "ten" | "10" => Some(10),
            _ => None,
        }
    };

    let matches: Vec<_> = list_item_regex.find_iter(text).collect();
    if matches.len() >= 2 {
        // Verify numbers are ascending (1, 2... or one, two...)
        let mut numbers = Vec::new();
        for m in &matches {
            if let Some(caps) = list_item_regex.captures(m.as_str()) {
                if let Some(num_match) = caps.get(1) {
                    if let Some(n) = word_to_num(num_match.as_str()) {
                        numbers.push((n, m.start(), m.end()));
                    }
                }
            }
        }

        if numbers.len() >= 2 && numbers[0].0 == 1 && numbers[1].0 == 2 {
            let mut result = String::new();
            // Prefix before first item
            let prefix = text[..numbers[0].1].trim();
            if !prefix.is_empty() {
                result.push_str(prefix);
                result.push_str("\n\n");
            }

            for i in 0..numbers.len() {
                let current_num = numbers[i].0;
                let start_idx = numbers[i].2;
                let end_idx = if i + 1 < numbers.len() {
                    numbers[i + 1].1
                } else {
                    text.len()
                };

                let mut item_text = text[start_idx..end_idx].trim().to_string();
                // Clean trailing commas, periods or conjunctions
                if item_text.ends_with(',') || item_text.ends_with(';') {
                    item_text.pop();
                }
                let item_cleaned = item_text.trim();
                if !item_cleaned.is_empty() {
                    result.push_str(&format!("{}. {}\n", current_num, capitalize_sentences(item_cleaned)));
                }
            }

            return result.trim().to_string();
        }
    }

    text.to_string()
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

    #[test]
    fn test_spoken_list_formatting() {
        let config = AppConfig::default();
        let input = "one, I need to be like this. Two, also like this, and three, finally like that.";
        let output = refine_text_with_config(input, &config);
        assert_eq!(output, "1. I need to be like this\n2. Also like this\n3. Finally like that.");
    }
}

