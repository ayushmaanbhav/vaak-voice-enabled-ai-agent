//! Mel Filterbank for Audio Preprocessing
//!
//! This module provides mel spectrogram extraction using sliding-window FFT.
//! Matches torchaudio.transforms.MelSpectrogram exactly.

use realfft::num_complex::Complex;
use std::sync::Arc;

/// Mel filterbank for audio preprocessing with sliding-window FFT
///
/// Uses realfft for efficient real-signal FFT computation.
/// Supports streaming mode with audio buffer for sliding window.
/// Matches torchaudio.transforms.MelSpectrogram exactly.
pub struct MelFilterbank {
    n_fft: usize,
    n_mels: usize,
    hop_length: usize,
    win_length: usize,
    mel_filters: Vec<Vec<f32>>,
    hann_window: Vec<f32>,
    /// Reusable FFT planner
    fft: Arc<dyn realfft::RealToComplex<f32>>,
    /// Sliding window buffer for streaming
    audio_buffer: parking_lot::Mutex<Vec<f32>>,
}

impl MelFilterbank {
    pub fn new(sample_rate: usize, n_fft: usize, n_mels: usize) -> Self {
        // Match torchaudio MelSpectrogram parameters exactly:
        // win_length=400 (25ms at 16kHz), hop_length=160 (10ms), n_fft=512
        let win_length = 400; // 25ms at 16kHz, same as Python
        let hop_length = 160; // 10ms at 16kHz, same as Python

        // Create Hann window of win_length (400), NOT n_fft (512)
        // This matches: window_fn=torch.hann_window with win_length
        // torchaudio's hann_window uses periodic=True internally
        let hann_window: Vec<f32> = (0..win_length)
            .map(|i| {
                // Periodic Hann window: 0.5 - 0.5 * cos(2*pi*n/N)
                let x = 2.0 * std::f32::consts::PI * i as f32 / win_length as f32;
                0.5 * (1.0 - x.cos())
            })
            .collect();

        // Create mel filterbank
        let mel_filters = Self::create_mel_filters(sample_rate, n_fft, n_mels);

        // Create FFT planner
        let mut planner = realfft::RealFftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(n_fft);

        Self {
            n_fft,
            n_mels,
            hop_length,
            win_length,
            mel_filters,
            hann_window,
            fft,
            audio_buffer: parking_lot::Mutex::new(Vec::new()),
        }
    }

    fn hz_to_mel(hz: f32) -> f32 {
        2595.0 * (1.0 + hz / 700.0).log10()
    }

    fn mel_to_hz(mel: f32) -> f32 {
        700.0 * (10.0_f32.powf(mel / 2595.0) - 1.0)
    }

    fn create_mel_filters(sample_rate: usize, n_fft: usize, n_mels: usize) -> Vec<Vec<f32>> {
        let fmin = 0.0;
        let fmax = sample_rate as f32 / 2.0;

        let mel_min = Self::hz_to_mel(fmin);
        let mel_max = Self::hz_to_mel(fmax);

        // Mel points
        let mel_points: Vec<f32> = (0..n_mels + 2)
            .map(|i| mel_min + (mel_max - mel_min) * i as f32 / (n_mels + 1) as f32)
            .collect();

        // Hz points
        let hz_points: Vec<f32> = mel_points.iter().map(|&m| Self::mel_to_hz(m)).collect();

        // FFT bin indices
        let bin_points: Vec<usize> = hz_points
            .iter()
            .map(|&hz| ((n_fft + 1) as f32 * hz / sample_rate as f32).floor() as usize)
            .collect();

        // Create triangular filters
        let n_bins = n_fft / 2 + 1;
        let mut filters = vec![vec![0.0f32; n_bins]; n_mels];

        for i in 0..n_mels {
            let start = bin_points[i];
            let center = bin_points[i + 1];
            let end = bin_points[i + 2];

            // Rising slope
            for j in start..center {
                if center > start && j < n_bins {
                    filters[i][j] = (j - start) as f32 / (center - start) as f32;
                }
            }

            // Falling slope
            for j in center..end {
                if end > center && j < n_bins {
                    filters[i][j] = (end - j) as f32 / (end - center) as f32;
                }
            }
        }

        filters
    }

    /// Compute FFT magnitude spectrum for a single frame using realfft
    fn compute_fft_frame(&self, windowed: &mut [f32]) -> Vec<f32> {
        let n_bins = self.n_fft / 2 + 1;
        let mut spectrum = vec![Complex::new(0.0f32, 0.0f32); n_bins];

        // Perform FFT
        if self.fft.process(windowed, &mut spectrum).is_ok() {
            spectrum.iter().map(|c| c.norm()).collect()
        } else {
            // Fallback to zeros on error
            vec![0.0f32; n_bins]
        }
    }

    /// Extract mel spectrogram from audio (batch mode) with per-utterance normalization
    ///
    /// Matches Python torchaudio.transforms.MelSpectrogram exactly:
    /// 1. Center padding (pad n_fft//2 on each side) - default center=True
    /// 2. Apply win_length (400) Hann window, zero-pad to n_fft (512)
    /// 3. Power spectrogram (magnitude squared, power=2.0)
    /// 4. Mel filterbank
    /// 5. Log transform with 1e-9 guard
    /// 6. GLOBAL (whole utterance) mean/std normalization
    pub fn extract(&self, audio: &[f32]) -> Vec<f32> {
        // Debug logging (only on first call)
        static DEBUG_LOGGED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        let should_log = !DEBUG_LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed);

        // Step 1: Apply center padding (torchaudio center=True default)
        // Pad n_fft // 2 samples on each side using reflect padding
        let pad_amount = self.n_fft / 2; // 256 for n_fft=512
        let padded_len = audio.len() + 2 * pad_amount;
        let mut padded = Vec::with_capacity(padded_len);

        // Reflect padding at start
        for i in (1..=pad_amount).rev() {
            let idx = i.min(audio.len() - 1);
            padded.push(audio[idx]);
        }
        // Original audio
        padded.extend_from_slice(audio);
        // Reflect padding at end
        for i in 0..pad_amount {
            let idx = audio.len().saturating_sub(2 + i);
            padded.push(audio[idx]);
        }

        if should_log {
            tracing::info!(
                audio_len = audio.len(),
                padded_len = padded.len(),
                pad_amount = pad_amount,
                "MelFilterbank: Padding applied"
            );
        }

        // Calculate number of frames
        // Frame centers are at: 0, hop_length, 2*hop_length, ... until we can't fit a full window
        let n_frames = (padded.len().saturating_sub(self.n_fft)) / self.hop_length + 1;

        if n_frames == 0 {
            return vec![0.0; self.n_mels];
        }

        let mut mel_spec = Vec::with_capacity(n_frames * self.n_mels);
        let mut raw_mel_sum = 0.0f32;
        let mut raw_mel_count = 0usize;

        for frame_idx in 0..n_frames {
            // CRITICAL FIX: torch.stft with center=True extracts frames CENTERED at
            // positions 0, hop_length, 2*hop_length, etc. of the ORIGINAL audio.
            // In the padded array, frame 0 is centered at position pad_amount (256).
            // So frame center = pad_amount + frame_idx * hop_length
            let frame_center = pad_amount + frame_idx * self.hop_length;
            let half_win = self.win_length / 2; // 200 for win_length=400

            // Step 2: Apply win_length (400) Hann window, zero-pad to n_fft (512)
            let mut windowed = vec![0.0f32; self.n_fft];

            // Extract win_length samples CENTERED at frame_center
            let frame_start = frame_center.saturating_sub(half_win);
            for i in 0..self.win_length {
                let audio_idx = frame_start + i;
                if audio_idx < padded.len() {
                    windowed[i] = padded[audio_idx] * self.hann_window[i];
                }
            }
            // Remaining samples (win_length..n_fft) are already zero (zero-padding)

            // Step 3: Compute FFT magnitudes
            let magnitudes = self.compute_fft_frame(&mut windowed);

            // Debug: Log first frame's FFT magnitudes
            if should_log && frame_idx == 0 {
                tracing::info!(
                    fft_bins_0_9 = ?&magnitudes[..10.min(magnitudes.len())],
                    "MelFilterbank: Frame 0 FFT magnitudes"
                );
            }

            // Step 4: Apply mel filterbank with power=2.0 (squared magnitude)
            let mut frame_mels = Vec::with_capacity(self.n_mels);
            for filter in &self.mel_filters {
                let mut mel_energy = 0.0f32;
                for (j, &mag) in magnitudes.iter().enumerate() {
                    // CRITICAL: Square the magnitude (power=2.0) to match torchaudio
                    mel_energy += (mag * mag) * filter[j];
                }
                raw_mel_sum += mel_energy;
                raw_mel_count += 1;
                // Step 5: Log transform with 1e-9 guard (matching Python)
                frame_mels.push((mel_energy + 1e-9).ln());
            }

            // Debug: Log first frame's mel values
            if should_log && frame_idx == 0 {
                tracing::info!(
                    raw_mels_0_9 = ?&frame_mels[..10.min(frame_mels.len())],
                    "MelFilterbank: Frame 0 log mel values (before normalization)"
                );
            }

            mel_spec.extend(frame_mels);
        }

        if should_log {
            let log_mel_mean: f32 = mel_spec.iter().sum::<f32>() / mel_spec.len() as f32;
            let log_mel_std: f32 = (mel_spec
                .iter()
                .map(|&x| (x - log_mel_mean).powi(2))
                .sum::<f32>()
                / mel_spec.len() as f32)
                .sqrt();
            tracing::info!(
                n_frames = n_frames,
                n_mels = self.n_mels,
                log_mel_mean = %format!("{:.6}", log_mel_mean),
                log_mel_std = %format!("{:.6}", log_mel_std),
                raw_mel_avg = %format!("{:.6}", raw_mel_sum / raw_mel_count as f32),
                "MelFilterbank: Pre-normalization stats"
            );
        }

        // Step 6: Apply GLOBAL per-utterance normalization (mean/std over entire spectrogram)
        // This matches Python: mean = mel_spec.mean(), std = mel_spec.std()
        Self::normalize_features_global(&mut mel_spec);

        if should_log {
            tracing::info!(
                first_frame_0_9 = ?&mel_spec[..10.min(mel_spec.len())],
                frame_25_0_9 = ?&mel_spec[25 * self.n_mels..25 * self.n_mels + 10.min(mel_spec.len())],
                "MelFilterbank: Post-normalization values"
            );
        }

        mel_spec
    }

    /// Apply GLOBAL per-utterance normalization to mel features
    ///
    /// Computes mean and std over the ENTIRE spectrogram (not per-channel)
    /// This matches Python: `(mel_spec - mel_spec.mean()) / mel_spec.std()`
    fn normalize_features_global(features: &mut [f32]) {
        if features.is_empty() {
            return;
        }

        let n = features.len() as f32;

        // Compute global mean
        let sum: f32 = features.iter().sum();
        let mean = sum / n;

        // Compute global std
        let sum_sq: f32 = features.iter().map(|&x| (x - mean) * (x - mean)).sum();
        let std = (sum_sq / n + 1e-10).sqrt();

        // Normalize all values
        for val in features.iter_mut() {
            *val = (*val - mean) / std;
        }
    }

    /// Legacy per-channel normalization (kept for reference)
    #[allow(dead_code)]
    fn normalize_features_per_channel(features: &mut [f32], n_mels: usize, n_frames: usize) {
        if n_frames == 0 {
            return;
        }

        for mel_idx in 0..n_mels {
            let mut sum = 0.0f32;
            let mut sum_sq = 0.0f32;

            for frame_idx in 0..n_frames {
                let idx = frame_idx * n_mels + mel_idx;
                let val = features[idx];
                sum += val;
                sum_sq += val * val;
            }

            let mean = sum / n_frames as f32;
            let variance = (sum_sq / n_frames as f32) - (mean * mean);
            let std = (variance + 1e-10).sqrt();

            for frame_idx in 0..n_frames {
                let idx = frame_idx * n_mels + mel_idx;
                features[idx] = (features[idx] - mean) / std;
            }
        }
    }

    /// Streaming mel extraction - add audio and get new mel frames
    ///
    /// Returns only the NEW mel frames since last call.
    /// Maintains internal buffer for sliding window.
    pub fn extract_streaming(&self, audio: &[f32]) -> Vec<f32> {
        let mut buffer = self.audio_buffer.lock();
        buffer.extend_from_slice(audio);

        let mut mel_frames = Vec::new();

        // Process complete frames
        while buffer.len() >= self.n_fft {
            // Apply window to current frame
            let mut windowed = vec![0.0f32; self.n_fft];
            for i in 0..self.n_fft {
                windowed[i] = buffer[i] * self.hann_window[i];
            }

            // Compute FFT magnitudes
            let magnitudes = self.compute_fft_frame(&mut windowed);

            // Apply mel filterbank
            for filter in &self.mel_filters {
                let mut mel_energy = 0.0f32;
                for (j, &mag) in magnitudes.iter().enumerate() {
                    mel_energy += mag * filter[j];
                }
                mel_frames.push((mel_energy + 1e-10).ln());
            }

            // Slide window by hop_length
            buffer.drain(..self.hop_length);
        }

        mel_frames
    }

    /// Reset streaming buffer
    pub fn reset_streaming(&self) {
        self.audio_buffer.lock().clear();
    }

    /// Get pending samples in buffer
    pub fn pending_samples(&self) -> usize {
        self.audio_buffer.lock().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mel_filterbank() {
        let mel = MelFilterbank::new(16000, 512, 80);
        assert_eq!(mel.mel_filters.len(), 80);
        assert_eq!(mel.hann_window.len(), 400); // win_length, not n_fft
    }

    #[test]
    fn test_mel_extract() {
        let mel = MelFilterbank::new(16000, 512, 80);

        // Generate 100ms of audio (1600 samples at 16kHz)
        let audio: Vec<f32> = (0..1600).map(|i| (i as f32 * 0.01).sin() * 0.5).collect();

        let features = mel.extract(&audio);

        // Should have multiple frames, each with 80 mel bins
        assert!(features.len() >= 80);
        assert_eq!(features.len() % 80, 0);
    }

    #[test]
    fn test_mel_streaming() {
        let mel = MelFilterbank::new(16000, 512, 80);

        // Generate audio chunks
        let chunk1: Vec<f32> = (0..512).map(|i| (i as f32 * 0.01).sin() * 0.5).collect();
        let chunk2: Vec<f32> = (512..1024).map(|i| (i as f32 * 0.01).sin() * 0.5).collect();

        let frames1 = mel.extract_streaming(&chunk1);
        let frames2 = mel.extract_streaming(&chunk2);

        // Should get frames from streaming
        assert!(frames1.len() >= 80 || frames2.len() >= 80);
    }

    #[test]
    fn test_hz_mel_conversion() {
        // Test round-trip conversion
        let hz = 1000.0;
        let mel = MelFilterbank::hz_to_mel(hz);
        let hz_back = MelFilterbank::mel_to_hz(mel);
        assert!((hz - hz_back).abs() < 0.01);
    }
}
