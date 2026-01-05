import React from 'react';
import { theme } from '../styles/theme';

interface ControlBarProps {
  isMuted: boolean;
  onToggleMute: () => void;
  onDisconnect: () => void;
}

export const ControlBar: React.FC<ControlBarProps> = ({
  isMuted,
  onToggleMute,
  onDisconnect,
}) => {
  return (
    <div style={styles.container}>
      <button
        style={{
          ...styles.button,
          ...(isMuted ? styles.muteButtonActive : styles.muteButton),
        }}
        onClick={onToggleMute}
        title={isMuted ? 'Unmute' : 'Mute'}
      >
        {isMuted ? '🔇' : '🎙️'}
        <span style={styles.buttonLabel}>{isMuted ? 'Unmute' : 'Mute'}</span>
      </button>

      <button
        style={{ ...styles.button, ...styles.disconnectButton }}
        onClick={onDisconnect}
        title="End conversation"
      >
        📞
        <span style={styles.buttonLabel}>End Call</span>
      </button>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    justifyContent: 'center',
    gap: theme.spacing.md,
    padding: theme.spacing.md,
    background: theme.colors.bgSecondary,
    borderTop: `1px solid ${theme.colors.border}`,
  },
  button: {
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing.sm,
    padding: `${theme.spacing.sm} ${theme.spacing.lg}`,
    fontSize: theme.fontSize.md,
    fontWeight: theme.fontWeight.medium,
    border: 'none',
    borderRadius: theme.borderRadius.full,
    cursor: 'pointer',
    transition: `all ${theme.transitions.fast}`,
  },
  muteButton: {
    background: theme.colors.bgTertiary,
    color: theme.colors.textPrimary,
  },
  muteButtonActive: {
    background: theme.colors.error,
    color: theme.colors.textPrimary,
  },
  disconnectButton: {
    background: theme.colors.error,
    color: theme.colors.textPrimary,
  },
  buttonLabel: {
    fontSize: theme.fontSize.sm,
  },
};
