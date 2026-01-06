import React from 'react';
import { Box, Text, FlexBox } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../theme/kotakTheme';

const MotionBox = motion(Box);

interface MetricCardProps {
  icon?: string;
  value: string;
  label: string;
  sublabel?: string;
  color?: 'primary' | 'success' | 'gold' | 'danger' | 'secondary';
  size?: 'small' | 'medium' | 'large';
  delay?: number;
}

const colorMap = {
  primary: kotakColors.primary,
  success: kotakColors.success,
  gold: kotakColors.gold,
  danger: kotakColors.danger,
  secondary: kotakColors.secondary,
};

const sizeMap = {
  small: { metric: '32px', label: '14px', padding: '20px', icon: '28px' },
  medium: { metric: '42px', label: '16px', padding: '24px', icon: '36px' },
  large: { metric: '56px', label: '18px', padding: '32px', icon: '48px' },
};

export const MetricCard: React.FC<MetricCardProps> = ({
  icon,
  value,
  label,
  sublabel,
  color = 'primary',
  size = 'medium',
  delay = 0,
}) => {
  const accentColor = colorMap[color];
  const sizes = sizeMap[size];

  return (
    <MotionBox
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, delay }}
      style={{
        background: kotakColors.darkCard,
        borderRadius: '16px',
        padding: sizes.padding,
        border: `1px solid rgba(255, 255, 255, 0.1)`,
        minWidth: size === 'small' ? '140px' : '180px',
      }}
    >
      <FlexBox flexDirection="column" alignItems="flex-start" style={{ gap: '12px' }}>
        {icon && (
          <Text fontSize={sizes.icon} margin="0">
            {icon}
          </Text>
        )}
        <Box>
          <Text
            fontSize={sizes.metric}
            fontWeight={700}
            color={accentColor}
            margin="0"
            style={{ lineHeight: 1.1 }}
          >
            {value}
          </Text>
          <Text
            fontSize={sizes.label}
            fontWeight={500}
            color={kotakColors.white}
            margin="8px 0 0 0"
          >
            {label}
          </Text>
          {sublabel && (
            <Text
              fontSize="14px"
              color={kotakColors.textMuted}
              margin="4px 0 0 0"
            >
              {sublabel}
            </Text>
          )}
        </Box>
      </FlexBox>
    </MotionBox>
  );
};

// Horizontal variant for inline metrics
export const MetricCardHorizontal: React.FC<MetricCardProps> = ({
  icon,
  value,
  label,
  sublabel,
  color = 'primary',
  delay = 0,
}) => {
  const accentColor = colorMap[color];

  return (
    <MotionBox
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.5, delay }}
      style={{
        background: kotakColors.darkCard,
        borderRadius: '12px',
        padding: '20px 24px',
        border: `1px solid rgba(255, 255, 255, 0.1)`,
        borderLeft: `4px solid ${accentColor}`,
      }}
    >
      <FlexBox alignItems="center" style={{ gap: '16px' }}>
        {icon && (
          <Text fontSize="32px" margin="0">
            {icon}
          </Text>
        )}
        <Box>
          <Text
            fontSize="28px"
            fontWeight={700}
            color={accentColor}
            margin="0"
            style={{ lineHeight: 1.1 }}
          >
            {value}
          </Text>
          <Text
            fontSize="16px"
            fontWeight={500}
            color={kotakColors.white}
            margin="4px 0 0 0"
          >
            {label}
          </Text>
          {sublabel && (
            <Text
              fontSize="14px"
              color={kotakColors.textMuted}
              margin="2px 0 0 0"
            >
              {sublabel}
            </Text>
          )}
        </Box>
      </FlexBox>
    </MotionBox>
  );
};

export default MetricCard;
