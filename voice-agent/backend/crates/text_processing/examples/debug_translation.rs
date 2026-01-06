//! Debug translation to see what's happening

use voice_agent_text_processing::translation::{
    CandleIndicTrans2Config, CandleIndicTrans2Translator,
};
use voice_agent_core::{Language, Translator};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    println!("=== Debug Translation Test ===\n");
    
    let config = CandleIndicTrans2Config {
        en_indic_path: PathBuf::from("models/translation/indictrans2-en-indic"),
        indic_en_path: PathBuf::from("models/translation/indictrans2-indic-en"),
        ..Default::default()
    };
    
    println!("Loading models...");
    let translator = CandleIndicTrans2Translator::new(config)?;
    println!("✓ Models loaded!\n");
    
    // Simple test
    println!("--- English → Hindi ---");
    let result = translator.translate("hello", Language::English, Language::Hindi).await?;
    println!("'hello' → '{}'\n", result);
    
    println!("--- Hindi → English ---");  
    let result = translator.translate("नमस्ते", Language::Hindi, Language::English).await?;
    println!("'नमस्ते' → '{}'\n", result);
    
    println!("--- English → Tamil ---");
    let result = translator.translate("hello", Language::English, Language::Tamil).await?;
    println!("'hello' → '{}'", result);
    // Print raw bytes to see what script it actually is
    println!("  Bytes: {:?}\n", result.as_bytes());
    
    println!("--- Tamil → English ---");
    // Use a simple Tamil word
    let tamil = "வணக்கம்"; // vanakkam = hello
    let result = translator.translate(tamil, Language::Tamil, Language::English).await?;
    println!("'{}' → '{}'", tamil, result);
    println!("  Bytes: {:?}\n", result.as_bytes());
    
    Ok(())
}
