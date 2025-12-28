//! Speculative Execution for LLM
//!
//! Implements multiple speculative strategies:
//! - SLM-First: Use small model first, upgrade if complex
//! - Race Parallel: Run SLM and LLM in parallel, use first good response
//! - Hybrid Streaming: Start with SLM, switch to LLM mid-stream
//! - Draft-Verify: EAGLE-style draft tokens with LLM verification

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::timeout;
use parking_lot::Mutex;

use crate::backend::{LlmBackend, GenerationResult};
use crate::prompt::Message;
use crate::LlmError;

/// Speculative execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeculativeMode {
    /// SLM first, upgrade if complex
    SlmFirst,
    /// Race SLM and LLM in parallel
    RaceParallel,
    /// Hybrid streaming (start SLM, switch to LLM)
    HybridStreaming,
    /// EAGLE-style draft-verify
    DraftVerify,
}

/// Speculative execution configuration
#[derive(Debug, Clone)]
pub struct SpeculativeConfig {
    /// Execution mode
    pub mode: SpeculativeMode,
    /// Complexity threshold for SLM-first upgrade
    pub complexity_threshold: f32,
    /// Timeout for SLM response (ms)
    pub slm_timeout_ms: u64,
    /// Minimum tokens before considering switch (hybrid)
    pub min_tokens_before_switch: usize,
    /// Quality threshold for acceptance
    pub quality_threshold: f32,
    /// Enable fallback to LLM on error
    pub fallback_enabled: bool,
}

impl Default for SpeculativeConfig {
    fn default() -> Self {
        Self {
            mode: SpeculativeMode::SlmFirst,
            complexity_threshold: 0.7,
            // P0 FIX: Reduced from 2000ms to 200ms to meet 500ms E2E latency budget
            slm_timeout_ms: 200,
            min_tokens_before_switch: 10,
            quality_threshold: 0.8,
            fallback_enabled: true,
        }
    }
}

/// Result of speculative execution
#[derive(Debug, Clone)]
pub struct SpeculativeResult {
    /// Generated text
    pub text: String,
    /// Which model was used
    pub model_used: ModelUsed,
    /// Generation result
    pub generation: GenerationResult,
    /// Was fallback used?
    pub used_fallback: bool,
    /// Complexity score (if computed)
    pub complexity_score: Option<f32>,
}

/// Which model was used
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelUsed {
    Slm,
    Llm,
    Hybrid,
}

/// Speculative Executor
pub struct SpeculativeExecutor {
    slm: Arc<dyn LlmBackend>,
    llm: Arc<dyn LlmBackend>,
    config: SpeculativeConfig,
    /// Statistics
    stats: Mutex<SpeculativeStats>,
}

/// Statistics for speculative execution
#[derive(Debug, Clone, Default)]
pub struct SpeculativeStats {
    pub slm_calls: usize,
    pub llm_calls: usize,
    pub slm_successes: usize,
    pub llm_fallbacks: usize,
    pub avg_slm_time_ms: f32,
    pub avg_llm_time_ms: f32,
}

impl SpeculativeExecutor {
    /// Create a new speculative executor
    pub fn new(
        slm: Arc<dyn LlmBackend>,
        llm: Arc<dyn LlmBackend>,
        config: SpeculativeConfig,
    ) -> Self {
        Self {
            slm,
            llm,
            config,
            stats: Mutex::new(SpeculativeStats::default()),
        }
    }

    /// Execute with speculative strategy
    pub async fn execute(&self, messages: &[Message]) -> Result<SpeculativeResult, LlmError> {
        match self.config.mode {
            SpeculativeMode::SlmFirst => self.execute_slm_first(messages).await,
            SpeculativeMode::RaceParallel => self.execute_race_parallel(messages).await,
            SpeculativeMode::HybridStreaming => self.execute_hybrid_streaming(messages).await,
            SpeculativeMode::DraftVerify => self.execute_draft_verify(messages).await,
        }
    }

    /// Execute with streaming
    pub async fn execute_stream(
        &self,
        messages: &[Message],
        tx: mpsc::Sender<String>,
    ) -> Result<SpeculativeResult, LlmError> {
        // For now, use SLM-first with streaming
        match self.config.mode {
            SpeculativeMode::SlmFirst => {
                self.execute_slm_first_stream(messages, tx).await
            }
            SpeculativeMode::HybridStreaming => {
                self.execute_hybrid_streaming_with_output(messages, tx).await
            }
            _ => {
                // Fall back to non-streaming for other modes
                let result = self.execute(messages).await?;
                let _ = tx.send(result.text.clone()).await;
                Ok(result)
            }
        }
    }

    /// SLM-first strategy
    async fn execute_slm_first(&self, messages: &[Message]) -> Result<SpeculativeResult, LlmError> {
        let start = Instant::now();

        // Estimate complexity
        let complexity = self.estimate_complexity(messages);

        if complexity > self.config.complexity_threshold {
            // High complexity, go straight to LLM
            let result = self.llm.generate(messages).await?;
            self.update_stats(false, true, start.elapsed());

            return Ok(SpeculativeResult {
                text: result.text.clone(),
                model_used: ModelUsed::Llm,
                generation: result,
                used_fallback: false,
                complexity_score: Some(complexity),
            });
        }

        // Try SLM first with timeout
        let slm_timeout = Duration::from_millis(self.config.slm_timeout_ms);

        match timeout(slm_timeout, self.slm.generate(messages)).await {
            Ok(Ok(result)) => {
                // Check quality
                let quality = self.estimate_quality(&result.text, messages);

                if quality >= self.config.quality_threshold {
                    self.update_stats(true, false, start.elapsed());
                    Ok(SpeculativeResult {
                        text: result.text.clone(),
                        model_used: ModelUsed::Slm,
                        generation: result,
                        used_fallback: false,
                        complexity_score: Some(complexity),
                    })
                } else if self.config.fallback_enabled {
                    // Quality too low, fall back to LLM
                    let result = self.llm.generate(messages).await?;
                    self.update_stats(true, true, start.elapsed());

                    Ok(SpeculativeResult {
                        text: result.text.clone(),
                        model_used: ModelUsed::Llm,
                        generation: result,
                        used_fallback: true,
                        complexity_score: Some(complexity),
                    })
                } else {
                    self.update_stats(true, false, start.elapsed());
                    Ok(SpeculativeResult {
                        text: result.text.clone(),
                        model_used: ModelUsed::Slm,
                        generation: result,
                        used_fallback: false,
                        complexity_score: Some(complexity),
                    })
                }
            }
            Ok(Err(_e)) if self.config.fallback_enabled => {
                // SLM error, fall back to LLM
                let result = self.llm.generate(messages).await?;
                self.update_stats(true, true, start.elapsed());

                Ok(SpeculativeResult {
                    text: result.text.clone(),
                    model_used: ModelUsed::Llm,
                    generation: result,
                    used_fallback: true,
                    complexity_score: Some(complexity),
                })
            }
            Ok(Err(e)) => Err(e),
            Err(_) if self.config.fallback_enabled => {
                // Timeout, fall back to LLM
                let result = self.llm.generate(messages).await?;
                self.update_stats(true, true, start.elapsed());

                Ok(SpeculativeResult {
                    text: result.text.clone(),
                    model_used: ModelUsed::Llm,
                    generation: result,
                    used_fallback: true,
                    complexity_score: Some(complexity),
                })
            }
            Err(_) => Err(LlmError::Timeout),
        }
    }

    /// SLM-first with streaming
    async fn execute_slm_first_stream(
        &self,
        messages: &[Message],
        tx: mpsc::Sender<String>,
    ) -> Result<SpeculativeResult, LlmError> {
        let start = Instant::now();
        let complexity = self.estimate_complexity(messages);

        if complexity > self.config.complexity_threshold {
            let result = self.llm.generate_stream(messages, tx).await?;
            self.update_stats(false, true, start.elapsed());

            return Ok(SpeculativeResult {
                text: result.text.clone(),
                model_used: ModelUsed::Llm,
                generation: result,
                used_fallback: false,
                complexity_score: Some(complexity),
            });
        }

        let result = self.slm.generate_stream(messages, tx).await?;
        self.update_stats(true, false, start.elapsed());

        Ok(SpeculativeResult {
            text: result.text.clone(),
            model_used: ModelUsed::Slm,
            generation: result,
            used_fallback: false,
            complexity_score: Some(complexity),
        })
    }

    /// Race parallel strategy
    ///
    /// P0 FIX: Now properly aborts the losing model to save resources.
    /// Uses tokio::spawn with AbortHandle to cancel the slower model.
    async fn execute_race_parallel(&self, messages: &[Message]) -> Result<SpeculativeResult, LlmError> {
        let start = Instant::now();

        // Clone what we need for the spawned tasks
        let slm = self.slm.clone();
        let llm = self.llm.clone();
        let messages_for_slm = messages.to_vec();
        let messages_for_llm = messages.to_vec();

        // Spawn both as abortable tasks
        let slm_handle = tokio::spawn(async move {
            slm.generate(&messages_for_slm).await
        });

        let llm_handle = tokio::spawn(async move {
            llm.generate(&messages_for_llm).await
        });

        // P0 FIX: Get abort handles BEFORE select! (which moves the JoinHandles)
        let slm_abort = slm_handle.abort_handle();
        let llm_abort = llm_handle.abort_handle();

        // Use select to get first result and abort the other
        tokio::select! {
            slm_result = slm_handle => {
                // SLM finished first - abort LLM to save resources
                llm_abort.abort();
                tracing::debug!("SLM won race, aborting LLM");

                match slm_result {
                    Ok(Ok(result)) => {
                        let quality = self.estimate_quality(&result.text, messages);
                        if quality >= self.config.quality_threshold {
                            self.update_stats(true, false, start.elapsed());
                            Ok(SpeculativeResult {
                                text: result.text.clone(),
                                model_used: ModelUsed::Slm,
                                generation: result,
                                used_fallback: false,
                                complexity_score: None,
                            })
                        } else if self.config.fallback_enabled {
                            // Quality too low, need LLM after all
                            // Note: we already aborted the LLM task, need to start fresh
                            let llm_result = self.llm.generate(messages).await?;
                            self.update_stats(true, true, start.elapsed());
                            Ok(SpeculativeResult {
                                text: llm_result.text.clone(),
                                model_used: ModelUsed::Llm,
                                generation: llm_result,
                                used_fallback: true,
                                complexity_score: None,
                            })
                        } else {
                            self.update_stats(true, false, start.elapsed());
                            Ok(SpeculativeResult {
                                text: result.text.clone(),
                                model_used: ModelUsed::Slm,
                                generation: result,
                                used_fallback: false,
                                complexity_score: None,
                            })
                        }
                    }
                    Ok(Err(_)) if self.config.fallback_enabled => {
                        let llm_result = self.llm.generate(messages).await?;
                        self.update_stats(true, true, start.elapsed());
                        Ok(SpeculativeResult {
                            text: llm_result.text.clone(),
                            model_used: ModelUsed::Llm,
                            generation: llm_result,
                            used_fallback: true,
                            complexity_score: None,
                        })
                    }
                    Ok(Err(e)) => Err(e),
                    Err(e) => Err(LlmError::Generation(format!("SLM task panicked: {}", e))),
                }
            }
            llm_result = llm_handle => {
                // LLM finished first - abort SLM to save resources
                slm_abort.abort();
                tracing::debug!("LLM won race, aborting SLM");

                match llm_result {
                    Ok(Ok(result)) => {
                        self.update_stats(false, true, start.elapsed());
                        Ok(SpeculativeResult {
                            text: result.text.clone(),
                            model_used: ModelUsed::Llm,
                            generation: result,
                            used_fallback: false,
                            complexity_score: None,
                        })
                    }
                    Ok(Err(e)) => Err(e),
                    Err(e) => Err(LlmError::Generation(format!("LLM task panicked: {}", e))),
                }
            }
        }
    }

    /// Hybrid streaming strategy
    async fn execute_hybrid_streaming(&self, messages: &[Message]) -> Result<SpeculativeResult, LlmError> {
        // For non-streaming hybrid, just use SLM-first
        self.execute_slm_first(messages).await
    }

    /// Hybrid streaming with output
    async fn execute_hybrid_streaming_with_output(
        &self,
        messages: &[Message],
        tx: mpsc::Sender<String>,
    ) -> Result<SpeculativeResult, LlmError> {
        let start = Instant::now();

        // Start with SLM
        let (slm_tx, mut slm_rx) = mpsc::channel::<String>(100);

        let slm = self.slm.clone();
        let messages_clone = messages.to_vec();

        let slm_handle = tokio::spawn(async move {
            slm.generate_stream(&messages_clone, slm_tx).await
        });

        let mut tokens = Vec::new();
        let mut should_switch = false;

        // Collect initial tokens from SLM
        while let Some(token) = slm_rx.recv().await {
            tokens.push(token.clone());

            // Forward to output
            if tx.send(token).await.is_err() {
                break;
            }

            // Check if we should switch to LLM
            if tokens.len() >= self.config.min_tokens_before_switch {
                let quality = self.estimate_quality(&tokens.join(""), messages);
                if quality < self.config.quality_threshold * 0.8 {
                    should_switch = true;
                    break;
                }
            }
        }

        if should_switch && self.config.fallback_enabled {
            // Switch to LLM
            drop(slm_handle); // Cancel SLM

            // Continue with LLM
            let result = self.llm.generate_stream(messages, tx).await?;
            self.update_stats(true, true, start.elapsed());

            Ok(SpeculativeResult {
                text: result.text.clone(),
                model_used: ModelUsed::Hybrid,
                generation: result,
                used_fallback: true,
                complexity_score: None,
            })
        } else {
            // Continue with SLM
            let result = slm_handle.await
                .map_err(|e| LlmError::Generation(e.to_string()))??;

            self.update_stats(true, false, start.elapsed());

            Ok(SpeculativeResult {
                text: result.text.clone(),
                model_used: ModelUsed::Slm,
                generation: result,
                used_fallback: false,
                complexity_score: None,
            })
        }
    }

    /// Draft-verify strategy (EAGLE-style)
    ///
    /// **WARNING**: This implementation is NOT true EAGLE-style speculative decoding.
    /// It runs SLM then LLM sequentially, which DOUBLES latency instead of reducing it.
    /// True EAGLE-style requires:
    /// 1. SLM generates draft tokens
    /// 2. LLM verifies ALL draft tokens in a single forward pass (parallel verification)
    /// 3. Accept verified prefix, discard rejected suffix
    ///
    /// TODO: Implement proper speculative decoding with KV cache sharing.
    /// For now, prefer SlmFirst or RaceParallel modes.
    #[deprecated(note = "Doubles latency - use SlmFirst or RaceParallel instead")]
    async fn execute_draft_verify(&self, messages: &[Message]) -> Result<SpeculativeResult, LlmError> {
        tracing::warn!("DraftVerify mode doubles latency - consider using SlmFirst or RaceParallel");

        // Simplified draft-verify: use SLM to draft, LLM to verify/complete
        let start = Instant::now();

        // Generate draft with SLM
        let draft = self.slm.generate(messages).await?;

        // Verify with LLM by asking it to continue/correct
        let mut verify_messages = messages.to_vec();
        verify_messages.push(crate::prompt::Message {
            role: crate::prompt::Role::Assistant,
            content: draft.text.clone(),
        });

        // LLM continues/corrects
        let final_result = self.llm.generate(&verify_messages).await?;

        self.update_stats(true, true, start.elapsed());

        Ok(SpeculativeResult {
            text: format!("{}{}", draft.text, final_result.text),
            model_used: ModelUsed::Hybrid,
            generation: final_result,
            used_fallback: false,
            complexity_score: None,
        })
    }

    /// Estimate query complexity
    fn estimate_complexity(&self, messages: &[Message]) -> f32 {
        // Simple heuristics for complexity
        let empty = String::new();
        let last_message = messages.last()
            .map(|m| &m.content)
            .unwrap_or(&empty);

        let mut score: f32 = 0.0;

        // Length-based
        if last_message.len() > 200 {
            score += 0.2;
        }

        // Question words
        let complex_markers = [
            "explain", "analyze", "compare", "describe",
            "calculate", "summarize", "translate",
            "समझाइए", "विश्लेषण", "तुलना", // Hindi
        ];

        let lower = last_message.to_lowercase();
        for marker in &complex_markers {
            if lower.contains(marker) {
                score += 0.3;
            }
        }

        // Multiple questions
        if last_message.matches('?').count() > 1 {
            score += 0.2;
        }

        // Code/technical content
        if last_message.contains("```") || last_message.contains("code") {
            score += 0.3;
        }

        score.min(1.0)
    }

    /// Estimate response quality
    fn estimate_quality(&self, response: &str, _messages: &[Message]) -> f32 {
        // Simple quality heuristics
        let mut score: f32 = 1.0;

        // Too short
        if response.len() < 20 {
            score -= 0.3;
        }

        // Repetition detection
        let words: Vec<&str> = response.split_whitespace().collect();
        if words.len() > 5 {
            let unique: std::collections::HashSet<&str> = words.iter().cloned().collect();
            let repetition_ratio = unique.len() as f32 / words.len() as f32;
            if repetition_ratio < 0.5 {
                score -= 0.4;
            }
        }

        // Contains error markers
        let error_markers = ["sorry", "cannot", "error", "invalid"];
        for marker in &error_markers {
            if response.to_lowercase().contains(marker) {
                score -= 0.2;
            }
        }

        score.max(0.0)
    }

    /// Update statistics
    ///
    /// P2 FIX: Uses Welford's online algorithm for numerically stable mean updates.
    /// The formula `mean += (x - mean) / n` avoids accumulating floating-point errors
    /// that can occur with the naive `(mean * (n-1) + x) / n` formula.
    fn update_stats(&self, used_slm: bool, used_llm: bool, duration: Duration) {
        let mut stats = self.stats.lock();
        let duration_ms = duration.as_millis() as f32;

        if used_slm {
            stats.slm_calls += 1;
            if !used_llm {
                stats.slm_successes += 1;
            }
            // Welford's algorithm: mean += (x - mean) / n
            let delta = duration_ms - stats.avg_slm_time_ms;
            stats.avg_slm_time_ms += delta / stats.slm_calls as f32;
        }

        if used_llm {
            stats.llm_calls += 1;
            if used_slm {
                stats.llm_fallbacks += 1;
            }
            // Welford's algorithm: mean += (x - mean) / n
            let delta = duration_ms - stats.avg_llm_time_ms;
            stats.avg_llm_time_ms += delta / stats.llm_calls as f32;
        }
    }

    /// Get statistics
    pub fn stats(&self) -> SpeculativeStats {
        self.stats.lock().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.lock() = SpeculativeStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = SpeculativeConfig::default();
        assert_eq!(config.mode, SpeculativeMode::SlmFirst);
        assert!(config.fallback_enabled);
    }

    #[test]
    fn test_complexity_estimation() {
        // Would need mock backends to test properly
    }
}
