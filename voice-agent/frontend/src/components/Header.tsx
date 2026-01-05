import React from 'react';
import { theme } from '../styles/theme';
import type { ConnectionStatus } from '../types';

interface HeaderProps {
  connectionStatus: ConnectionStatus;
  sessionId: string | null;
}

export const Header: React.FC<HeaderProps> = ({ connectionStatus, sessionId }) => {
  const statusColors: Record<ConnectionStatus, string> = {
    disconnected: theme.colors.textMuted,
    connecting: theme.colors.warning,
    connected: theme.colors.success,
    error: theme.colors.error,
  };

  return (
    <header style={styles.header}>
      <div style={styles.logoContainer}>
        <div style={styles.logo}>KOTAK</div>
        <div style={styles.tagline}>Gold Loan Voice Agent</div>
      </div>

      <div style={styles.statusContainer}>
        <div
          style={{
            ...styles.statusDot,
            backgroundColor: statusColors[connectionStatus],
          }}
        />
        <span style={styles.statusText}>
          {connectionStatus === 'connected' && sessionId
            ? `Connected (${sessionId.slice(0, 8)}...)`
            : connectionStatus.charAt(0).toUpperCase() + connectionStatus.slice(1)}
        </span>
      </div>
    </header>
  );
};

const styles: Record<string, React.CSSProperties> = {
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: `${theme.spacing.md} ${theme.spacing.xl}`,
    background: theme.colors.kotakRed,
    color: theme.colors.textPrimary,
    boxShadow: theme.shadows.md,
  },
  logoContainer: {
    display: 'flex',
    flexDirection: 'column',
  },
  logo: {
    fontSize: theme.fontSize.xxl,
    fontWeight: theme.fontWeight.bold,
    letterSpacing: '3px',
  },
  tagline: {
    fontSize: theme.fontSize.sm,
    opacity: 0.9,
    marginTop: theme.spacing.xs,
  },
  statusContainer: {
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing.sm,
    padding: `${theme.spacing.sm} ${theme.spacing.md}`,
    background: 'rgba(0, 0, 0, 0.2)',
    borderRadius: theme.borderRadius.full,
  },
  statusDot: {
    width: '10px',
    height: '10px',
    borderRadius: '50%',
    animation: 'pulse 2s infinite',
  },
  statusText: {
    fontSize: theme.fontSize.sm,
    fontWeight: theme.fontWeight.medium,
  },
};
