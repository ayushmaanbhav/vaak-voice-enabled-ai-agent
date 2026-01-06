import { defaultTheme } from 'spectacle';

// Kotak Brand Colors
export const kotakColors = {
  // Primary brand
  primary: '#ED1C24',        // Kotak Red
  primaryDark: '#B91C1C',    // Darker red
  primaryLight: '#FEE2E2',   // Light red tint
  secondary: '#003874',      // Kotak Blue

  // Backgrounds
  dark: '#0F172A',           // Slide background (dark blue-gray)
  darkCard: '#1E293B',       // Card background
  darkCardHover: '#334155',  // Card hover
  light: '#FFFFFF',          // Light theme option

  // Text
  white: '#FFFFFF',
  textPrimary: '#F1F5F9',
  textSecondary: '#CBD5E1',
  textMuted: '#94A3B8',

  // Accents
  gold: '#F59E0B',           // Gold accent
  goldLight: '#FEF3C7',      // Light gold
  success: '#10B981',        // Green for positive
  successLight: '#D1FAE5',   // Light green
  warning: '#FBBF24',        // Yellow for caution
  danger: '#EF4444',         // Red for issues
  dangerLight: '#FEE2E2',    // Light red

  // Chart colors (accessible palette)
  chart: ['#ED1C24', '#003874', '#F59E0B', '#10B981', '#8B5CF6', '#EC4899', '#14B8A6', '#F97316']
};

// Typography scale for presentations
export const typography = {
  slideTitle: {
    fontSize: '48px',
    fontWeight: 700,
    lineHeight: 1.2,
  },
  sectionLabel: {
    fontSize: '14px',
    fontWeight: 600,
    letterSpacing: '2px',
    textTransform: 'uppercase' as const,
  },
  subtitle: {
    fontSize: '28px',
    fontWeight: 500,
    lineHeight: 1.4,
  },
  body: {
    fontSize: '18px',
    fontWeight: 400,
    lineHeight: 1.6,
  },
  bodyLarge: {
    fontSize: '20px',
    fontWeight: 400,
    lineHeight: 1.6,
  },
  label: {
    fontSize: '16px',
    fontWeight: 500,
    lineHeight: 1.4,
  },
  labelSmall: {
    fontSize: '14px',
    fontWeight: 500,
    lineHeight: 1.4,
  },
  metric: {
    fontSize: '56px',
    fontWeight: 700,
    lineHeight: 1.1,
  },
  metricMedium: {
    fontSize: '42px',
    fontWeight: 700,
    lineHeight: 1.1,
  },
  metricSmall: {
    fontSize: '32px',
    fontWeight: 700,
    lineHeight: 1.2,
  },
  metricLabel: {
    fontSize: '16px',
    fontWeight: 400,
    lineHeight: 1.4,
  },
  quote: {
    fontSize: '22px',
    fontWeight: 400,
    fontStyle: 'italic' as const,
    lineHeight: 1.5,
  },
  tableHeader: {
    fontSize: '16px',
    fontWeight: 600,
    lineHeight: 1.4,
  },
  tableCell: {
    fontSize: '16px',
    fontWeight: 400,
    lineHeight: 1.4,
  },
};

// Spectacle theme configuration
export const kotakTheme = {
  ...defaultTheme,
  colors: {
    primary: kotakColors.white,
    secondary: kotakColors.textMuted,
    tertiary: kotakColors.primary,
    quaternary: kotakColors.gold,
  },
  fonts: {
    header: '"Poppins", -apple-system, BlinkMacSystemFont, sans-serif',
    text: '"Poppins", -apple-system, BlinkMacSystemFont, sans-serif',
    monospace: '"JetBrains Mono", monospace',
  },
  fontSizes: {
    h1: '48px',
    h2: '36px',
    h3: '28px',
    text: '18px',
    monospace: '16px',
  },
  space: [0, 4, 8, 16, 24, 32, 48, 64, 96, 128],
};

// Animation configurations
export const animations = {
  fadeIn: {
    initial: { opacity: 0 },
    animate: { opacity: 1 },
    transition: { duration: 0.5 },
  },
  slideUp: {
    initial: { opacity: 0, y: 30 },
    animate: { opacity: 1, y: 0 },
    transition: { duration: 0.5 },
  },
  slideLeft: {
    initial: { opacity: 0, x: -30 },
    animate: { opacity: 1, x: 0 },
    transition: { duration: 0.5 },
  },
  slideRight: {
    initial: { opacity: 0, x: 30 },
    animate: { opacity: 1, x: 0 },
    transition: { duration: 0.5 },
  },
  scaleIn: {
    initial: { opacity: 0, scale: 0.9 },
    animate: { opacity: 1, scale: 1 },
    transition: { duration: 0.5 },
  },
  stagger: (index: number, baseDelay: number = 0.1) => ({
    initial: { opacity: 0, y: 20 },
    animate: { opacity: 1, y: 0 },
    transition: { duration: 0.4, delay: index * baseDelay },
  }),
};

// Slide transition
export const slideTransition = {
  from: { opacity: 0, transform: 'translateX(100%)' },
  enter: { opacity: 1, transform: 'translateX(0)' },
  leave: { opacity: 0, transform: 'translateX(-100%)' },
};

// Card styles
export const cardStyles = {
  primary: {
    background: kotakColors.darkCard,
    borderRadius: '16px',
    padding: '24px',
    border: `1px solid rgba(255, 255, 255, 0.1)`,
  },
  highlight: {
    background: `linear-gradient(135deg, ${kotakColors.primary}15, ${kotakColors.primary}05)`,
    borderRadius: '16px',
    padding: '24px',
    border: `2px solid ${kotakColors.primary}40`,
  },
  success: {
    background: `linear-gradient(135deg, ${kotakColors.success}15, ${kotakColors.success}05)`,
    borderRadius: '16px',
    padding: '24px',
    border: `2px solid ${kotakColors.success}40`,
  },
  gold: {
    background: `linear-gradient(135deg, ${kotakColors.gold}15, ${kotakColors.gold}05)`,
    borderRadius: '16px',
    padding: '24px',
    border: `2px solid ${kotakColors.gold}40`,
  },
  danger: {
    background: `linear-gradient(135deg, ${kotakColors.danger}15, ${kotakColors.danger}05)`,
    borderRadius: '16px',
    padding: '24px',
    border: `2px solid ${kotakColors.danger}40`,
  },
};

export default kotakTheme;
