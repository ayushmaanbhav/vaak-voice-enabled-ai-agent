// Audio utilities for recording and playback

/**
 * Convert a Blob to base64 string
 */
export function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => {
      const base64 = reader.result as string;
      // Remove data URL prefix (e.g., "data:audio/webm;base64,")
      const base64Data = base64.split(',')[1];
      resolve(base64Data);
    };
    reader.onerror = reject;
    reader.readAsDataURL(blob);
  });
}

/**
 * Convert base64 PCM audio to AudioBuffer for playback
 * Backend sends 16-bit PCM at 16kHz
 */
export function base64PcmToAudioBuffer(
  base64: string,
  audioContext: AudioContext,
  sampleRate: number = 16000
): AudioBuffer {
  // Decode base64
  const binaryString = atob(base64);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }

  // Convert 16-bit PCM to float32
  const int16Array = new Int16Array(bytes.buffer);
  const float32Array = new Float32Array(int16Array.length);
  for (let i = 0; i < int16Array.length; i++) {
    float32Array[i] = int16Array[i] / 32768.0;
  }

  // Create AudioBuffer
  const audioBuffer = audioContext.createBuffer(1, float32Array.length, sampleRate);
  audioBuffer.getChannelData(0).set(float32Array);

  return audioBuffer;
}

/**
 * Create an AudioContext with proper configuration
 */
export function createAudioContext(): AudioContext {
  const AudioContextClass = window.AudioContext || (window as any).webkitAudioContext;
  return new AudioContextClass({ sampleRate: 16000 });
}

/**
 * Play an AudioBuffer
 */
export function playAudioBuffer(
  audioBuffer: AudioBuffer,
  audioContext: AudioContext,
  onEnded?: () => void
): AudioBufferSourceNode {
  const source = audioContext.createBufferSource();
  source.buffer = audioBuffer;
  source.connect(audioContext.destination);
  if (onEnded) {
    source.onended = onEnded;
  }
  source.start();
  return source;
}

/**
 * Get supported audio MIME type for MediaRecorder
 */
export function getSupportedMimeType(): string {
  const types = [
    'audio/webm;codecs=opus',
    'audio/webm',
    'audio/ogg;codecs=opus',
    'audio/mp4',
  ];

  for (const type of types) {
    if (MediaRecorder.isTypeSupported(type)) {
      return type;
    }
  }

  return 'audio/webm'; // fallback
}

/**
 * Request microphone permission and return stream
 */
export async function getMicrophoneStream(): Promise<MediaStream> {
  return navigator.mediaDevices.getUserMedia({
    audio: {
      channelCount: 1,
      sampleRate: 16000,
      echoCancellation: true,
      noiseSuppression: true,
      autoGainControl: true,
    },
  });
}

/**
 * AudioPlayer class for managing streaming audio playback
 */
export class AudioPlayer {
  private audioContext: AudioContext | null = null;
  private queue: AudioBuffer[] = [];
  private isPlaying: boolean = false;
  private currentSource: AudioBufferSourceNode | null = null;
  private onPlaybackComplete?: () => void;

  constructor() {
    this.audioContext = createAudioContext();
  }

  /**
   * Resume audio context (needed after user interaction)
   */
  async resume(): Promise<void> {
    if (this.audioContext?.state === 'suspended') {
      await this.audioContext.resume();
    }
  }

  /**
   * Add audio data to the queue and start playback
   */
  async addToQueue(base64Pcm: string): Promise<void> {
    if (!this.audioContext) return;

    await this.resume();

    try {
      const audioBuffer = base64PcmToAudioBuffer(base64Pcm, this.audioContext);
      this.queue.push(audioBuffer);

      if (!this.isPlaying) {
        this.playNext();
      }
    } catch (error) {
      console.error('Error adding audio to queue:', error);
    }
  }

  /**
   * Play the next buffer in the queue
   */
  private playNext(): void {
    if (!this.audioContext || this.queue.length === 0) {
      this.isPlaying = false;
      this.onPlaybackComplete?.();
      return;
    }

    this.isPlaying = true;
    const buffer = this.queue.shift()!;

    this.currentSource = playAudioBuffer(buffer, this.audioContext, () => {
      this.currentSource = null;
      this.playNext();
    });
  }

  /**
   * Stop all playback and clear queue
   */
  stop(): void {
    if (this.currentSource) {
      this.currentSource.stop();
      this.currentSource = null;
    }
    this.queue = [];
    this.isPlaying = false;
  }

  /**
   * Set callback for when all queued audio finishes playing
   */
  setOnPlaybackComplete(callback: () => void): void {
    this.onPlaybackComplete = callback;
  }

  /**
   * Check if currently playing
   */
  get playing(): boolean {
    return this.isPlaying;
  }

  /**
   * Cleanup resources
   */
  destroy(): void {
    this.stop();
    if (this.audioContext) {
      this.audioContext.close();
      this.audioContext = null;
    }
  }
}
