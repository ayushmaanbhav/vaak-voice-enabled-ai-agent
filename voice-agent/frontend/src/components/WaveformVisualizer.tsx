import React, { useEffect, useRef } from 'react';
import { theme } from '../styles/theme';
import type { AgentState, VadState } from '../types';

interface WaveformVisualizerProps {
  inputLevel: number;
  isMuted: boolean;
  agentState: AgentState;
  vadState: VadState;
  isSpeaking: boolean;
}

export const WaveformVisualizer: React.FC<WaveformVisualizerProps> = ({
  inputLevel,
  isMuted,
  agentState,
  vadState,
  isSpeaking,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number>(0);
  const barsRef = useRef<number[]>(Array(32).fill(0));

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const width = canvas.width;
    const height = canvas.height;
    const barCount = 32;
    const barWidth = (width - (barCount - 1) * 2) / barCount;
    const centerY = height / 2;

    const animate = () => {
      ctx.clearRect(0, 0, width, height);

      // Update bars based on input level
      const targetLevel = isMuted ? 0 : inputLevel;

      for (let i = 0; i < barCount; i++) {
        // Add some randomness and smoothing
        const randomFactor = 0.3 + Math.random() * 0.7;
        const targetHeight = targetLevel * height * 0.8 * randomFactor;

        // Smooth transition
        barsRef.current[i] += (targetHeight - barsRef.current[i]) * 0.2;

        const barHeight = Math.max(2, barsRef.current[i]);
        const x = i * (barWidth + 2);

        // Color based on state
        let color = theme.colors.textMuted;
        if (isSpeaking) {
          color = theme.colors.success;
        } else if (vadState === 'speech_active' || vadState === 'speech_detected') {
          color = theme.colors.kotakRed;
        } else if (agentState === 'processing') {
          color = theme.colors.warning;
        } else if (!isMuted && targetLevel > 0.01) {
          color = theme.colors.kotakRedLight;
        }

        // Draw bar (mirrored from center)
        ctx.fillStyle = color;
        ctx.fillRect(x, centerY - barHeight / 2, barWidth, barHeight);
      }

      animationRef.current = requestAnimationFrame(animate);
    };

    animate();

    return () => {
      cancelAnimationFrame(animationRef.current);
    };
  }, [inputLevel, isMuted, agentState, vadState, isSpeaking]);

  // Determine status text and color
  let statusText = 'Idle';
  let statusColor = theme.colors.textMuted;

  if (isMuted) {
    statusText = 'Muted';
    statusColor = theme.colors.error;
  } else if (isSpeaking) {
    statusText = 'Agent Speaking';
    statusColor = theme.colors.success;
  } else if (agentState === 'processing') {
    statusText = 'Processing...';
    statusColor = theme.colors.warning;
  } else if (vadState === 'speech_active') {
    statusText = 'Listening';
    statusColor = theme.colors.kotakRed;
  } else if (vadState === 'speech_detected') {
    statusText = 'Speech Detected';
    statusColor = theme.colors.kotakRedLight;
  } else if (agentState === 'listening') {
    statusText = 'Ready';
    statusColor = theme.colors.success;
  }

  return (
    <div style={styles.container}>
      <canvas ref={canvasRef} width={320} height={60} style={styles.canvas} />
      <div style={{ ...styles.status, color: statusColor }}>
        <div
          style={{
            ...styles.statusDot,
            backgroundColor: statusColor,
            animation: agentState !== 'idle' ? 'pulse 1.5s infinite' : 'none',
          }}
        />
        {statusText}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: theme.spacing.sm,
    padding: theme.spacing.md,
    background: theme.colors.bgCard,
    borderRadius: theme.borderRadius.lg,
  },
  canvas: {
    borderRadius: theme.borderRadius.md,
  },
  status: {
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing.sm,
    fontSize: theme.fontSize.sm,
    fontWeight: theme.fontWeight.medium,
  },
  statusDot: {
    width: '8px',
    height: '8px',
    borderRadius: '50%',
  },
};
