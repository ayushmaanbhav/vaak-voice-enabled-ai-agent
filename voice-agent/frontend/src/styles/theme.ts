// Kotak brand colors and theme
export const theme = {
  colors: {
    // Brand colors
    kotakRed: '#ED1C24',
    kotakRedDark: '#C41920',
    kotakRedLight: '#FF3B42',

    // Background colors (dark theme)
    bgPrimary: '#0f0f1a',
    bgSecondary: '#1a1a2e',
    bgTertiary: '#252540',
    bgCard: 'rgba(255, 255, 255, 0.05)',
    bgCardHover: 'rgba(255, 255, 255, 0.08)',
    bgSelected: 'rgba(237, 28, 36, 0.15)',

    // Text colors
    textPrimary: '#ffffff',
    textSecondary: 'rgba(255, 255, 255, 0.7)',
    textMuted: 'rgba(255, 255, 255, 0.5)',

    // Status colors
    success: '#4ade80',
    warning: '#fbbf24',
    error: '#ef4444',
    info: '#3b82f6',

    // Border colors
    border: 'rgba(255, 255, 255, 0.1)',
    borderHover: 'rgba(255, 255, 255, 0.2)',
    borderSelected: '#ED1C24',
  },

  spacing: {
    xs: '0.25rem',
    sm: '0.5rem',
    md: '1rem',
    lg: '1.5rem',
    xl: '2rem',
    xxl: '3rem',
  },

  borderRadius: {
    sm: '4px',
    md: '8px',
    lg: '12px',
    xl: '16px',
    full: '9999px',
  },

  fontSize: {
    xs: '0.75rem',
    sm: '0.875rem',
    md: '1rem',
    lg: '1.125rem',
    xl: '1.25rem',
    xxl: '1.5rem',
    xxxl: '2rem',
  },

  fontWeight: {
    normal: 400,
    medium: 500,
    semibold: 600,
    bold: 700,
  },

  shadows: {
    sm: '0 1px 2px rgba(0, 0, 0, 0.2)',
    md: '0 4px 6px rgba(0, 0, 0, 0.3)',
    lg: '0 10px 15px rgba(0, 0, 0, 0.4)',
    glow: '0 0 20px rgba(237, 28, 36, 0.3)',
  },

  transitions: {
    fast: '0.15s ease',
    normal: '0.2s ease',
    slow: '0.3s ease',
  },
};

export type Theme = typeof theme;
