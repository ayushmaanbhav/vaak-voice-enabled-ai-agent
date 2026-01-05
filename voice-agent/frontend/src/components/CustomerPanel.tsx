import React from 'react';
import { theme } from '../styles/theme';
import type { Customer, Language } from '../types';

interface CustomerPanelProps {
  customer: Customer | null;
  language: Language | null;
  stage: string;
  turnCount: number;
  sessionDuration: number;
}

const formatCurrency = (amount: number): string => {
  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
    maximumFractionDigits: 0,
  }).format(amount);
};

const formatDuration = (ms: number): string => {
  const seconds = Math.floor(ms / 1000);
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
};

const calculateSavings = (outstanding: number, currentRate: number): number => {
  const kotakRate = 10;
  return (outstanding * (currentRate - kotakRate)) / 100;
};

const segmentLabels: Record<string, { label: string; color: string }> = {
  high_value: { label: 'High Value', color: '#FFD700' },
  trust_seeker: { label: 'Trust Seeker', color: '#4A90D9' },
  shakti: { label: 'Shakti', color: '#E91E63' },
  young_pro: { label: 'Young Pro', color: '#00BCD4' },
};

const providerColors: Record<string, string> = {
  muthoot: '#FF9800',
  manappuram: '#9C27B0',
  iifl: '#4CAF50',
  other: '#607D8B',
};

export const CustomerPanel: React.FC<CustomerPanelProps> = ({
  customer,
  language,
  stage,
  turnCount,
  sessionDuration,
}) => {
  if (!customer) {
    return (
      <aside style={styles.panel}>
        <div style={styles.empty}>No customer selected</div>
      </aside>
    );
  }

  const segment = segmentLabels[customer.segment] || { label: customer.segment, color: '#888' };
  const savings = calculateSavings(customer.estimated_outstanding, customer.estimated_rate);

  return (
    <aside style={styles.panel}>
      <div style={styles.section}>
        <h3 style={styles.sectionTitle}>Customer</h3>
        <div style={styles.customerName}>{customer.name}</div>
        <div style={styles.badges}>
          <span style={{ ...styles.badge, backgroundColor: segment.color }}>{segment.label}</span>
          <span
            style={{
              ...styles.badge,
              backgroundColor: providerColors[customer.current_provider] || '#888',
            }}
          >
            {customer.current_provider.toUpperCase()}
          </span>
        </div>
        <div style={styles.infoRow}>
          <span style={styles.infoLabel}>City</span>
          <span style={styles.infoValue}>{customer.city}</span>
        </div>
        {language && (
          <div style={styles.infoRow}>
            <span style={styles.infoLabel}>Language</span>
            <span style={styles.infoValue}>
              {language.native} ({language.name})
            </span>
          </div>
        )}
      </div>

      <div style={styles.section}>
        <h3 style={styles.sectionTitle}>Loan Details</h3>
        <div style={styles.infoRow}>
          <span style={styles.infoLabel}>Outstanding</span>
          <span style={styles.infoValue}>{formatCurrency(customer.estimated_outstanding)}</span>
        </div>
        <div style={styles.infoRow}>
          <span style={styles.infoLabel}>Current Rate</span>
          <span style={{ ...styles.infoValue, color: theme.colors.error }}>
            {customer.estimated_rate}%
          </span>
        </div>
        <div style={styles.infoRow}>
          <span style={styles.infoLabel}>Kotak Rate</span>
          <span style={{ ...styles.infoValue, color: theme.colors.success }}>10%</span>
        </div>
        <div style={styles.savingsBox}>
          <div style={styles.savingsLabel}>Potential Savings</div>
          <div style={styles.savingsValue}>{formatCurrency(savings)}/year</div>
        </div>
      </div>

      <div style={styles.section}>
        <h3 style={styles.sectionTitle}>Session</h3>
        <div style={styles.infoRow}>
          <span style={styles.infoLabel}>Stage</span>
          <span style={styles.stageValue}>{stage || 'Waiting'}</span>
        </div>
        <div style={styles.infoRow}>
          <span style={styles.infoLabel}>Turns</span>
          <span style={styles.infoValue}>{turnCount}</span>
        </div>
        <div style={styles.infoRow}>
          <span style={styles.infoLabel}>Duration</span>
          <span style={styles.infoValue}>{formatDuration(sessionDuration)}</span>
        </div>
      </div>
    </aside>
  );
};

const styles: Record<string, React.CSSProperties> = {
  panel: {
    width: '280px',
    background: theme.colors.bgSecondary,
    borderRight: `1px solid ${theme.colors.border}`,
    padding: theme.spacing.lg,
    display: 'flex',
    flexDirection: 'column',
    gap: theme.spacing.lg,
    overflowY: 'auto',
  },
  empty: {
    color: theme.colors.textMuted,
    textAlign: 'center',
    padding: theme.spacing.xl,
  },
  section: {
    display: 'flex',
    flexDirection: 'column',
    gap: theme.spacing.sm,
  },
  sectionTitle: {
    fontSize: theme.fontSize.xs,
    fontWeight: theme.fontWeight.semibold,
    color: theme.colors.textMuted,
    textTransform: 'uppercase',
    letterSpacing: '1px',
    marginBottom: theme.spacing.xs,
  },
  customerName: {
    fontSize: theme.fontSize.lg,
    fontWeight: theme.fontWeight.bold,
    color: theme.colors.textPrimary,
  },
  badges: {
    display: 'flex',
    gap: theme.spacing.xs,
    flexWrap: 'wrap',
  },
  badge: {
    fontSize: theme.fontSize.xs,
    padding: `${theme.spacing.xs} ${theme.spacing.sm}`,
    borderRadius: theme.borderRadius.sm,
    color: '#fff',
    fontWeight: theme.fontWeight.medium,
  },
  infoRow: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  infoLabel: {
    fontSize: theme.fontSize.sm,
    color: theme.colors.textMuted,
  },
  infoValue: {
    fontSize: theme.fontSize.sm,
    color: theme.colors.textPrimary,
    fontWeight: theme.fontWeight.medium,
  },
  stageValue: {
    fontSize: theme.fontSize.sm,
    color: theme.colors.kotakRed,
    fontWeight: theme.fontWeight.semibold,
  },
  savingsBox: {
    marginTop: theme.spacing.sm,
    padding: theme.spacing.md,
    background: 'rgba(74, 222, 128, 0.1)',
    borderRadius: theme.borderRadius.md,
    border: `1px solid ${theme.colors.success}`,
  },
  savingsLabel: {
    fontSize: theme.fontSize.xs,
    color: theme.colors.success,
    marginBottom: theme.spacing.xs,
  },
  savingsValue: {
    fontSize: theme.fontSize.xl,
    fontWeight: theme.fontWeight.bold,
    color: theme.colors.success,
  },
};
