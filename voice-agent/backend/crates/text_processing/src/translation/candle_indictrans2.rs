//! IndicTrans2 Candle-based Translator
//!
//! Native Rust implementation using Candle for efficient translation
//! between English and 22 Indian languages.
//!
//! Uses two separate models:
//! - indictrans2-en-indic: English → Indic languages
//! - indictrans2-indic-en: Indic languages → English

#[cfg(feature = "candle")]
mod candle_impl {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    use async_trait::async_trait;
    use candle_core::{DType, Device, IndexOp, Module, Tensor, D};
    use candle_nn::{embedding, layer_norm, linear, linear_no_bias, Embedding, LayerNorm, Linear, VarBuilder};
    use futures::Stream;
    use parking_lot::RwLock;
    use sentencepiece::SentencePieceProcessor;
    use std::pin::Pin;

    use voice_agent_core::{Error, Language, Translator};

    use super::super::ScriptDetector;

    // Helper type for our public API
    type Result<T> = std::result::Result<T, Error>;

    // Internal result type using candle errors
    type CandleResult<T> = std::result::Result<T, candle_core::Error>;

    // Convert candle error to our error type
    fn to_translation_error(e: candle_core::Error) -> Error {
        Error::other(format!("Candle error: {}", e))
    }

    // Convert string error to our error type
    fn translation_error(msg: impl Into<String>) -> Error {
        Error::other(msg)
    }

    // ========================================================================
    // Configuration
    // ========================================================================

    /// IndicTrans2 model configuration
    #[derive(Debug, Clone)]
    pub struct IndicTrans2Config {
        pub encoder_layers: usize,
        pub decoder_layers: usize,
        pub encoder_embed_dim: usize,
        pub decoder_embed_dim: usize,
        pub encoder_ffn_dim: usize,
        pub decoder_ffn_dim: usize,
        pub encoder_attention_heads: usize,
        pub decoder_attention_heads: usize,
        pub encoder_vocab_size: usize,
        pub decoder_vocab_size: usize,
        pub max_source_positions: usize,
        pub max_target_positions: usize,
        pub pad_token_id: usize,
        pub bos_token_id: usize,
        pub eos_token_id: usize,
        pub decoder_start_token_id: usize,
        pub scale_embedding: bool,
        pub normalize_before: bool,
        pub layernorm_embedding: bool,
        pub dropout: f64,
        pub activation: Activation,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Activation {
        Gelu,
        Relu,
    }

    impl Default for IndicTrans2Config {
        fn default() -> Self {
            Self {
                encoder_layers: 18,
                decoder_layers: 18,
                encoder_embed_dim: 512,
                decoder_embed_dim: 512,
                encoder_ffn_dim: 2048,
                decoder_ffn_dim: 2048,
                encoder_attention_heads: 8,
                decoder_attention_heads: 8,
                encoder_vocab_size: 32322,
                decoder_vocab_size: 122672,
                max_source_positions: 256,
                max_target_positions: 256,
                pad_token_id: 1,
                bos_token_id: 0,
                eos_token_id: 2,
                decoder_start_token_id: 2,
                scale_embedding: true,
                normalize_before: true,
                layernorm_embedding: true,
                dropout: 0.0,
                activation: Activation::Gelu,
            }
        }
    }

    /// Translator configuration
    #[derive(Debug, Clone)]
    pub struct CandleIndicTrans2Config {
        pub en_indic_path: PathBuf,
        pub indic_en_path: PathBuf,
        pub device: Device,
        pub max_length: usize,
        pub num_beams: usize,
        pub cache_enabled: bool,
        pub cache_size: usize,
    }

    impl Default for CandleIndicTrans2Config {
        fn default() -> Self {
            Self {
                en_indic_path: PathBuf::from("models/translation/indictrans2-en-indic"),
                indic_en_path: PathBuf::from("models/translation/indictrans2-indic-en"),
                device: Device::Cpu,
                max_length: 256,
                num_beams: 1, // Greedy for now
                cache_enabled: true,
                cache_size: 1000,
            }
        }
    }

    // ========================================================================
    // Sinusoidal Positional Embedding
    // ========================================================================

    fn sinusoidal_position_embedding(
        num_positions: usize,
        embed_dim: usize,
        padding_idx: usize,
        device: &Device,
    ) -> CandleResult<Tensor> {
        let half_dim = embed_dim / 2;
        let emb_scale = (10000f64).ln() / (half_dim - 1) as f64;

        let positions: Vec<f32> = (0..num_positions).map(|i| i as f32).collect();
        let dim_scale: Vec<f32> = (0..half_dim)
            .map(|i| (-emb_scale * i as f64).exp() as f32)
            .collect();

        let positions = Tensor::new(positions, device)?;
        let dim_scale = Tensor::new(dim_scale, device)?;

        let positions = positions.unsqueeze(1)?;
        let dim_scale = dim_scale.unsqueeze(0)?;
        let emb = positions.broadcast_mul(&dim_scale)?;

        let sin_emb = emb.sin()?;
        let cos_emb = emb.cos()?;
        let mut emb = Tensor::cat(&[&sin_emb, &cos_emb], 1)?;

        if padding_idx < num_positions {
            let zeros = Tensor::zeros((1, embed_dim), DType::F32, device)?;
            let before = if padding_idx > 0 {
                Some(emb.i(0..padding_idx)?)
            } else {
                None
            };
            let after = if padding_idx < num_positions - 1 {
                Some(emb.i((padding_idx + 1)..)?)
            } else {
                None
            };

            let parts: Vec<&Tensor> = [before.as_ref(), Some(&zeros), after.as_ref()]
                .into_iter()
                .flatten()
                .collect();
            emb = Tensor::cat(&parts, 0)?;
        }

        Ok(emb)
    }

    // ========================================================================
    // Multi-Head Attention
    // ========================================================================

    struct MultiHeadAttention {
        q_proj: Linear,
        k_proj: Linear,
        v_proj: Linear,
        out_proj: Linear,
        num_heads: usize,
        head_dim: usize,
        scale: f64,
    }

    impl MultiHeadAttention {
        fn new(embed_dim: usize, num_heads: usize, vb: VarBuilder) -> CandleResult<Self> {
            let head_dim = embed_dim / num_heads;
            Ok(Self {
                q_proj: linear(embed_dim, embed_dim, vb.pp("q_proj"))?,
                k_proj: linear(embed_dim, embed_dim, vb.pp("k_proj"))?,
                v_proj: linear(embed_dim, embed_dim, vb.pp("v_proj"))?,
                out_proj: linear(embed_dim, embed_dim, vb.pp("out_proj"))?,
                num_heads,
                head_dim,
                scale: (head_dim as f64).powf(-0.5),
            })
        }

        fn forward(
            &self,
            query: &Tensor,
            key: &Tensor,
            value: &Tensor,
            attention_mask: Option<&Tensor>,
        ) -> CandleResult<Tensor> {
            let (batch_size, tgt_len, _) = query.dims3()?;
            let (_, src_len, _) = key.dims3()?;

            let q = self.q_proj.forward(query)?;
            let k = self.k_proj.forward(key)?;
            let v = self.v_proj.forward(value)?;

            let q = q
                .reshape((batch_size, tgt_len, self.num_heads, self.head_dim))?
                .transpose(1, 2)?;
            let k = k
                .reshape((batch_size, src_len, self.num_heads, self.head_dim))?
                .transpose(1, 2)?;
            let v = v
                .reshape((batch_size, src_len, self.num_heads, self.head_dim))?
                .transpose(1, 2)?;

            let attn_weights = (q.matmul(&k.transpose(2, 3)?)? * self.scale)?;

            let attn_weights = if let Some(mask) = attention_mask {
                attn_weights.broadcast_add(mask)?
            } else {
                attn_weights
            };

            let attn_weights = candle_nn::ops::softmax_last_dim(&attn_weights)?;
            let attn_output = attn_weights.matmul(&v)?;

            let attn_output = attn_output.transpose(1, 2)?.reshape((
                batch_size,
                tgt_len,
                self.num_heads * self.head_dim,
            ))?;

            self.out_proj.forward(&attn_output)
        }
    }

    // ========================================================================
    // Encoder Layer
    // ========================================================================

    struct EncoderLayer {
        self_attn: MultiHeadAttention,
        self_attn_layer_norm: LayerNorm,
        fc1: Linear,
        fc2: Linear,
        final_layer_norm: LayerNorm,
        normalize_before: bool,
        activation: Activation,
    }

    impl EncoderLayer {
        fn new(config: &IndicTrans2Config, vb: VarBuilder) -> CandleResult<Self> {
            let embed_dim = config.encoder_embed_dim;
            Ok(Self {
                self_attn: MultiHeadAttention::new(
                    embed_dim,
                    config.encoder_attention_heads,
                    vb.pp("self_attn"),
                )?,
                self_attn_layer_norm: layer_norm(embed_dim, 1e-5, vb.pp("self_attn_layer_norm"))?,
                fc1: linear(embed_dim, config.encoder_ffn_dim, vb.pp("fc1"))?,
                fc2: linear(config.encoder_ffn_dim, embed_dim, vb.pp("fc2"))?,
                final_layer_norm: layer_norm(embed_dim, 1e-5, vb.pp("final_layer_norm"))?,
                normalize_before: config.normalize_before,
                activation: config.activation,
            })
        }

        fn forward(
            &self,
            hidden_states: &Tensor,
            attention_mask: Option<&Tensor>,
        ) -> CandleResult<Tensor> {
            let residual = hidden_states.clone();

            let hidden_states = if self.normalize_before {
                self.self_attn_layer_norm.forward(hidden_states)?
            } else {
                hidden_states.clone()
            };

            let hidden_states = self.self_attn.forward(
                &hidden_states,
                &hidden_states,
                &hidden_states,
                attention_mask,
            )?;
            let hidden_states = (residual + hidden_states)?;

            let hidden_states = if !self.normalize_before {
                self.self_attn_layer_norm.forward(&hidden_states)?
            } else {
                hidden_states
            };

            let residual = hidden_states.clone();
            let hidden_states = if self.normalize_before {
                self.final_layer_norm.forward(&hidden_states)?
            } else {
                hidden_states
            };

            let hidden_states = self.fc1.forward(&hidden_states)?;
            let hidden_states = match self.activation {
                Activation::Gelu => hidden_states.gelu()?,
                Activation::Relu => hidden_states.relu()?,
            };
            let hidden_states = self.fc2.forward(&hidden_states)?;
            let hidden_states = (residual + hidden_states)?;

            if !self.normalize_before {
                self.final_layer_norm.forward(&hidden_states)
            } else {
                Ok(hidden_states)
            }
        }
    }

    // ========================================================================
    // Decoder Layer
    // ========================================================================

    struct DecoderLayer {
        self_attn: MultiHeadAttention,
        self_attn_layer_norm: LayerNorm,
        encoder_attn: MultiHeadAttention,
        encoder_attn_layer_norm: LayerNorm,
        fc1: Linear,
        fc2: Linear,
        final_layer_norm: LayerNorm,
        normalize_before: bool,
        activation: Activation,
    }

    impl DecoderLayer {
        fn new(config: &IndicTrans2Config, vb: VarBuilder) -> CandleResult<Self> {
            let embed_dim = config.decoder_embed_dim;
            Ok(Self {
                self_attn: MultiHeadAttention::new(
                    embed_dim,
                    config.decoder_attention_heads,
                    vb.pp("self_attn"),
                )?,
                self_attn_layer_norm: layer_norm(embed_dim, 1e-5, vb.pp("self_attn_layer_norm"))?,
                encoder_attn: MultiHeadAttention::new(
                    embed_dim,
                    config.decoder_attention_heads,
                    vb.pp("encoder_attn"),
                )?,
                encoder_attn_layer_norm: layer_norm(
                    embed_dim,
                    1e-5,
                    vb.pp("encoder_attn_layer_norm"),
                )?,
                fc1: linear(embed_dim, config.decoder_ffn_dim, vb.pp("fc1"))?,
                fc2: linear(config.decoder_ffn_dim, embed_dim, vb.pp("fc2"))?,
                final_layer_norm: layer_norm(embed_dim, 1e-5, vb.pp("final_layer_norm"))?,
                normalize_before: config.normalize_before,
                activation: config.activation,
            })
        }

        fn forward(
            &self,
            hidden_states: &Tensor,
            encoder_hidden_states: &Tensor,
            self_attn_mask: Option<&Tensor>,
            encoder_attn_mask: Option<&Tensor>,
        ) -> CandleResult<Tensor> {
            let residual = hidden_states.clone();
            let hidden_states = if self.normalize_before {
                self.self_attn_layer_norm.forward(hidden_states)?
            } else {
                hidden_states.clone()
            };

            let hidden_states = self.self_attn.forward(
                &hidden_states,
                &hidden_states,
                &hidden_states,
                self_attn_mask,
            )?;
            let hidden_states = (residual + hidden_states)?;
            let hidden_states = if !self.normalize_before {
                self.self_attn_layer_norm.forward(&hidden_states)?
            } else {
                hidden_states
            };

            let residual = hidden_states.clone();
            let hidden_states = if self.normalize_before {
                self.encoder_attn_layer_norm.forward(&hidden_states)?
            } else {
                hidden_states
            };

            let hidden_states = self.encoder_attn.forward(
                &hidden_states,
                encoder_hidden_states,
                encoder_hidden_states,
                encoder_attn_mask,
            )?;
            let hidden_states = (residual + hidden_states)?;
            let hidden_states = if !self.normalize_before {
                self.encoder_attn_layer_norm.forward(&hidden_states)?
            } else {
                hidden_states
            };

            let residual = hidden_states.clone();
            let hidden_states = if self.normalize_before {
                self.final_layer_norm.forward(&hidden_states)?
            } else {
                hidden_states
            };

            let hidden_states = self.fc1.forward(&hidden_states)?;
            let hidden_states = match self.activation {
                Activation::Gelu => hidden_states.gelu()?,
                Activation::Relu => hidden_states.relu()?,
            };
            let hidden_states = self.fc2.forward(&hidden_states)?;
            let hidden_states = (residual + hidden_states)?;

            if !self.normalize_before {
                self.final_layer_norm.forward(&hidden_states)
            } else {
                Ok(hidden_states)
            }
        }
    }

    // ========================================================================
    // Encoder
    // ========================================================================

    struct Encoder {
        embed_tokens: Embedding,
        embed_positions: Tensor,
        embed_scale: f64,
        layernorm_embedding: Option<LayerNorm>,
        layers: Vec<EncoderLayer>,
        layer_norm: Option<LayerNorm>,
        padding_idx: usize,
    }

    impl Encoder {
        fn new(config: &IndicTrans2Config, vb: VarBuilder, device: &Device) -> CandleResult<Self> {
            let embed_dim = config.encoder_embed_dim;
            let embed_scale = if config.scale_embedding {
                (embed_dim as f64).sqrt()
            } else {
                1.0
            };

            let embed_positions = sinusoidal_position_embedding(
                config.max_source_positions + 2,
                embed_dim,
                config.pad_token_id,
                device,
            )?;

            let layers: Vec<_> = (0..config.encoder_layers)
                .map(|i| EncoderLayer::new(config, vb.pp(format!("layers.{}", i))))
                .collect::<CandleResult<_>>()?;

            Ok(Self {
                embed_tokens: embedding(
                    config.encoder_vocab_size,
                    embed_dim,
                    vb.pp("embed_tokens"),
                )?,
                embed_positions,
                embed_scale,
                layernorm_embedding: if config.layernorm_embedding {
                    Some(layer_norm(embed_dim, 1e-5, vb.pp("layernorm_embedding"))?)
                } else {
                    None
                },
                layers,
                layer_norm: if config.normalize_before {
                    Some(layer_norm(embed_dim, 1e-5, vb.pp("layer_norm"))?)
                } else {
                    None
                },
                padding_idx: config.pad_token_id,
            })
        }

        fn forward(
            &self,
            input_ids: &Tensor,
            attention_mask: Option<&Tensor>,
        ) -> CandleResult<Tensor> {
            let (batch_size, seq_len) = input_ids.dims2()?;

            let inputs_embeds = self.embed_tokens.forward(input_ids)?;
            let inputs_embeds = (inputs_embeds * self.embed_scale)?;

            // Simple position IDs (0..seq_len + padding_idx + 1)
            let position_ids: Vec<u32> = (0..seq_len)
                .map(|i| (self.padding_idx + 1 + i) as u32)
                .collect();
            let position_ids = Tensor::new(position_ids.as_slice(), input_ids.device())?;
            let positions = self.embed_positions.index_select(&position_ids, 0)?;
            let embed_dim = positions.dim(1)?;
            let positions = positions
                .unsqueeze(0)?
                .broadcast_as((batch_size, seq_len, embed_dim))?;

            let mut hidden_states = inputs_embeds.add(&positions)?;

            if let Some(ref ln) = self.layernorm_embedding {
                hidden_states = ln.forward(&hidden_states)?;
            }

            let attention_mask = self.prepare_attention_mask(attention_mask, seq_len)?;

            for layer in &self.layers {
                hidden_states = layer.forward(&hidden_states, attention_mask.as_ref())?;
            }

            if let Some(ref ln) = self.layer_norm {
                hidden_states = ln.forward(&hidden_states)?;
            }

            Ok(hidden_states)
        }

        fn prepare_attention_mask(
            &self,
            attention_mask: Option<&Tensor>,
            _seq_len: usize,
        ) -> CandleResult<Option<Tensor>> {
            match attention_mask {
                Some(mask) => {
                    let mask = mask.unsqueeze(1)?.unsqueeze(1)?;
                    let ones = Tensor::ones_like(&mask)?;
                    let inverted = ones.sub(&mask.to_dtype(DType::F32)?)?;
                    let neg_inf = Tensor::new(&[f32::NEG_INFINITY], mask.device())?;
                    let mask = inverted.broadcast_mul(&neg_inf)?;
                    Ok(Some(mask))
                },
                None => Ok(None),
            }
        }
    }

    // ========================================================================
    // Decoder
    // ========================================================================

    struct Decoder {
        embed_tokens: Embedding,
        embed_positions: Tensor,
        embed_scale: f64,
        layernorm_embedding: Option<LayerNorm>,
        layers: Vec<DecoderLayer>,
        layer_norm: Option<LayerNorm>,
        padding_idx: usize,
    }

    impl Decoder {
        fn new(config: &IndicTrans2Config, vb: VarBuilder, device: &Device) -> CandleResult<Self> {
            let embed_dim = config.decoder_embed_dim;
            let embed_scale = if config.scale_embedding {
                (embed_dim as f64).sqrt()
            } else {
                1.0
            };

            let embed_positions = sinusoidal_position_embedding(
                config.max_target_positions + 2,
                embed_dim,
                config.pad_token_id,
                device,
            )?;

            let layers: Vec<_> = (0..config.decoder_layers)
                .map(|i| DecoderLayer::new(config, vb.pp(format!("layers.{}", i))))
                .collect::<CandleResult<_>>()?;

            Ok(Self {
                embed_tokens: embedding(
                    config.decoder_vocab_size,
                    embed_dim,
                    vb.pp("embed_tokens"),
                )?,
                embed_positions,
                embed_scale,
                layernorm_embedding: if config.layernorm_embedding {
                    Some(layer_norm(embed_dim, 1e-5, vb.pp("layernorm_embedding"))?)
                } else {
                    None
                },
                layers,
                layer_norm: if config.normalize_before {
                    Some(layer_norm(embed_dim, 1e-5, vb.pp("layer_norm"))?)
                } else {
                    None
                },
                padding_idx: config.pad_token_id,
            })
        }

        fn forward(
            &self,
            input_ids: &Tensor,
            encoder_hidden_states: &Tensor,
            encoder_attention_mask: Option<&Tensor>,
            past_length: usize,
        ) -> CandleResult<Tensor> {
            let (batch_size, seq_len) = input_ids.dims2()?;

            let inputs_embeds = self.embed_tokens.forward(input_ids)?;
            let inputs_embeds = (inputs_embeds * self.embed_scale)?;

            let positions: Vec<u32> = (0..seq_len)
                .map(|i| (self.padding_idx + 1 + past_length + i) as u32)
                .collect();
            let position_ids = Tensor::new(positions.as_slice(), input_ids.device())?;
            let positions = self.embed_positions.index_select(&position_ids, 0)?;
            let embed_dim = positions.dim(1)?;
            let positions = positions
                .unsqueeze(0)?
                .broadcast_as((batch_size, seq_len, embed_dim))?;

            let mut hidden_states = inputs_embeds.add(&positions)?;

            if let Some(ref ln) = self.layernorm_embedding {
                hidden_states = ln.forward(&hidden_states)?;
            }

            let causal_mask = self.create_causal_mask(seq_len, input_ids.device())?;
            let encoder_attention_mask =
                self.prepare_encoder_attention_mask(encoder_attention_mask)?;

            for layer in &self.layers {
                hidden_states = layer.forward(
                    &hidden_states,
                    encoder_hidden_states,
                    Some(&causal_mask),
                    encoder_attention_mask.as_ref(),
                )?;
            }

            if let Some(ref ln) = self.layer_norm {
                hidden_states = ln.forward(&hidden_states)?;
            }

            Ok(hidden_states)
        }

        fn create_causal_mask(&self, seq_len: usize, device: &Device) -> CandleResult<Tensor> {
            // Create causal mask: positions where we CAN'T attend become -inf
            // Lower triangular = 1 where i >= j (can attend to past and current)
            let mut mask_data = vec![0f32; seq_len * seq_len];
            for i in 0..seq_len {
                for j in 0..seq_len {
                    if j > i {
                        // Can't attend to future positions
                        mask_data[i * seq_len + j] = f32::NEG_INFINITY;
                    }
                }
            }
            let mask = Tensor::from_vec(mask_data, (seq_len, seq_len), device)?;
            mask.unsqueeze(0)?.unsqueeze(0)
        }

        fn prepare_encoder_attention_mask(
            &self,
            attention_mask: Option<&Tensor>,
        ) -> CandleResult<Option<Tensor>> {
            match attention_mask {
                Some(mask) => {
                    let mask = mask.unsqueeze(1)?.unsqueeze(1)?;
                    let ones = Tensor::ones_like(&mask)?;
                    let inverted = ones.sub(&mask.to_dtype(DType::F32)?)?;
                    let neg_inf = Tensor::new(&[f32::NEG_INFINITY], mask.device())?;
                    let mask = inverted.broadcast_mul(&neg_inf)?;
                    Ok(Some(mask))
                },
                None => Ok(None),
            }
        }
    }

    // ========================================================================
    // Full Model
    // ========================================================================

    struct IndicTrans2Model {
        encoder: Encoder,
        decoder: Decoder,
        lm_head: Linear,
        config: IndicTrans2Config,
    }

    impl IndicTrans2Model {
        fn new(config: IndicTrans2Config, vb: VarBuilder, device: &Device) -> CandleResult<Self> {
            Ok(Self {
                encoder: Encoder::new(&config, vb.pp("model.encoder"), device)?,
                decoder: Decoder::new(&config, vb.pp("model.decoder"), device)?,
                lm_head: linear_no_bias(
                    config.decoder_embed_dim,
                    config.decoder_vocab_size,
                    vb.pp("lm_head"),
                )?,
                config,
            })
        }

        fn generate(
            &self,
            input_ids: &Tensor,
            attention_mask: Option<&Tensor>,
            max_length: usize,
        ) -> CandleResult<Vec<u32>> {
            let encoder_output = self.encoder.forward(input_ids, attention_mask)?;

            let device = encoder_output.device();
            let mut output_ids = vec![self.config.decoder_start_token_id as u32];

            for _ in 0..max_length {
                let decoder_input = Tensor::new(output_ids.as_slice(), device)?.unsqueeze(0)?;

                let hidden_states =
                    self.decoder
                        .forward(&decoder_input, &encoder_output, attention_mask, 0)?;

                let logits = self.lm_head.forward(&hidden_states)?;
                let last_logits = logits.i((0, output_ids.len() - 1))?;
                let next_token = last_logits.argmax(D::Minus1)?.to_scalar::<u32>()?;

                if next_token == self.config.eos_token_id as u32 {
                    break;
                }

                output_ids.push(next_token);
            }

            Ok(output_ids)
        }
    }

    // ========================================================================
    // Tokenizer Wrapper
    // ========================================================================

    struct IndicTransTokenizer {
        src_spm: SentencePieceProcessor,
        tgt_spm: SentencePieceProcessor,
        src_vocab: HashMap<String, u32>,
        tgt_vocab: HashMap<String, u32>,
        tgt_vocab_rev: HashMap<u32, String>,
        unk_token_id: u32,
        pad_token_id: u32,
        eos_token_id: u32,
        bos_token_id: u32,
    }

    impl IndicTransTokenizer {
        fn new(model_path: &Path) -> Result<Self> {
            let src_spm = SentencePieceProcessor::open(model_path.join("model.SRC"))
                .map_err(|e| translation_error(format!("Failed to load source SPM: {}", e)))?;
            let tgt_spm = SentencePieceProcessor::open(model_path.join("model.TGT"))
                .map_err(|e| translation_error(format!("Failed to load target SPM: {}", e)))?;

            let src_vocab: HashMap<String, u32> = serde_json::from_str(
                &std::fs::read_to_string(model_path.join("dict.SRC.json")).map_err(|e| {
                    translation_error(format!("Failed to read source vocab: {}", e))
                })?,
            )
            .map_err(|e| translation_error(format!("Failed to parse source vocab: {}", e)))?;

            let tgt_vocab: HashMap<String, u32> = serde_json::from_str(
                &std::fs::read_to_string(model_path.join("dict.TGT.json")).map_err(|e| {
                    translation_error(format!("Failed to read target vocab: {}", e))
                })?,
            )
            .map_err(|e| translation_error(format!("Failed to parse target vocab: {}", e)))?;

            let tgt_vocab_rev: HashMap<u32, String> =
                tgt_vocab.iter().map(|(k, v)| (*v, k.clone())).collect();

            let unk_token_id = *src_vocab.get("<unk>").unwrap_or(&0);
            let pad_token_id = *src_vocab.get("<pad>").unwrap_or(&1);
            let eos_token_id = *src_vocab.get("</s>").unwrap_or(&2);
            let bos_token_id = *src_vocab.get("<s>").unwrap_or(&0);

            Ok(Self {
                src_spm,
                tgt_spm,
                src_vocab,
                tgt_vocab,
                tgt_vocab_rev,
                unk_token_id,
                pad_token_id,
                eos_token_id,
                bos_token_id,
            })
        }

        fn encode_source(&self, text: &str, src_lang: &str, tgt_lang: &str) -> Result<Vec<u32>> {
            let pieces = self
                .src_spm
                .encode(text)
                .map_err(|e| translation_error(format!("SPM encode failed: {}", e)))?;

            let mut ids = Vec::with_capacity(pieces.len() + 3);

            ids.push(*self.src_vocab.get(src_lang).unwrap_or(&self.unk_token_id));
            ids.push(*self.src_vocab.get(tgt_lang).unwrap_or(&self.unk_token_id));

            for piece in pieces.iter() {
                let id = *self
                    .src_vocab
                    .get(piece.piece.as_str())
                    .unwrap_or(&self.unk_token_id);
                ids.push(id);
            }

            ids.push(self.eos_token_id);

            Ok(ids)
        }

        fn decode_target(&self, ids: &[u32]) -> String {
            let pieces: Vec<String> = ids
                .iter()
                .filter(|&&id| {
                    id != self.pad_token_id && id != self.eos_token_id && id != self.bos_token_id
                })
                .map(|&id| {
                    self.tgt_vocab_rev
                        .get(&id)
                        .cloned()
                        .unwrap_or_else(|| "<unk>".to_string())
                })
                .collect();

            pieces.join("").replace("▁", " ").trim().to_string()
        }
    }

    // ========================================================================
    // Translation Cache
    // ========================================================================

    struct TranslationCache {
        entries: HashMap<String, String>,
        max_size: usize,
    }

    impl TranslationCache {
        fn new(max_size: usize) -> Self {
            Self {
                entries: HashMap::new(),
                max_size,
            }
        }

        fn make_key(text: &str, from: Language, to: Language) -> String {
            format!("{}:{}:{}", from, to, text)
        }

        fn get(&self, text: &str, from: Language, to: Language) -> Option<&str> {
            let key = Self::make_key(text, from, to);
            self.entries.get(&key).map(|s| s.as_str())
        }

        fn insert(&mut self, text: &str, from: Language, to: Language, translation: String) {
            if self.entries.len() >= self.max_size {
                let keys_to_remove: Vec<_> = self
                    .entries
                    .keys()
                    .take(self.max_size / 2)
                    .cloned()
                    .collect();
                for key in keys_to_remove {
                    self.entries.remove(&key);
                }
            }

            let key = Self::make_key(text, from, to);
            self.entries.insert(key, translation);
        }
    }

    // ========================================================================
    // Main Translator
    // ========================================================================

    /// IndicTrans2 translator using Candle
    pub struct CandleIndicTrans2Translator {
        en_indic_model: IndicTrans2Model,
        indic_en_model: IndicTrans2Model,
        en_indic_tokenizer: IndicTransTokenizer,
        indic_en_tokenizer: IndicTransTokenizer,
        config: CandleIndicTrans2Config,
        cache: RwLock<TranslationCache>,
        detector: ScriptDetector,
        device: Device,
    }

    impl CandleIndicTrans2Translator {
        pub fn new(config: CandleIndicTrans2Config) -> Result<Self> {
            let device = config.device.clone();

            tracing::info!(path = ?config.en_indic_path, "Loading en→indic model");
            let en_indic_model = Self::load_model(&config.en_indic_path, &device)?;
            let en_indic_tokenizer = IndicTransTokenizer::new(&config.en_indic_path)?;

            tracing::info!(path = ?config.indic_en_path, "Loading indic→en model");
            let indic_en_model = Self::load_model(&config.indic_en_path, &device)?;
            let indic_en_tokenizer = IndicTransTokenizer::new(&config.indic_en_path)?;

            Ok(Self {
                en_indic_model,
                indic_en_model,
                en_indic_tokenizer,
                indic_en_tokenizer,
                config: config.clone(),
                cache: RwLock::new(TranslationCache::new(config.cache_size)),
                detector: ScriptDetector::new(),
                device,
            })
        }

        fn load_model(path: &Path, device: &Device) -> Result<IndicTrans2Model> {
            let config_path = path.join("config.json");
            let config_str = std::fs::read_to_string(&config_path)
                .map_err(|e| translation_error(format!("Failed to read config: {}", e)))?;
            let config_json: serde_json::Value = serde_json::from_str(&config_str)
                .map_err(|e| translation_error(format!("Failed to parse config: {}", e)))?;

            let model_config = IndicTrans2Config {
                encoder_layers: config_json["encoder_layers"].as_u64().unwrap_or(18) as usize,
                decoder_layers: config_json["decoder_layers"].as_u64().unwrap_or(18) as usize,
                encoder_embed_dim: config_json["encoder_embed_dim"].as_u64().unwrap_or(512)
                    as usize,
                decoder_embed_dim: config_json["decoder_embed_dim"].as_u64().unwrap_or(512)
                    as usize,
                encoder_ffn_dim: config_json["encoder_ffn_dim"].as_u64().unwrap_or(2048) as usize,
                decoder_ffn_dim: config_json["decoder_ffn_dim"].as_u64().unwrap_or(2048) as usize,
                encoder_attention_heads: config_json["encoder_attention_heads"]
                    .as_u64()
                    .unwrap_or(8) as usize,
                decoder_attention_heads: config_json["decoder_attention_heads"]
                    .as_u64()
                    .unwrap_or(8) as usize,
                encoder_vocab_size: config_json["encoder_vocab_size"].as_u64().unwrap_or(32322)
                    as usize,
                decoder_vocab_size: config_json["decoder_vocab_size"].as_u64().unwrap_or(122672)
                    as usize,
                max_source_positions: config_json["max_source_positions"].as_u64().unwrap_or(256)
                    as usize,
                max_target_positions: config_json["max_target_positions"].as_u64().unwrap_or(256)
                    as usize,
                pad_token_id: config_json["pad_token_id"].as_u64().unwrap_or(1) as usize,
                bos_token_id: config_json["bos_token_id"].as_u64().unwrap_or(0) as usize,
                eos_token_id: config_json["eos_token_id"].as_u64().unwrap_or(2) as usize,
                decoder_start_token_id: config_json["decoder_start_token_id"].as_u64().unwrap_or(2)
                    as usize,
                scale_embedding: config_json["scale_embedding"].as_bool().unwrap_or(true),
                normalize_before: config_json["encoder_normalize_before"]
                    .as_bool()
                    .unwrap_or(true),
                layernorm_embedding: config_json["layernorm_embedding"].as_bool().unwrap_or(true),
                dropout: 0.0,
                activation: Activation::Gelu,
            };

            let weights_path = path.join("model.safetensors");

            // Load tensors from safetensors file
            let tensors = candle_core::safetensors::load(&weights_path, device)
                .map_err(|e| translation_error(format!("Failed to load weights: {}", e)))?;

            let vb = VarBuilder::from_tensors(tensors, DType::F32, device);

            IndicTrans2Model::new(model_config, vb, device).map_err(to_translation_error)
        }

        fn translate_internal(&self, text: &str, from: Language, to: Language) -> Result<String> {
            let is_en_to_indic = from == Language::English;

            let (model, tokenizer, src_code, tgt_code) = if is_en_to_indic {
                (
                    &self.en_indic_model,
                    &self.en_indic_tokenizer,
                    "eng_Latn",
                    language_to_indictrans_code(to),
                )
            } else {
                (
                    &self.indic_en_model,
                    &self.indic_en_tokenizer,
                    language_to_indictrans_code(from),
                    "eng_Latn",
                )
            };

            let input_ids = tokenizer.encode_source(text, src_code, tgt_code)?;
            let input_tensor = Tensor::new(input_ids.as_slice(), &self.device)
                .map_err(to_translation_error)?
                .unsqueeze(0)
                .map_err(to_translation_error)?;

            let output_ids = model
                .generate(&input_tensor, None, self.config.max_length)
                .map_err(to_translation_error)?;

            Ok(tokenizer.decode_target(&output_ids))
        }
    }

    fn language_to_indictrans_code(lang: Language) -> &'static str {
        match lang {
            Language::Hindi => "hin_Deva",
            Language::English => "eng_Latn",
            Language::Tamil => "tam_Taml",
            Language::Telugu => "tel_Telu",
            Language::Bengali => "ben_Beng",
            Language::Marathi => "mar_Deva",
            Language::Gujarati => "guj_Gujr",
            Language::Kannada => "kan_Knda",
            Language::Malayalam => "mal_Mlym",
            Language::Punjabi => "pan_Guru",
            Language::Odia => "ory_Orya",
            Language::Assamese => "asm_Beng",
            Language::Konkani => "kok_Deva",
            Language::Maithili => "mai_Deva",
            Language::Nepali => "npi_Deva",
            Language::Sanskrit => "san_Deva",
            Language::Sindhi => "snd_Arab",
            Language::Urdu => "urd_Arab",
            Language::Kashmiri => "kas_Arab",
            Language::Dogri => "doi_Deva",
            Language::Bodo => "brx_Deva",
            Language::Santali => "sat_Olck",
            Language::Manipuri => "mni_Beng",
        }
    }

    #[async_trait]
    impl Translator for CandleIndicTrans2Translator {
        async fn translate(
            &self,
            text: &str,
            from: Language,
            to: Language,
        ) -> voice_agent_core::Result<String> {
            if from == to {
                return Ok(text.to_string());
            }

            if self.config.cache_enabled {
                let cache = self.cache.read();
                if let Some(cached) = cache.get(text, from, to) {
                    return Ok(cached.to_string());
                }
            }

            let translation = self.translate_internal(text, from, to)?;

            if self.config.cache_enabled {
                let mut cache = self.cache.write();
                cache.insert(text, from, to, translation.clone());
            }

            Ok(translation)
        }

        async fn detect_language(&self, text: &str) -> voice_agent_core::Result<Language> {
            Ok(self.detector.detect(text))
        }

        fn translate_stream<'a>(
            &'a self,
            text_stream: Pin<Box<dyn Stream<Item = String> + Send + 'a>>,
            from: Language,
            to: Language,
        ) -> Pin<Box<dyn Stream<Item = voice_agent_core::Result<String>> + Send + 'a>> {
            use futures::StreamExt;
            Box::pin(
                text_stream.then(move |text| async move { self.translate(&text, from, to).await }),
            )
        }

        fn supports_pair(&self, from: Language, to: Language) -> bool {
            let is_english_source = from == Language::English;
            let is_english_target = to == Language::English;
            is_english_source != is_english_target
        }

        fn name(&self) -> &str {
            "indictrans2-candle"
        }
    }
}

#[cfg(feature = "candle")]
pub use candle_impl::{CandleIndicTrans2Config, CandleIndicTrans2Translator};

#[cfg(not(feature = "candle"))]
pub mod stub {
    use async_trait::async_trait;
    use futures::Stream;
    use std::path::PathBuf;
    use std::pin::Pin;

    use voice_agent_core::{Language, Translator};

    #[derive(Debug, Clone, Default)]
    pub struct CandleIndicTrans2Config {
        pub en_indic_path: PathBuf,
        pub indic_en_path: PathBuf,
    }

    pub struct CandleIndicTrans2Translator;

    impl CandleIndicTrans2Translator {
        pub fn new(_config: CandleIndicTrans2Config) -> voice_agent_core::Result<Self> {
            tracing::warn!("Candle feature not enabled - translation will pass through");
            Ok(Self)
        }
    }

    #[async_trait]
    impl Translator for CandleIndicTrans2Translator {
        async fn translate(
            &self,
            text: &str,
            _from: Language,
            _to: Language,
        ) -> voice_agent_core::Result<String> {
            Ok(text.to_string())
        }

        async fn detect_language(&self, _text: &str) -> voice_agent_core::Result<Language> {
            Ok(Language::English)
        }

        fn translate_stream<'a>(
            &'a self,
            text_stream: Pin<Box<dyn Stream<Item = String> + Send + 'a>>,
            _from: Language,
            _to: Language,
        ) -> Pin<Box<dyn Stream<Item = voice_agent_core::Result<String>> + Send + 'a>> {
            use futures::StreamExt;
            Box::pin(text_stream.map(Ok))
        }

        fn supports_pair(&self, _from: Language, _to: Language) -> bool {
            false
        }

        fn name(&self) -> &str {
            "indictrans2-stub"
        }
    }
}

#[cfg(not(feature = "candle"))]
pub use stub::{CandleIndicTrans2Config, CandleIndicTrans2Translator};
