import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const highlights = [
  {
    value: 'Rs 363 Cr',
    label: '3-Year PAT',
    sublabel: 'Cumulative profit after tax',
    icon: '💰',
    color: kotakColors.success,
  },
  {
    value: '2.45x',
    label: 'ROI',
    sublabel: 'Return on Rs 148 Cr investment',
    icon: '📈',
    color: kotakColors.gold,
  },
  {
    value: '425K',
    label: 'Customers',
    sublabel: 'Acquired over 3 years',
    icon: '👥',
    color: kotakColors.primary,
  },
];

export const AnswerPreviewSlide: React.FC = () => {
  return (
    <Slide backgroundColor={kotakColors.dark}>
      <FlexBox flexDirection="column" height="100%" padding="40px 60px">
        {/* Section Label */}
        <MotionBox
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.4 }}
        >
          <Text
            fontSize="14px"
            fontWeight={600}
            color={kotakColors.primary}
            margin="0 0 8px 0"
            style={{ letterSpacing: '2px', textTransform: 'uppercase' }}
          >
            THE ANSWER
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 48px 0">
            A Measured Aggressive Strategy with Strong Returns
          </Heading>
        </MotionBox>

        {/* Highlight Cards */}
        <FlexBox flex={1} alignItems="center" justifyContent="center" style={{ gap: '32px' }}>
          {highlights.map((item, index) => (
            <MotionBox
              key={index}
              initial={{ opacity: 0, y: 30, scale: 0.9 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              transition={{ duration: 0.5, delay: 0.3 + index * 0.15 }}
              style={{
                flex: 1,
                maxWidth: '300px',
                background: `linear-gradient(135deg, ${item.color}15, ${item.color}05)`,
                borderRadius: '20px',
                padding: '40px 32px',
                border: `2px solid ${item.color}40`,
                textAlign: 'center',
              }}
            >
              <Text fontSize="48px" margin="0 0 16px 0">{item.icon}</Text>
              <Text fontSize="48px" fontWeight={700} color={item.color} margin="0">
                {item.value}
              </Text>
              <Text fontSize="20px" fontWeight={600} color={kotakColors.white} margin="12px 0 4px 0">
                {item.label}
              </Text>
              <Text fontSize="15px" color={kotakColors.textMuted} margin="0">
                {item.sublabel}
              </Text>
            </MotionBox>
          ))}
        </FlexBox>

        {/* Investment Context */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.9 }}
          style={{ marginTop: '32px' }}
        >
          <FlexBox
            style={{
              background: kotakColors.darkCard,
              borderRadius: '12px',
              padding: '20px 32px',
            }}
            justifyContent="space-around"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">
                Investment Required
              </Text>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.white} margin="0">
                Rs 148 Cr <Text fontSize="16px" color={kotakColors.textMuted} style={{ display: 'inline' }}>(3 years)</Text>
              </Text>
            </Box>
            <Box style={{ width: '1px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">
                Payback Period
              </Text>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.success} margin="0">
                ~20 months
              </Text>
            </Box>
            <Box style={{ width: '1px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">
                Risk Profile
              </Text>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.gold} margin="0">
                LOW
              </Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Here's the summary: Rs 363 crore PAT over 3 years on an investment of Rs 148 crore.
        That's a 2.45x return with payback in about 20 months. Gold loans are inherently
        low-risk with near-zero NPAs when managed well. Now let's look at the market
        opportunity that makes this possible.
      </Notes>
    </Slide>
  );
};

export default AnswerPreviewSlide;
