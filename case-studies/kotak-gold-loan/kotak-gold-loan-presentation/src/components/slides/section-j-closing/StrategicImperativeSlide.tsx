import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const imperatives = [
  {
    number: '01',
    title: 'Market Timing',
    reason: 'NBFC trust crisis + regulatory tightening = acquisition window',
    icon: '⏰',
  },
  {
    number: '02',
    title: 'Competitive Moat',
    reason: 'First-mover in Switch & Save + Bridge Loan innovation',
    icon: '🏰',
  },
  {
    number: '03',
    title: 'Risk-Adjusted Returns',
    reason: '2.45x ROI on secured, low-NPA asset class',
    icon: '📈',
  },
  {
    number: '04',
    title: 'Strategic Fit',
    reason: 'Leverages existing 811+ branches, MSME relationships',
    icon: '🎯',
  },
  {
    number: '05',
    title: 'Customer Value',
    reason: 'Rs 40K+ annual savings per customer = genuine value creation',
    icon: '💰',
  },
];

export const StrategicImperativeSlide: React.FC = () => {
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
            THE CASE FOR ACTION
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 32px 0">
            Five Reasons to Act Now
          </Heading>
        </MotionBox>

        {/* Imperatives List */}
        <FlexBox flex={1} flexDirection="column" style={{ gap: '14px' }}>
          {imperatives.map((item, index) => (
            <MotionBox
              key={index}
              initial={{ opacity: 0, x: -40 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.5, delay: 0.2 + index * 0.1 }}
              style={{
                background: kotakColors.darkCard,
                borderRadius: '12px',
                padding: '20px 24px',
                borderLeft: `4px solid ${kotakColors.primary}`,
              }}
            >
              <FlexBox alignItems="center" style={{ gap: '20px' }}>
                {/* Number */}
                <Box
                  style={{
                    width: '48px',
                    height: '48px',
                    borderRadius: '12px',
                    background: `${kotakColors.primary}20`,
                    border: `2px solid ${kotakColors.primary}`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    flexShrink: 0,
                  }}
                >
                  <Text fontSize="18px" fontWeight={700} color={kotakColors.primary} margin="0">
                    {item.number}
                  </Text>
                </Box>

                {/* Icon */}
                <Text fontSize="32px" margin="0">{item.icon}</Text>

                {/* Content */}
                <Box style={{ flex: 1 }}>
                  <Text fontSize="18px" fontWeight={700} color={kotakColors.white} margin="0 0 4px 0">
                    {item.title}
                  </Text>
                  <Text fontSize="15px" color={kotakColors.textMuted} margin="0">
                    {item.reason}
                  </Text>
                </Box>

                {/* Checkmark */}
                <Text fontSize="24px" color={kotakColors.success} margin="0">✓</Text>
              </FlexBox>
            </MotionBox>
          ))}
        </FlexBox>

        {/* Bottom Call to Action */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.8 }}
          style={{ marginTop: '20px' }}
        >
          <Box
            style={{
              background: `linear-gradient(90deg, ${kotakColors.primary}, ${kotakColors.primaryDark})`,
              borderRadius: '12px',
              padding: '20px 32px',
              textAlign: 'center',
            }}
          >
            <Text fontSize="20px" fontWeight={700} color={kotakColors.white} margin="0">
              The question isn't "should we do this?" — it's "can we afford not to?"
            </Text>
          </Box>
        </MotionBox>
      </FlexBox>

      <Notes>
        Five strategic imperatives: Market timing with NBFC crisis, competitive moat through innovation,
        strong risk-adjusted returns, strategic fit with existing infrastructure, and genuine customer value.
        The window is open now - we must act.
      </Notes>
    </Slide>
  );
};

export default StrategicImperativeSlide;
