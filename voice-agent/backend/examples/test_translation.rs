//! Translation layer integration test
//! Run: cargo run --example test_translation

use voice_agent_text_processing::translation::{
    CandleIndicTrans2Config, CandleIndicTrans2Translator,
};
use voice_agent_core::{Language, Translator};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("=== Translation Layer Test ===\n");
    
    // Test with explicit config
    let config = CandleIndicTrans2Config {
        en_indic_path: PathBuf::from("models/translation/indictrans2-en-indic"),
        indic_en_path: PathBuf::from("models/translation/indictrans2-indic-en"),
        ..Default::default()
    };
    
    println!("Loading IndicTrans2 models...");
    let translator = CandleIndicTrans2Translator::new(config)?;
    println!("✓ Models loaded successfully!\n");
    
    // Test Hindi -> English
    println!("--- Test 1: Hindi → English ---");
    let hindi_text = "नमस्ते, आपका स्वागत है";
    println!("Input (Hindi): {}", hindi_text);
    let result = translator.translate(hindi_text, Language::Hindi, Language::English).await?;
    println!("Output (English): {}\n", result);
    
    // Test English -> Hindi
    println!("--- Test 2: English → Hindi ---");
    let english_text = "Hello, welcome to Kotak Gold Loan";
    println!("Input (English): {}", english_text);
    let result = translator.translate(english_text, Language::English, Language::Hindi).await?;
    println!("Output (Hindi): {}\n", result);
    
    // Test Tamil -> English
    println!("--- Test 3: Tamil → English ---");
    let tamil_text = "வணக்கம், தங்க கடன் பற்றி தெரிந்துகொள்ள வேண்டும்";
    println!("Input (Tamil): {}", tamil_text);
    let result = translator.translate(tamil_text, Language::Tamil, Language::English).await?;
    println!("Output (English): {}\n", result);
    
    // Test English -> Tamil
    println!("--- Test 4: English → Tamil ---");
    let english_text2 = "What is the interest rate for gold loan?";
    println!("Input (English): {}", english_text2);
    let result = translator.translate(english_text2, Language::English, Language::Tamil).await?;
    println!("Output (Tamil): {}\n", result);
    
    // Test language detection
    println!("--- Test 5: Language Detection ---");
    let texts = vec![
        ("Hello world", "English"),
        ("नमस्ते दुनिया", "Hindi"),
        ("வணக்கம் உலகம்", "Tamil"),
        ("నమస్కారం ప్రపంచం", "Telugu"),
    ];
    
    for (text, expected) in texts {
        let lang = translator.detect_language(text).await?;
        let status = if format!("{:?}", lang).to_lowercase() == expected.to_lowercase() { "✓" } else { "✗" };
        println!("{} Text: '{}' → Detected: {:?} (expected: {})", status, text, lang, expected);
    }
    
    println!("\n=== All Tests Complete ===");
    Ok(())
}
