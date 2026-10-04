use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacementItem {
    pub spoken: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: String,
    pub time: String,
    pub timestamp: u64,
    pub text: String,
    pub words: usize,
    pub duration: u64,
    #[serde(default)]
    pub app: String,
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
    #[serde(default = "default_noise_deafening")]
    pub noise_deafening: bool,
    #[serde(default = "default_auto_capitalize")]
    pub auto_capitalize: bool,
    pub dictionary: Vec<String>,
    pub replacements: Vec<ReplacementItem>,
    #[serde(default = "default_deep_context")]
    pub deep_context: bool,
    #[serde(default = "default_hands_free_hotkey")]
    pub hands_free_hotkey: String,
}

fn default_noise_deafening() -> bool {
    false
}

fn default_auto_capitalize() -> bool {
    true
}

fn default_deep_context() -> bool {
    true
}

fn default_hands_free_hotkey() -> String {
    "F8".to_string()
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
            noise_deafening: false,
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
            deep_context: true,
            hands_free_hotkey: "F8".to_string(),
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

/// Robustly strips conversational AI boilerplate preambles and sign-offs (e.g. "Okay, here is the text...", "Let me know if...")
pub fn clean_chatbot_boilerplate(text: &str) -> String {
    let mut cleaned = text.trim().to_string();

    // 1. Strip markdown code block wrappers
    if cleaned.starts_with("```") {
        if let Some(first_nl) = cleaned.find('\n') {
            let inner = &cleaned[first_nl + 1..];
            if inner.ends_with("```") {
                cleaned = inner[..inner.len() - 3].trim().to_string();
            } else {
                cleaned = inner.trim().to_string();
            }
        }
    }

    // 2. Strip conversational preambles (e.g. "Okay, here is...", "Sure, here's...", "Here is the text with...")
    static PREAMBLE_RE: OnceLock<Regex> = OnceLock::new();
    let preamble_re = PREAMBLE_RE.get_or_init(|| {
        Regex::new(r"(?i)^(?:(?:okay|ok|sure|certainly|here\s+is|here's|below\s+is|i've|i\s+have)[^\n\:\.\!]*[\:\.\!]\s*)+").unwrap()
    });
    cleaned = preamble_re.replace(&cleaned, "").trim().to_string();

    // 3. Strip trailing conversational sign-offs (e.g. "Let me know if you need anything else!", "Hope this helps!", etc.)
    static TRAILING_RE: OnceLock<Regex> = OnceLock::new();
    let trailing_re = TRAILING_RE.get_or_init(|| {
        Regex::new(r"(?i)(?:\n+|\s+)(?:let\s+me\s+know|hope\s+this\s+helps|feel\s+free|please\s+note|is\s+there\s+anything)[^\n]*[\.\!\?]?\s*$").unwrap()
    });
    cleaned = trailing_re.replace(&cleaned, "").trim().to_string();

    // 4. Strip command prefixes echoed by model
    if cleaned.to_lowercase().starts_with("rewrite spoken dictation:") {
        cleaned = cleaned["rewrite spoken dictation:".len()..].trim().to_string();
    } else if cleaned.to_lowercase().starts_with("rewrite dictation:") {
        cleaned = cleaned["rewrite dictation:".len()..].trim().to_string();
    } else if cleaned.to_lowercase().starts_with("transcribe and format:") {
        cleaned = cleaned["transcribe and format:".len()..].trim().to_string();
    }

    // 5. Strip accidental enclosing quotes wrapped by model
    if (cleaned.starts_with('"') && cleaned.ends_with('"') && cleaned.len() >= 2)
        || (cleaned.starts_with('“') && cleaned.ends_with('”') && cleaned.len() >= 2)
    {
        cleaned = cleaned[1..cleaned.len() - 1].trim().to_string();
    }

    cleaned
}

/// Asynchronously invokes local Gemma 2 (via Ollama) with the user's custom instructions and optional screen context
pub async fn refine_with_llm(raw: &str, config: &AppConfig, screen_context: Option<&str>) -> String {
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

    let deep_context_block = if config.deep_context {
        if let Some(ctx) = screen_context {
            if !ctx.trim().is_empty() {
                format!(
                    "\nACTIVE APPLICATION & SCREEN CONTEXT (BOOST ACCURACY):\n\"\"\"\n{}\n\"\"\"\n\
                    Use this active window and application context to accurately transcribe specialized terms, technical identifiers, variables, or software jargon relevant to what the user is working on.\n",
                    ctx.trim()
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let system_prompt = format!(
        "You are the internal voice dictation processing engine of AetherVoice. You act like a keyboard transcription tool, NOT an AI chatbot.\n\
        Your job is to transform raw, rambling spoken speech into clean, articulate written prose according to the user's instructions.\n\n\
        CRITICAL INVARIANTS:\n\
        1. NEVER TALK TO THE USER. You are NOT an AI assistant. Never output conversational remarks, preambles, or postambles (e.g. NEVER say 'Here is the text...', 'Sure', 'According to your settings', 'Let me know if you need anything else').\n\
        2. FIRST-PERSON DICTATION PERSPECTIVE: Maintain the user's perspective ('I need...', 'We need...'). Never address the user as 'you' or give instructions to the user.\n\
        3. SPEECH POLISHING: Remove speech crutches ('so basically', 'um', 'uh', 'like', 'you know') and rambling filler clauses. Rewrite repetitive speech into concise, articulate written prose.\n\
        4. PRESERVE MEANING: Keep all core requirements, facts, and intent completely intact.\n\
        5. INTELLIGENT STRUCTURING: Do not turn ordinary paragraphs into bullet points unless the user clearly dictates a list, sequence of items, or specific bullet points. Let natural sentences flow as coherent, well-structured paragraphs, while structuring actual items/steps with clean numbering or bullet points.\n\
        6. OUTPUT RULE: Output ONLY the final processed text ready to be pasted. No quotes, no intro, no comments.{}{}",
        custom_instruction_block,
        deep_context_block
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
                "content": "Rewrite spoken dictation: so basically what we want to do is make sure that the server restarts automatically whenever there is a crash so that our users don't see any downtime"
            },
            {
                "role": "assistant",
                "content": "We need to ensure the server automatically restarts upon crashing to prevent user downtime."
            },
            {
                "role": "user",
                "content": "Rewrite spoken dictation: I am going to the store and this is my shopping list eggs grits watermelon sugar then after that I will come back and call you back"
            },
            {
                "role": "assistant",
                "content": "I am going to the store and this is my shopping list:\n\n1. Eggs\n2. Grits\n3. Watermelon\n4. Sugar\n\nAfter that, I will come back and call you back."
            },
            {
                "role": "user",
                "content": format!("Rewrite spoken dictation: {}", raw)
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
                        let final_text = clean_chatbot_boilerplate(content);

                        if !final_text.is_empty() {
                            println!("[AetherVoice] Local LLM rephrased: '{}'", final_text);
                            // Apply dictionary casing and user replacements without mangling LLM's list formatting
                            return apply_dictionary_and_replacements(&final_text, config);
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

