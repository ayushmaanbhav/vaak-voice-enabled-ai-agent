# Multilingual Support Plan - All Indian Languages

> **Priority:** P0 CRITICAL
> **Scope:** Language-agnostic support for 22+ Indian languages
> **Impact:** Core functionality for Indian market

---

## Current State Analysis

### What's Already Working Well

| Component | Status | Notes |
|-----------|--------|-------|
| **STT (IndicConformer)** | Excellent | Supports 22 Indian languages natively |
| **TTS (IndicF5)** | Good | Supports major Indian languages |
| **Unicode Word Boundaries** | Good | Uses `unicode_segmentation` crate |
| **Grapheme Segmentation** | Good | Proper token estimation for Indic scripts |
| **Hindi Punctuation** | Good | Supports । ॥ (danda/double danda) |
| **Rust `.to_lowercase()`** | OK | Unicode-aware, works for most scripts |

### What Needs Improvement

| Component | Issue | Impact |
|-----------|-------|--------|
| **Indic Numeral Support** | Only Devanagari (०-९) | Other scripts fail |
| **Number Word Extraction** | Only Hindi words | Tamil, Telugu, etc. fail |
| **Currency Multipliers** | Hindi/English only | Regional terms not recognized |
| **Script Detection** | Devanagari only | Can't optimize for other scripts |
| **Phone Validation** | ASCII digits only | Indic numeral input fails |

---

## Supported Languages (22 Indian Languages)

| Code | Language | Script | Numeral Range |
|------|----------|--------|---------------|
| as | Assamese | Bengali | U+09E6-U+09EF |
| bn | Bengali | Bengali | U+09E6-U+09EF |
| brx | Bodo | Devanagari | U+0966-U+096F |
| doi | Dogri | Devanagari | U+0966-U+096F |
| gu | Gujarati | Gujarati | U+0AE6-U+0AEF |
| hi | Hindi | Devanagari | U+0966-U+096F |
| kn | Kannada | Kannada | U+0CE6-U+0CEF |
| kok | Konkani | Devanagari | U+0966-U+096F |
| ks | Kashmiri | Arabic/Devanagari | U+0966-U+096F |
| mai | Maithili | Devanagari | U+0966-U+096F |
| ml | Malayalam | Malayalam | U+0D66-U+0D6F |
| mni | Manipuri | Bengali/Meetei | U+09E6-U+09EF |
| mr | Marathi | Devanagari | U+0966-U+096F |
| ne | Nepali | Devanagari | U+0966-U+096F |
| or | Odia | Odia | U+0B66-U+0B6F |
| pa | Punjabi | Gurmukhi | U+0A66-U+0A6F |
| sa | Sanskrit | Devanagari | U+0966-U+096F |
| sat | Santali | Ol Chiki | U+1C50-U+1C59 |
| sd | Sindhi | Arabic/Devanagari | U+0966-U+096F |
| ta | Tamil | Tamil | U+0BE6-U+0BEF |
| te | Telugu | Telugu | U+0C66-U+0C6F |
| ur | Urdu | Arabic | U+0660-U+0669 |

---

## Solution 1: Universal Indic Numeral Support

### Design: Language-Agnostic Numeral Normalization

```rust
// crates/core/src/indic_numerals.rs (NEW FILE)

/// All Indic numeral ranges with their Unicode blocks
pub const INDIC_NUMERAL_RANGES: &[(char, char, &str)] = &[
    // Script Name         Start    End      Script ID
    ('\u{0966}', '\u{096F}', "devanagari"),  // ०-९
    ('\u{09E6}', '\u{09EF}', "bengali"),     // ০-৯
    ('\u{0A66}', '\u{0A6F}', "gurmukhi"),    // ੦-੯
    ('\u{0AE6}', '\u{0AEF}', "gujarati"),    // ૦-૯
    ('\u{0B66}', '\u{0B6F}', "odia"),        // ୦-୯
    ('\u{0BE6}', '\u{0BEF}', "tamil"),       // ௦-௯
    ('\u{0C66}', '\u{0C6F}', "telugu"),      // ౦-౯
    ('\u{0CE6}', '\u{0CEF}', "kannada"),     // ೦-೯
    ('\u{0D66}', '\u{0D6F}', "malayalam"),   // ൦-൯
    ('\u{0660}', '\u{0669}', "arabic"),      // ٠-٩ (for Urdu)
    ('\u{1C50}', '\u{1C59}', "ol_chiki"),    // ᱐-᱙ (Santali)
];

/// Convert any Indic numeral character to ASCII digit
pub fn indic_to_ascii_digit(c: char) -> Option<char> {
    for &(start, end, _) in INDIC_NUMERAL_RANGES {
        if c >= start && c <= end {
            let digit = (c as u32 - start as u32) as u8;
            return Some((b'0' + digit) as char);
        }
    }
    // Already ASCII digit
    if c.is_ascii_digit() {
        return Some(c);
    }
    None
}

/// Normalize all Indic numerals in text to ASCII
pub fn normalize_numerals(text: &str) -> String {
    text.chars()
        .map(|c| indic_to_ascii_digit(c).unwrap_or(c))
        .collect()
}

/// Extract numeric value from mixed Indic/ASCII text
pub fn extract_number(text: &str) -> Option<f64> {
    let normalized = normalize_numerals(text);

    // Find contiguous digit sequences
    let mut num_str = String::new();
    let mut has_decimal = false;

    for c in normalized.chars() {
        if c.is_ascii_digit() {
            num_str.push(c);
        } else if c == '.' && !has_decimal {
            num_str.push(c);
            has_decimal = true;
        } else if !num_str.is_empty() {
            break;
        }
    }

    num_str.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devanagari_numerals() {
        assert_eq!(normalize_numerals("५००००"), "50000");
        assert_eq!(normalize_numerals("१२३"), "123");
    }

    #[test]
    fn test_tamil_numerals() {
        assert_eq!(normalize_numerals("௫௦௦௦௦"), "50000");
    }

    #[test]
    fn test_bengali_numerals() {
        assert_eq!(normalize_numerals("৫০০০০"), "50000");
    }

    #[test]
    fn test_mixed_numerals() {
        // "5 lakh" in Devanagari + ASCII
        assert_eq!(normalize_numerals("५ lakh"), "5 lakh");
    }
}
```

---

## Solution 2: Multilingual Currency Multiplier Words

### Design: Support Multipliers in All Major Languages

```rust
// crates/agent/src/multilingual_amounts.rs (NEW FILE)

use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Multiplier values for Indian numbering system
pub const LAKH: f64 = 100_000.0;
pub const CRORE: f64 = 10_000_000.0;
pub const THOUSAND: f64 = 1_000.0;

/// Multilingual multiplier words mapped to their values
/// Key: lowercase word, Value: multiplier
pub static MULTIPLIER_WORDS: Lazy<HashMap<&'static str, f64>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // === LAKH (1,00,000) ===
    // Hindi/English/Urdu
    m.insert("lakh", LAKH);
    m.insert("lac", LAKH);
    m.insert("lakhs", LAKH);
    m.insert("लाख", LAKH);           // Hindi
    // Tamil
    m.insert("லட்சம்", LAKH);        // Tamil: latcham
    m.insert("latcham", LAKH);
    // Telugu
    m.insert("లక్ష", LAKH);          // Telugu: laksha
    m.insert("laksha", LAKH);
    // Kannada
    m.insert("ಲಕ್ಷ", LAKH);          // Kannada: laksha
    // Malayalam
    m.insert("ലക്ഷം", LAKH);         // Malayalam: laksham
    m.insert("laksham", LAKH);
    // Bengali
    m.insert("লাখ", LAKH);           // Bengali: lakh
    m.insert("লক্ষ", LAKH);          // Bengali: laksha (formal)
    // Gujarati
    m.insert("લાખ", LAKH);           // Gujarati: lakh
    // Marathi
    m.insert("लक्ष", LAKH);          // Marathi: laksha
    // Punjabi
    m.insert("ਲੱਖ", LAKH);           // Punjabi: lakkh
    // Odia
    m.insert("ଲକ୍ଷ", LAKH);          // Odia: laksha

    // === CRORE (1,00,00,000) ===
    // Hindi/English/Urdu
    m.insert("crore", CRORE);
    m.insert("cr", CRORE);
    m.insert("crores", CRORE);
    m.insert("करोड़", CRORE);         // Hindi
    m.insert("karod", CRORE);
    // Tamil
    m.insert("கோடி", CRORE);         // Tamil: kodi
    m.insert("kodi", CRORE);
    // Telugu
    m.insert("కోటి", CRORE);         // Telugu: koti
    m.insert("koti", CRORE);
    // Kannada
    m.insert("ಕೋಟಿ", CRORE);         // Kannada: koti
    // Malayalam
    m.insert("കോടി", CRORE);         // Malayalam: kodi
    // Bengali
    m.insert("কোটি", CRORE);         // Bengali: koti
    // Gujarati
    m.insert("કરોડ", CRORE);         // Gujarati: karod
    // Marathi
    m.insert("कोटी", CRORE);         // Marathi: koti
    // Punjabi
    m.insert("ਕਰੋੜ", CRORE);         // Punjabi: karor
    // Odia
    m.insert("କୋଟି", CRORE);         // Odia: koti

    // === THOUSAND (1,000) ===
    // Hindi/English/Urdu
    m.insert("thousand", THOUSAND);
    m.insert("k", THOUSAND);
    m.insert("हज़ार", THOUSAND);      // Hindi: hazar
    m.insert("हजार", THOUSAND);      // Hindi: hajar (alternate)
    m.insert("hazar", THOUSAND);
    m.insert("hazaar", THOUSAND);
    // Tamil
    m.insert("ஆயிரம்", THOUSAND);    // Tamil: aayiram
    m.insert("aayiram", THOUSAND);
    // Telugu
    m.insert("వేయి", THOUSAND);      // Telugu: veyi
    m.insert("వెయ్యి", THOUSAND);    // Telugu: veyyi
    m.insert("veyi", THOUSAND);
    // Kannada
    m.insert("ಸಾವಿರ", THOUSAND);     // Kannada: saavira
    m.insert("saavira", THOUSAND);
    // Malayalam
    m.insert("ആയിരം", THOUSAND);     // Malayalam: aayiram
    // Bengali
    m.insert("হাজার", THOUSAND);     // Bengali: hajar
    m.insert("hajar", THOUSAND);
    // Gujarati
    m.insert("હજાર", THOUSAND);      // Gujarati: hajar
    // Marathi
    m.insert("हजार", THOUSAND);      // Marathi: hajar
    // Punjabi
    m.insert("ਹਜ਼ਾਰ", THOUSAND);     // Punjabi: hazaar
    // Odia
    m.insert("ହଜାର", THOUSAND);      // Odia: hajara

    m
});

/// Multilingual number words (1-10 range) mapped to values
pub static NUMBER_WORDS: Lazy<HashMap<&'static str, f64>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // === HINDI (Romanized and Devanagari) ===
    m.insert("ek", 1.0);        m.insert("एक", 1.0);
    m.insert("do", 2.0);        m.insert("दो", 2.0);
    m.insert("teen", 3.0);      m.insert("तीन", 3.0);
    m.insert("char", 4.0);      m.insert("चार", 4.0);
    m.insert("paanch", 5.0);    m.insert("पांच", 5.0);
    m.insert("panch", 5.0);     m.insert("पाँच", 5.0);
    m.insert("chhe", 6.0);      m.insert("छह", 6.0);
    m.insert("saat", 7.0);      m.insert("सात", 7.0);
    m.insert("aath", 8.0);      m.insert("आठ", 8.0);
    m.insert("nau", 9.0);       m.insert("नौ", 9.0);
    m.insert("das", 10.0);      m.insert("दस", 10.0);

    // === TAMIL ===
    m.insert("ondru", 1.0);     m.insert("ஒன்று", 1.0);
    m.insert("irandu", 2.0);    m.insert("இரண்டு", 2.0);
    m.insert("moondru", 3.0);   m.insert("மூன்று", 3.0);
    m.insert("naangu", 4.0);    m.insert("நான்கு", 4.0);
    m.insert("ainthu", 5.0);    m.insert("ஐந்து", 5.0);
    m.insert("aaru", 6.0);      m.insert("ஆறு", 6.0);
    m.insert("ezhu", 7.0);      m.insert("ஏழு", 7.0);
    m.insert("ettu", 8.0);      m.insert("எட்டு", 8.0);
    m.insert("onbathu", 9.0);   m.insert("ஒன்பது", 9.0);
    m.insert("patthu", 10.0);   m.insert("பத்து", 10.0);

    // === TELUGU ===
    m.insert("okati", 1.0);     m.insert("ఒకటి", 1.0);
    m.insert("rendu", 2.0);     m.insert("రెండు", 2.0);
    m.insert("moodu", 3.0);     m.insert("మూడు", 3.0);
    m.insert("naalugu", 4.0);   m.insert("నాలుగు", 4.0);
    m.insert("aidu", 5.0);      m.insert("ఐదు", 5.0);
    m.insert("aaru", 6.0);      m.insert("ఆరు", 6.0);
    m.insert("edu", 7.0);       m.insert("ఏడు", 7.0);
    m.insert("enimidi", 8.0);   m.insert("ఎనిమిది", 8.0);
    m.insert("tommidi", 9.0);   m.insert("తొమ్మిది", 9.0);
    m.insert("padi", 10.0);     m.insert("పది", 10.0);

    // === BENGALI ===
    m.insert("এক", 1.0);        m.insert("দুই", 2.0);
    m.insert("তিন", 3.0);       m.insert("চার", 4.0);
    m.insert("পাঁচ", 5.0);      m.insert("ছয়", 6.0);
    m.insert("সাত", 7.0);       m.insert("আট", 8.0);
    m.insert("নয়", 9.0);        m.insert("দশ", 10.0);

    // Add more languages as needed...
    m
});

/// Extract amount from multilingual text
pub fn extract_multilingual_amount(text: &str) -> Option<f64> {
    use crate::indic_numerals::normalize_numerals;

    let normalized = normalize_numerals(text);
    let words: Vec<&str> = normalized.split_whitespace().collect();

    let mut total: f64 = 0.0;
    let mut current_num: Option<f64> = None;

    for word in words {
        // Check for multiplier words (case-insensitive for Latin)
        let lower = word.to_lowercase();

        // Check if it's a number word
        if let Some(&num) = NUMBER_WORDS.get(word) {
            current_num = Some(num);
            continue;
        }
        if let Some(&num) = NUMBER_WORDS.get(lower.as_str()) {
            current_num = Some(num);
            continue;
        }

        // Try to parse as numeric
        if let Ok(num) = word.parse::<f64>() {
            current_num = Some(num);
            continue;
        }

        // Check for multiplier words
        if let Some(&mult) = MULTIPLIER_WORDS.get(word) {
            total += current_num.unwrap_or(1.0) * mult;
            current_num = None;
            continue;
        }
        if let Some(&mult) = MULTIPLIER_WORDS.get(lower.as_str()) {
            total += current_num.unwrap_or(1.0) * mult;
            current_num = None;
            continue;
        }
    }

    // Add any remaining number
    if let Some(num) = current_num {
        total += num;
    }

    if total > 0.0 { Some(total) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hindi_amount() {
        assert_eq!(extract_multilingual_amount("पांच लाख"), Some(500_000.0));
        assert_eq!(extract_multilingual_amount("5 lakh"), Some(500_000.0));
    }

    #[test]
    fn test_tamil_amount() {
        assert_eq!(extract_multilingual_amount("ஐந்து லட்சம்"), Some(500_000.0));
        assert_eq!(extract_multilingual_amount("5 latcham"), Some(500_000.0));
    }

    #[test]
    fn test_telugu_amount() {
        assert_eq!(extract_multilingual_amount("ఐదు లక్ష"), Some(500_000.0));
    }

    #[test]
    fn test_mixed_script_amount() {
        // Devanagari numeral + English multiplier
        assert_eq!(extract_multilingual_amount("५ lakh"), Some(500_000.0));
    }
}
```

---

## Solution 3: Universal Script Detection

### Design: Detect All Major Indic Scripts

```rust
// crates/core/src/script_detect.rs (NEW FILE)

/// Detected script type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Script {
    Latin,
    Devanagari,
    Bengali,
    Tamil,
    Telugu,
    Kannada,
    Malayalam,
    Gujarati,
    Odia,
    Gurmukhi,
    Arabic,
    OlChiki,
    Unknown,
}

impl Script {
    /// Get Unicode range for this script
    pub fn range(&self) -> Option<(char, char)> {
        match self {
            Script::Devanagari => Some(('\u{0900}', '\u{097F}')),
            Script::Bengali => Some(('\u{0980}', '\u{09FF}')),
            Script::Gurmukhi => Some(('\u{0A00}', '\u{0A7F}')),
            Script::Gujarati => Some(('\u{0A80}', '\u{0AFF}')),
            Script::Odia => Some(('\u{0B00}', '\u{0B7F}')),
            Script::Tamil => Some(('\u{0B80}', '\u{0BFF}')),
            Script::Telugu => Some(('\u{0C00}', '\u{0C7F}')),
            Script::Kannada => Some(('\u{0C80}', '\u{0CFF}')),
            Script::Malayalam => Some(('\u{0D00}', '\u{0D7F}')),
            Script::Arabic => Some(('\u{0600}', '\u{06FF}')),
            Script::OlChiki => Some(('\u{1C50}', '\u{1C7F}')),
            _ => None,
        }
    }
}

/// Detect the dominant script in text
pub fn detect_script(text: &str) -> Script {
    let mut counts: std::collections::HashMap<Script, usize> = std::collections::HashMap::new();

    for c in text.chars() {
        let script = char_to_script(c);
        if script != Script::Unknown {
            *counts.entry(script).or_insert(0) += 1;
        }
    }

    counts.into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(script, _)| script)
        .unwrap_or(Script::Unknown)
}

/// Get script for a single character
pub fn char_to_script(c: char) -> Script {
    let code = c as u32;

    match code {
        // Latin (ASCII + Extended)
        0x0000..=0x007F | 0x0080..=0x00FF | 0x0100..=0x017F => Script::Latin,

        // Devanagari
        0x0900..=0x097F | 0xA8E0..=0xA8FF => Script::Devanagari,

        // Bengali/Assamese
        0x0980..=0x09FF => Script::Bengali,

        // Gurmukhi (Punjabi)
        0x0A00..=0x0A7F => Script::Gurmukhi,

        // Gujarati
        0x0A80..=0x0AFF => Script::Gujarati,

        // Odia
        0x0B00..=0x0B7F => Script::Odia,

        // Tamil
        0x0B80..=0x0BFF => Script::Tamil,

        // Telugu
        0x0C00..=0x0C7F => Script::Telugu,

        // Kannada
        0x0C80..=0x0CFF => Script::Kannada,

        // Malayalam
        0x0D00..=0x0D7F => Script::Malayalam,

        // Arabic (for Urdu)
        0x0600..=0x06FF | 0x0750..=0x077F => Script::Arabic,

        // Ol Chiki (Santali)
        0x1C50..=0x1C7F => Script::OlChiki,

        _ => Script::Unknown,
    }
}

/// Check if text contains any Indic script
pub fn has_indic_script(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(char_to_script(c),
            Script::Devanagari | Script::Bengali | Script::Tamil |
            Script::Telugu | Script::Kannada | Script::Malayalam |
            Script::Gujarati | Script::Odia | Script::Gurmukhi |
            Script::Arabic | Script::OlChiki
        )
    })
}

/// Get all scripts present in text
pub fn detect_scripts(text: &str) -> Vec<Script> {
    let mut scripts: std::collections::HashSet<Script> = std::collections::HashSet::new();

    for c in text.chars() {
        let script = char_to_script(c);
        if script != Script::Unknown {
            scripts.insert(script);
        }
    }

    scripts.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hindi() {
        assert_eq!(detect_script("नमस्ते"), Script::Devanagari);
    }

    #[test]
    fn test_tamil() {
        assert_eq!(detect_script("வணக்கம்"), Script::Tamil);
    }

    #[test]
    fn test_telugu() {
        assert_eq!(detect_script("నమస్కారం"), Script::Telugu);
    }

    #[test]
    fn test_code_mixed() {
        let scripts = detect_scripts("Hello नमस्ते");
        assert!(scripts.contains(&Script::Latin));
        assert!(scripts.contains(&Script::Devanagari));
    }
}
```

---

## Solution 4: Language-Agnostic Phone Number Validation

### Design: Support Indic Numeral Phone Numbers

```rust
// Update: crates/agent/src/intent.rs

use crate::indic_numerals::normalize_numerals;

/// Extract phone number from text (supports all Indic numerals)
pub fn extract_phone_number(text: &str) -> Option<String> {
    // First normalize all Indic numerals to ASCII
    let normalized = normalize_numerals(text);

    // Standard Indian phone regex
    let phone_regex = regex::Regex::new(r"[6-9]\d{9}").unwrap();

    phone_regex.find(&normalized)
        .map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devanagari_phone() {
        // Phone in Devanagari: 9876543210
        assert_eq!(
            extract_phone_number("मेरा नंबर ९८७६५४३२१० है"),
            Some("9876543210".to_string())
        );
    }

    #[test]
    fn test_tamil_phone() {
        // Phone in Tamil: 9876543210
        assert_eq!(
            extract_phone_number("என் எண் ௯௮௭௬௫௪௩௨௧௦"),
            Some("9876543210".to_string())
        );
    }

    #[test]
    fn test_mixed_phone() {
        // Mix of ASCII and Devanagari
        assert_eq!(
            extract_phone_number("call me at 98765४३२१०"),
            Some("9876543210".to_string())
        );
    }
}
```

---

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1)

| Task | File | Priority |
|------|------|----------|
| Create `indic_numerals.rs` | `crates/core/src/` | P0 |
| Create `script_detect.rs` | `crates/core/src/` | P0 |
| Add unit tests for all 11 scripts | `crates/core/src/` | P0 |
| Export from `crates/core/src/lib.rs` | | P0 |

### Phase 2: Agent Integration (Week 1-2)

| Task | File | Priority |
|------|------|----------|
| Create `multilingual_amounts.rs` | `crates/agent/src/` | P0 |
| Update `intent.rs` slot extraction | `crates/agent/src/` | P0 |
| Update phone extraction | `crates/agent/src/` | P1 |
| Add integration tests | `crates/agent/tests/` | P1 |

### Phase 3: RAG & Pipeline Updates (Week 2)

| Task | File | Priority |
|------|------|----------|
| Update `retriever.rs` text normalization | `crates/rag/src/` | P1 |
| Update `sparse_search.rs` tokenization | `crates/rag/src/` | P1 |
| Update `g2p.rs` to support all scripts | `crates/pipeline/src/tts/` | P2 |
| Add script-specific TTS handling | `crates/pipeline/src/tts/` | P2 |

### Phase 4: Testing & Validation (Week 2-3)

| Task | Description | Priority |
|------|-------------|----------|
| Unit tests for each script | 11 scripts × 5 tests | P0 |
| Integration tests | End-to-end multilingual | P1 |
| Test audio files | Record samples in 5+ languages | P1 |
| Performance benchmarks | Ensure no latency regression | P1 |

---

## Test Matrix

### Languages to Test

| Language | Script | Test Phrase | Expected Amount |
|----------|--------|-------------|-----------------|
| Hindi | Devanagari | "पांच लाख" | 500,000 |
| Tamil | Tamil | "ஐந்து லட்சம்" | 500,000 |
| Telugu | Telugu | "ఐదు లక్ష" | 500,000 |
| Bengali | Bengali | "পাঁচ লাখ" | 500,000 |
| Kannada | Kannada | "ಐದು ಲಕ್ಷ" | 500,000 |
| Malayalam | Malayalam | "അഞ്ച് ലക്ഷം" | 500,000 |
| Gujarati | Gujarati | "પાંચ લાખ" | 500,000 |
| Marathi | Devanagari | "पाच लक्ष" | 500,000 |
| Punjabi | Gurmukhi | "ਪੰਜ ਲੱਖ" | 500,000 |
| Odia | Odia | "ପାଞ୍ଚ ଲକ୍ଷ" | 500,000 |

### Phone Number Tests

| Script | Input | Expected |
|--------|-------|----------|
| Devanagari | ९८७६५४३२१० | 9876543210 |
| Bengali | ৯৮৭৬৫৪৩২১০ | 9876543210 |
| Tamil | ௯௮௭௬௫௪௩௨௧௦ | 9876543210 |
| Telugu | ౯౮౭౬౫౪౩౨౧౦ | 9876543210 |

---

## Architecture Impact

### New Files to Create

```
crates/core/src/
├── lib.rs              # Update exports
├── indic_numerals.rs   # NEW: Universal numeral handling
└── script_detect.rs    # NEW: Script detection

crates/agent/src/
├── lib.rs              # Update exports
└── multilingual_amounts.rs  # NEW: Multilingual amount extraction
```

### Files to Modify

```
crates/agent/src/intent.rs       # Use multilingual extraction
crates/rag/src/retriever.rs      # Unicode-safe normalization
crates/rag/src/sparse_search.rs  # Script-aware tokenization
crates/pipeline/src/tts/g2p.rs   # Extend to all scripts
```

---

## Key Design Principles

1. **Normalize Early**: Convert Indic numerals to ASCII at input boundary
2. **Script-Agnostic Core**: Core logic works with normalized text
3. **Preserve Original**: Keep original text for display, normalize for processing
4. **Fail Gracefully**: Unknown scripts fall back to Latin processing
5. **Extensible Maps**: Easy to add new languages/words via HashMaps

---

## Dependencies

No new external dependencies required. Uses:
- `unicode-segmentation` (already in use)
- `regex` (already in use)
- `once_cell` (already in use via `lazy_static`)

---

*This plan replaces the Hindi-only approach with a comprehensive multilingual solution supporting all 22 Indian languages that IndicConformer supports.*
