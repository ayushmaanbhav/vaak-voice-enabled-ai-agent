import React from 'react';
import { Box, Text, FlexBox } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../theme/kotakTheme';

const MotionBox = motion(Box);

interface QuoteCardProps {
  quote: string;
  source?: string;
  rating?: number;
  maxRating?: number;
  painPoint?: string;
  delay?: number;
  variant?: 'default' | 'compact' | 'highlight';
}

export const QuoteCard: React.FC<QuoteCardProps> = ({
  quote,
  source,
  rating,
  maxRating = 5,
  painPoint,
  delay = 0,
  variant = 'default',
}) => {
  const renderStars = () => {
    if (rating === undefined) return null;
    const stars = [];
    for (let i = 0; i < maxRating; i++) {
      stars.push(
        <Text
          key={i}
          fontSize="16px"
          margin="0"
          style={{ opacity: i < rating ? 1 : 0.3 }}
        >
          ★
        </Text>
      );
    }
    return (
      <FlexBox style={{ gap: '2px' }}>
        {stars}
        <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 0 8px">
          {rating}/{maxRating}
        </Text>
      </FlexBox>
    );
  };

  if (variant === 'compact') {
    return (
      <MotionBox
        initial={{ opacity: 0, x: -20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.4, delay }}
        style={{
          background: kotakColors.darkCard,
          borderRadius: '12px',
          padding: '16px 20px',
          borderLeft: `3px solid ${kotakColors.danger}`,
        }}
      >
        <Text
          fontSize="16px"
          color={kotakColors.textPrimary}
          margin="0"
          style={{ fontStyle: 'italic', lineHeight: 1.5 }}
        >
          "{quote}"
        </Text>
        {source && (
          <Text fontSize="14px" color={kotakColors.textMuted} margin="8px 0 0 0">
            — {source}
          </Text>
        )}
      </MotionBox>
    );
  }

  if (variant === 'highlight') {
    return (
      <MotionBox
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.5, delay }}
        style={{
          background: `linear-gradient(135deg, ${kotakColors.gold}15, ${kotakColors.gold}05)`,
          borderRadius: '16px',
          padding: '28px 32px',
          border: `2px solid ${kotakColors.gold}40`,
        }}
      >
        <Text fontSize="28px" color={kotakColors.gold} margin="0 0 16px 0">
          💬
        </Text>
        <Text
          fontSize="22px"
          color={kotakColors.white}
          margin="0"
          style={{ fontStyle: 'italic', lineHeight: 1.5 }}
        >
          "{quote}"
        </Text>
        {source && (
          <Text fontSize="16px" color={kotakColors.textMuted} margin="16px 0 0 0">
            — {source}
          </Text>
        )}
      </MotionBox>
    );
  }

  return (
    <MotionBox
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, delay }}
      style={{
        background: kotakColors.darkCard,
        borderRadius: '16px',
        padding: '24px',
        border: `1px solid rgba(255, 255, 255, 0.1)`,
      }}
    >
      <FlexBox justifyContent="space-between" alignItems="flex-start" style={{ marginBottom: '12px' }}>
        {painPoint && (
          <Box
            style={{
              background: `${kotakColors.danger}20`,
              borderRadius: '6px',
              padding: '4px 10px',
            }}
          >
            <Text fontSize="12px" fontWeight={600} color={kotakColors.danger} margin="0">
              {painPoint}
            </Text>
          </Box>
        )}
        {renderStars()}
      </FlexBox>

      <Text
        fontSize="18px"
        color={kotakColors.textPrimary}
        margin="0"
        style={{ fontStyle: 'italic', lineHeight: 1.6 }}
      >
        "{quote}"
      </Text>

      {source && (
        <Text fontSize="14px" color={kotakColors.textMuted} margin="12px 0 0 0">
          — {source}
        </Text>
      )}
    </MotionBox>
  );
};

export default QuoteCard;
