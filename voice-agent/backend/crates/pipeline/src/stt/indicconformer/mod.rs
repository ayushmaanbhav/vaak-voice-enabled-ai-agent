//! IndicConformer STT - Speech-to-Text for Indian Languages
//!
//! Implementation of AI4Bharat's IndicConformer 600M multilingual model.
//! Optimized for Hindi, Marathi, and other Indian languages.
//!
//! # Module Structure
//!
//! - `config`: Configuration structs
//! - `mel`: Mel filterbank for audio preprocessing
//! - `core`: Main STT implementation
//!
//! # Model Architecture
//!
//! - Mel spectrogram preprocessing (80 mel bins, 16kHz)
//! - Conformer encoder (encoder.onnx)
//! - CTC decoder (ctc_decoder.onnx)
//! - Language-specific post-net (joint_post_net_hi.onnx for Hindi)

mod config;
mod core;
mod mel;

// Re-export public types
pub use config::IndicConformerConfig;
pub use core::IndicConformerStt;
pub use mel::MelFilterbank;
