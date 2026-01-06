#!/usr/bin/env python3
"""
IndicConformer Backend - Uses original ai4bharat model with working preprocessing.

This uses the original encoder.onnx + ctc_decoder.onnx (not FP16 quantized)
with the mel spectrogram preprocessing from hindi-quantized that we know works.
"""

import os
import numpy as np
import onnxruntime as ort
import torch
import torchaudio.transforms as T
from typing import Tuple, Optional, Dict, List

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
MODELS_DIR = os.path.join(SCRIPT_DIR, '..', '..', 'models', 'stt', 'indicconformer')


class IndicConformerCTC:
    """
    IndicConformer CTC inference using original ONNX models.

    Uses encoder.onnx + ctc_decoder.onnx with external weights.
    Preprocessing matches torchaudio MelSpectrogram (validated working).
    """

    # Supported languages and their vocab offsets in joint vocabulary
    LANGUAGES = {
        'as': 0,      # Assamese
        'bn': 257,    # Bengali
        'brx': 514,   # Bodo
        'doi': 771,   # Dogri
        'gu': 1028,   # Gujarati
        'hi': 1285,   # Hindi
        'kn': 1542,   # Kannada
        'kok': 1799,  # Konkani
        'ks': 2056,   # Kashmiri
        'mai': 2313,  # Maithili
        'ml': 2570,   # Malayalam
        'mni': 2827,  # Manipuri
        'mr': 3084,   # Marathi
        'ne': 3341,   # Nepali
        'or': 3598,   # Odia
        'pa': 3855,   # Punjabi
        'sa': 4112,   # Sanskrit
        'sat': 4369,  # Santali
        'sd': 4626,   # Sindhi
        'ta': 4883,   # Tamil
        'te': 5140,   # Telugu
        'ur': 5397,   # Urdu
    }

    def __init__(self, model_dir: str = None, language: str = 'hi'):
        self.model_dir = model_dir or MODELS_DIR
        self.language = language
        self.assets_dir = os.path.join(self.model_dir, 'assets')

        self.encoder = None
        self.decoder = None
        self.vocab = None
        self.mel_transform = None

    def _load_models(self):
        """Lazy load ONNX models."""
        if self.encoder is not None:
            return

        print(f"[IndicConformer] Loading models from {self.assets_dir}")

        # Session options
        opts = ort.SessionOptions()
        opts.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
        opts.intra_op_num_threads = 4

        # Load encoder (with external data)
        encoder_path = os.path.join(self.assets_dir, 'encoder.onnx')
        self.encoder = ort.InferenceSession(
            encoder_path,
            sess_options=opts,
            providers=['CPUExecutionProvider']
        )
        print(f"[IndicConformer] Encoder loaded")

        # Load CTC decoder
        decoder_path = os.path.join(self.assets_dir, 'ctc_decoder.onnx')
        self.decoder = ort.InferenceSession(
            decoder_path,
            sess_options=opts,
            providers=['CPUExecutionProvider']
        )
        print(f"[IndicConformer] CTC decoder loaded")

        # Load vocabulary
        self._load_vocab()

        # Create mel transform (matching hindi-quantized parameters)
        self.mel_transform = T.MelSpectrogram(
            sample_rate=16000,
            n_fft=512,
            win_length=400,
            hop_length=160,
            f_min=0.0,
            f_max=8000.0,
            n_mels=80,
            window_fn=torch.hann_window,
            power=2.0,
            normalized=False
        )

    def _load_vocab(self):
        """Load per-language vocabulary and language mask."""
        import json

        # Load per-language vocab from vocab.json (required)
        vocab_path = os.path.join(self.assets_dir, 'vocab.json')
        if not os.path.exists(vocab_path):
            raise FileNotFoundError(f"vocab.json not found in {self.assets_dir}")

        with open(vocab_path, 'r', encoding='utf-8') as f:
            vocab_data = json.load(f)

        if self.language not in vocab_data:
            raise ValueError(f"Language '{self.language}' not in vocab.json. Available: {list(vocab_data.keys())}")

        # vocab is a list of 257 tokens for this language
        self.vocab = vocab_data[self.language]
        print(f"[IndicConformer] Loaded {len(self.vocab)} tokens for {self.language}")

        # Load language mask (required for filtering CTC output)
        mask_path = os.path.join(self.assets_dir, 'language_masks.json')
        if not os.path.exists(mask_path):
            raise FileNotFoundError(f"language_masks.json not found in {self.assets_dir}")

        with open(mask_path, 'r', encoding='utf-8') as f:
            masks_data = json.load(f)

        if self.language not in masks_data:
            raise ValueError(f"Language '{self.language}' not in language_masks.json")

        # mask is a list of booleans [5633] - True for tokens belonging to this language
        self.language_mask = masks_data[self.language]
        true_count = sum(self.language_mask)
        print(f"[IndicConformer] Loaded language mask: {true_count} tokens enabled")

    def preprocess(self, audio: np.ndarray, sr: int = 16000) -> Tuple[np.ndarray, np.ndarray]:
        """
        Preprocess audio to mel spectrogram features.

        Matches hindi-quantized preprocessing exactly:
        - MelSpectrogram with power=2.0
        - Log transform with 1e-9 guard
        - Per-utterance normalization
        """
        self._load_models()

        # Resample if needed
        if sr != 16000:
            from librosa import resample
            audio = resample(audio, orig_sr=sr, target_sr=16000)

        # Convert to torch tensor
        audio_tensor = torch.from_numpy(audio).float()
        if audio_tensor.dim() == 1:
            audio_tensor = audio_tensor.unsqueeze(0)  # [1, samples]

        # Compute mel spectrogram
        mel_spec = self.mel_transform(audio_tensor)  # [1, 80, time]

        # Log transform
        mel_spec = torch.log(mel_spec + 1e-9)

        # Per-utterance normalization (global mean/std)
        mean = mel_spec.mean()
        std = mel_spec.std() + 1e-9
        mel_spec = (mel_spec - mean) / std

        # Convert to numpy float32 (model expects float32, not float16)
        features = mel_spec.numpy().astype(np.float32)
        length = np.array([features.shape[2]], dtype=np.int64)

        return features, length

    def decode_ctc(self, logprobs: np.ndarray) -> str:
        """
        Decode CTC output using greedy decoding with language mask filtering.

        The CTC decoder outputs logprobs over the joint vocabulary (5633 tokens).
        We filter to only this language's tokens (257) using the language mask,
        then decode using the per-language vocabulary.

        Args:
            logprobs: [batch, time, 5633] log probabilities over joint vocab

        Returns:
            Decoded text string
        """
        # Convert language mask to numpy boolean array for indexing
        mask = np.array(self.language_mask, dtype=bool)

        # Filter logprobs to only this language's tokens: [batch, time, 5633] -> [batch, time, 257]
        filtered_logprobs = logprobs[:, :, mask]

        # Apply log_softmax on filtered logits (re-normalize after filtering)
        filtered_logprobs = torch.from_numpy(filtered_logprobs).log_softmax(dim=-1).numpy()

        # Get best path from filtered logprobs
        predictions = np.argmax(filtered_logprobs, axis=-1).squeeze(0)  # [time]

        # CTC decode: remove blanks and repeats
        # In per-language vocab, blank_id = 256 (last token)
        blank_id = 256

        # Use unique_consecutive to collapse repeats
        decoded = []
        previous = -1  # Use -1 so first token is always added
        for p in predictions:
            if p != previous:
                decoded.append(int(p))
            previous = p

        # Convert to text using per-language vocabulary
        # Skip blank tokens (id=256)
        text_parts = []
        for idx in decoded:
            if idx == blank_id:
                continue
            if idx < len(self.vocab):
                token = self.vocab[idx]
                # Skip special tokens
                if token not in ['<unk>', '<blk>', '<blank>', '|']:
                    text_parts.append(token)

        text = ''.join(text_parts)
        # Replace SentencePiece underscore with space
        text = text.replace('▁', ' ').strip()

        return text

    def transcribe(self, audio: np.ndarray, sr: int = 16000) -> str:
        """
        Transcribe audio to text.

        Args:
            audio: Float32 audio samples [-1, 1]
            sr: Sample rate (will resample to 16kHz if different)

        Returns:
            Transcribed text
        """
        self._load_models()

        # Preprocess
        features, length = self.preprocess(audio, sr)

        # Run encoder
        encoder_inputs = {
            'audio_signal': features,
            'length': length
        }
        encoder_outputs = self.encoder.run(None, encoder_inputs)
        encoded = encoder_outputs[0]  # [batch, hidden, time]

        # Run CTC decoder
        decoder_inputs = {
            'encoder_output': encoded
        }
        decoder_outputs = self.decoder.run(None, decoder_inputs)
        logprobs = decoder_outputs[0]  # [batch, time, vocab]

        # Decode
        text = self.decode_ctc(logprobs)

        return text


def test():
    """Test the backend with sample audio."""
    import soundfile as sf

    # Find test audio
    test_files = [
        os.path.join(MODELS_DIR, '..', 'hindi-quantized', 'file.wav'),
        os.path.join(MODELS_DIR, 'test.wav'),
    ]

    test_file = None
    for f in test_files:
        if os.path.exists(f):
            test_file = f
            break

    if test_file is None:
        print("No test audio found")
        return

    print(f"Testing with: {test_file}")

    # Load audio
    audio, sr = sf.read(test_file)
    if audio.ndim > 1:
        audio = audio.mean(axis=1)
    audio = audio.astype(np.float32)

    # Transcribe
    model = IndicConformerCTC(language='hi')
    text = model.transcribe(audio, sr)

    print(f"Transcription: {text}")


if __name__ == '__main__':
    test()
