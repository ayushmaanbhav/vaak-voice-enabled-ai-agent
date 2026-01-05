import React from 'react';
import { theme } from '../styles/theme';
import type { Metrics, AgentState, VadState } from '../types';

interface MetricsPanelProps {
  metrics: Metrics;
  agentState: AgentState;
  vadState: VadState;
  isSpeaking: boolean;
  audioMode?: 'webrtc' | 'websocket';
}

export const MetricsPanel: React.FC<MetricsPanelProps> = ({
  metrics,
  agentState,
  vadState,
  isSpeaking,
  audioMode = 'websocket',
}) => {
  const formatLatency = (ms: number | undefined): string => {
    if (ms === undefined) return '--';
    return `${ms}ms`;
  };

  const getLatencyColor = (ms: number | undefined): string => {
    if (ms === undefined) return theme.colors.textMuted;
    if (ms < 200) return theme.colors.success;
    if (ms < 500) return theme.colors.warning;
    return theme.colors.error;
  };

  // Get system state display
  const getSystemState = (): { label: string; color: string } => {
    if (isSpeaking) {
      return { label: 'Speaking', color: theme.colors.success };
    }
    switch (agentState) {
      case 'listening':
        return { label: 'Listening', color: theme.colors.kotakRed };
      case 'processing':
        return { label: 'Processing', color: theme.colors.warning };
      case 'speaking':
        return { label: 'Speaking', color: theme.colors.success };
      default:
        return { label: 'Idle', color: theme.colors.textMuted };
    }
  };

  // Get speaker indicator
  const getSpeaker = (): { label: string; color: string } => {
    if (isSpeaking) {
      return { label: 'Agent', color: theme.colors.success };
    }
    if (vadState === 'speech_active' || vadState === 'speech_detected') {
      return { label: 'User', color: theme.colors.kotakRed };
    }
    return { label: 'None', color: theme.colors.textMuted };
  };

  const systemState = getSystemState();
  const speaker = getSpeaker();

  return (
    <div style={styles.container}>
      <h4 style={styles.title}>Status</h4>

      {/* Status indicators */}
      <div style={styles.statusGrid}>
        <div style={styles.statusItem}>
          <div style={styles.statusLabel}>Audio</div>
          <div
            style={{
              ...styles.statusValue,
              color: theme.colors.success,
            }}
          >
            {audioMode === 'websocket' ? 'WebSocket' : 'WebRTC'}
          </div>
        </div>

        <div style={styles.statusItem}>
          <div style={styles.statusLabel}>State</div>
          <div
            style={{
              ...styles.statusValue,
              color: systemState.color,
            }}
          >
            {systemState.label}
          </div>
        </div>

        <div style={styles.statusItem}>
          <div style={styles.statusLabel}>Speaking</div>
          <div
            style={{
              ...styles.statusValue,
              color: speaker.color,
            }}
          >
            {speaker.label}
          </div>
        </div>

        <div style={styles.statusItem}>
          <div style={styles.statusLabel}>VAD</div>
          <div
            style={{
              ...styles.statusValue,
              color:
                vadState === 'speech_active'
                  ? theme.colors.success
                  : vadState === 'speech_detected'
                    ? theme.colors.warning
                    : theme.colors.textMuted,
            }}
          >
            {vadState === 'speech_active'
              ? 'Active'
              : vadState === 'speech_detected'
                ? 'Detected'
                : vadState === 'speech_ended'
                  ? 'Ended'
                  : 'Silence'}
          </div>
        </div>
      </div>

      {/* Latency metrics */}
      <h4 style={{ ...styles.title, marginTop: theme.spacing.md }}>Latency</h4>
      <div style={styles.grid}>
        <div style={styles.metric}>
          <div style={styles.metricLabel}>ASR</div>
          <div
            style={{
              ...styles.metricValue,
              color: getLatencyColor(metrics.asrLatencyMs),
            }}
          >
            {formatLatency(metrics.asrLatencyMs)}
          </div>
        </div>

        <div style={styles.metric}>
          <div style={styles.metricLabel}>LLM</div>
          <div
            style={{
              ...styles.metricValue,
              color: getLatencyColor(metrics.llmLatencyMs),
            }}
          >
            {formatLatency(metrics.llmLatencyMs)}
          </div>
        </div>

        <div style={styles.metric}>
          <div style={styles.metricLabel}>TTS</div>
          <div
            style={{
              ...styles.metricValue,
              color: getLatencyColor(metrics.ttsLatencyMs),
            }}
          >
            {formatLatency(metrics.ttsLatencyMs)}
          </div>
        </div>
      </div>

      {/* Audio levels */}
      <h4 style={{ ...styles.title, marginTop: theme.spacing.md }}>Audio</h4>
      <div style={styles.audioLevels}>
        <div style={styles.levelRow}>
          <span style={styles.levelLabel}>Mic</span>
          <div style={styles.levelBar}>
            <div
              style={{
                ...styles.levelFill,
                width: `${metrics.audioInputLevel * 100}%`,
                backgroundColor: theme.colors.kotakRed,
              }}
            />
          </div>
        </div>
        <div style={styles.levelRow}>
          <span style={styles.levelLabel}>Out</span>
          <div style={styles.levelBar}>
            <div
              style={{
                ...styles.levelFill,
                width: `${metrics.audioOutputLevel * 100}%`,
                backgroundColor: theme.colors.success,
              }}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    background: theme.colors.bgCard,
    borderRadius: theme.borderRadius.lg,
    padding: theme.spacing.md,
  },
  title: {
    fontSize: theme.fontSize.xs,
    fontWeight: theme.fontWeight.semibold,
    color: theme.colors.textMuted,
    textTransform: 'uppercase',
    letterSpacing: '1px',
    marginBottom: theme.spacing.sm,
  },
  statusGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(2, 1fr)',
    gap: theme.spacing.sm,
  },
  statusItem: {
    padding: theme.spacing.sm,
    background: theme.colors.bgTertiary,
    borderRadius: theme.borderRadius.md,
  },
  statusLabel: {
    fontSize: theme.fontSize.xs,
    color: theme.colors.textMuted,
    marginBottom: '2px',
  },
  statusValue: {
    fontSize: theme.fontSize.sm,
    fontWeight: theme.fontWeight.semibold,
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(3, 1fr)',
    gap: theme.spacing.sm,
  },
  metric: {
    textAlign: 'center',
  },
  metricLabel: {
    fontSize: theme.fontSize.xs,
    color: theme.colors.textMuted,
    marginBottom: theme.spacing.xs,
  },
  metricValue: {
    fontSize: theme.fontSize.sm,
    fontWeight: theme.fontWeight.semibold,
    fontFamily: 'monospace',
  },
  audioLevels: {
    display: 'flex',
    flexDirection: 'column',
    gap: theme.spacing.xs,
  },
  levelRow: {
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing.sm,
  },
  levelLabel: {
    fontSize: theme.fontSize.xs,
    color: theme.colors.textMuted,
    width: '30px',
  },
  levelBar: {
    flex: 1,
    height: '6px',
    background: theme.colors.bgTertiary,
    borderRadius: theme.borderRadius.full,
    overflow: 'hidden',
  },
  levelFill: {
    height: '100%',
    borderRadius: theme.borderRadius.full,
    transition: 'width 0.1s ease',
  },
};
