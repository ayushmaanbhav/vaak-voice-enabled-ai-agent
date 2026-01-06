import React from 'react';
import { Box, Text, FlexBox } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../theme/kotakTheme';

const MotionBox = motion(Box);

interface ProfileAttribute {
  label: string;
  value: string;
}

interface ProfileCardProps {
  id?: string;
  name: string;
  subtitle?: string;
  avatar?: string;
  color?: string;
  attributes: ProfileAttribute[];
  highlight?: string;
  delay?: number;
}

export const ProfileCard: React.FC<ProfileCardProps> = ({
  id,
  name,
  subtitle,
  avatar,
  color = kotakColors.primary,
  attributes,
  highlight,
  delay = 0,
}) => {
  return (
    <MotionBox
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, delay }}
      style={{
        background: kotakColors.darkCard,
        borderRadius: '16px',
        padding: '28px',
        border: `1px solid rgba(255, 255, 255, 0.1)`,
        borderTop: `4px solid ${color}`,
      }}
    >
      {/* Header */}
      <FlexBox alignItems="center" style={{ gap: '16px', marginBottom: '24px' }}>
        {avatar && (
          <Box
            style={{
              width: '56px',
              height: '56px',
              borderRadius: '50%',
              background: `${color}20`,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              border: `2px solid ${color}`,
            }}
          >
            <Text fontSize="28px" margin="0">{avatar}</Text>
          </Box>
        )}
        <Box>
          <FlexBox alignItems="center" style={{ gap: '8px' }}>
            {id && (
              <Box
                style={{
                  background: color,
                  borderRadius: '4px',
                  padding: '2px 8px',
                }}
              >
                <Text fontSize="12px" fontWeight={700} color={kotakColors.white} margin="0">
                  {id}
                </Text>
              </Box>
            )}
            <Text fontSize="22px" fontWeight={700} color={kotakColors.white} margin="0">
              {name}
            </Text>
          </FlexBox>
          {subtitle && (
            <Text fontSize="16px" color={color} margin="4px 0 0 0">
              {subtitle}
            </Text>
          )}
        </Box>
      </FlexBox>

      {/* Attributes Grid */}
      <Box
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(2, 1fr)',
          gap: '16px',
        }}
      >
        {attributes.map((attr, index) => (
          <MotionBox
            key={index}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.3, delay: delay + index * 0.05 }}
            style={{
              background: 'rgba(255, 255, 255, 0.03)',
              borderRadius: '8px',
              padding: '12px 16px',
            }}
          >
            <Text fontSize="13px" color={kotakColors.textMuted} margin="0 0 4px 0">
              {attr.label}
            </Text>
            <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0">
              {attr.value}
            </Text>
          </MotionBox>
        ))}
      </Box>

      {/* Highlight Box */}
      {highlight && (
        <Box
          style={{
            marginTop: '20px',
            background: `${color}15`,
            borderRadius: '8px',
            padding: '12px 16px',
            borderLeft: `3px solid ${color}`,
          }}
        >
          <Text fontSize="15px" color={kotakColors.textPrimary} margin="0" style={{ lineHeight: 1.5 }}>
            {highlight}
          </Text>
        </Box>
      )}
    </MotionBox>
  );
};

// Segment card variant
interface SegmentCardProps {
  id: string;
  name: string;
  share: number;
  color: string;
  conversion: string;
  description: string;
  delay?: number;
}

export const SegmentCard: React.FC<SegmentCardProps> = ({
  id,
  name,
  share,
  color,
  conversion,
  description,
  delay = 0,
}) => {
  return (
    <MotionBox
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.4, delay }}
      style={{
        background: kotakColors.darkCard,
        borderRadius: '16px',
        padding: '24px',
        border: `1px solid rgba(255, 255, 255, 0.1)`,
        borderLeft: `4px solid ${color}`,
      }}
    >
      <FlexBox justifyContent="space-between" alignItems="flex-start" style={{ marginBottom: '16px' }}>
        <Box>
          <Box
            style={{
              background: color,
              borderRadius: '4px',
              padding: '2px 8px',
              display: 'inline-block',
              marginBottom: '8px',
            }}
          >
            <Text fontSize="12px" fontWeight={700} color={kotakColors.white} margin="0">
              {id}
            </Text>
          </Box>
          <Text fontSize="18px" fontWeight={700} color={kotakColors.white} margin="0">
            {name}
          </Text>
        </Box>
        <Box style={{ textAlign: 'right' }}>
          <Text fontSize="28px" fontWeight={700} color={color} margin="0">
            {share}%
          </Text>
          <Text fontSize="12px" color={kotakColors.textMuted} margin="0">
            of target
          </Text>
        </Box>
      </FlexBox>

      <Text fontSize="15px" color={kotakColors.textMuted} margin="0 0 16px 0" style={{ lineHeight: 1.5 }}>
        {description}
      </Text>

      <Box
        style={{
          background: `${color}15`,
          borderRadius: '6px',
          padding: '8px 12px',
        }}
      >
        <Text fontSize="14px" color={color} margin="0">
          Conversion: <strong>{conversion}</strong>
        </Text>
      </Box>
    </MotionBox>
  );
};

export default ProfileCard;
