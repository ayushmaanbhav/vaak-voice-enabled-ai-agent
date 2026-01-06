import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const askItems = [
  {
    ask: 'Investment Approval',
    detail: 'Rs 53 Cr Year 1 budget (of Rs 148 Cr total)',
    timeline: 'Immediate',
    priority: 'Critical',
    icon: '💰',
  },
  {
    ask: 'Product Launch',
    detail: 'Switch & Save with Bridge Loan feature',
    timeline: 'Q1 2026',
    priority: 'Critical',
    icon: '🚀',
  },
  {
    ask: 'AI Development',
    detail: 'Begin Computer Vision + Predictive Model work',
    timeline: 'Q1 2026',
    priority: 'High',
    icon: '🤖',
  },
  {
    ask: 'Doorstep Pilot',
    detail: 'Mumbai pilot with 20 field executives',
    timeline: 'Q2 2026',
    priority: 'Medium',
    icon: '🏠',
  },
  {
    ask: 'Team Authorization',
    detail: '15-member dedicated gold loan acquisition team',
    timeline: 'Q1 2026',
    priority: 'High',
    icon: '👥',
  },
];

export const TheAskSlide: React.FC = () => {
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
            color={kotakColors.gold}
            margin="0 0 8px 0"
            style={{ letterSpacing: '2px', textTransform: 'uppercase' }}
          >
            THE ASK
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            What We're Requesting
          </Heading>
        </MotionBox>

        {/* Ask Items */}
        <FlexBox flex={1} flexDirection="column" style={{ gap: '12px' }}>
          {askItems.map((item, index) => (
            <MotionBox
              key={index}
              initial={{ opacity: 0, x: -30 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.4, delay: 0.2 + index * 0.1 }}
              style={{
                background: kotakColors.darkCard,
                borderRadius: '12px',
                padding: '18px 24px',
                borderLeft: `4px solid ${item.priority === 'Critical' ? kotakColors.primary : item.priority === 'High' ? kotakColors.gold : kotakColors.success}`,
              }}
            >
              <FlexBox alignItems="center" style={{ gap: '20px' }}>
                {/* Icon */}
                <Box
                  style={{
                    width: '50px',
                    height: '50px',
                    borderRadius: '12px',
                    background: `${kotakColors.primary}15`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    flexShrink: 0,
                  }}
                >
                  <Text fontSize="26px" margin="0">{item.icon}</Text>
                </Box>

                {/* Content */}
                <Box style={{ flex: 1 }}>
                  <Text fontSize="17px" fontWeight={700} color={kotakColors.white} margin="0 0 4px 0">
                    {item.ask}
                  </Text>
                  <Text fontSize="14px" color={kotakColors.textMuted} margin="0">
                    {item.detail}
                  </Text>
                </Box>

                {/* Timeline */}
                <Box style={{ textAlign: 'center', minWidth: '80px' }}>
                  <Text fontSize="14px" fontWeight={600} color={kotakColors.gold} margin="0">
                    {item.timeline}
                  </Text>
                  <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">
                    Timeline
                  </Text>
                </Box>

                {/* Priority Badge */}
                <Box
                  style={{
                    padding: '6px 14px',
                    background: item.priority === 'Critical' ? `${kotakColors.primary}30` : item.priority === 'High' ? `${kotakColors.gold}30` : `${kotakColors.success}30`,
                    borderRadius: '20px',
                    minWidth: '80px',
                    textAlign: 'center',
                  }}
                >
                  <Text
                    fontSize="12px"
                    fontWeight={700}
                    color={item.priority === 'Critical' ? kotakColors.primary : item.priority === 'High' ? kotakColors.gold : kotakColors.success}
                    margin="0"
                  >
                    {item.priority}
                  </Text>
                </Box>
              </FlexBox>
            </MotionBox>
          ))}
        </FlexBox>

        {/* Summary Box */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.8 }}
          style={{ marginTop: '20px' }}
        >
          <FlexBox
            style={{
              background: `linear-gradient(90deg, ${kotakColors.gold}20, ${kotakColors.gold}05)`,
              borderRadius: '12px',
              padding: '20px 32px',
              border: `1px solid ${kotakColors.gold}30`,
            }}
            justifyContent="space-between"
            alignItems="center"
          >
            <Box>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">
                Year 1 Investment Ask
              </Text>
              <Text fontSize="32px" fontWeight={700} color={kotakColors.gold} margin="0">
                Rs 53 Cr
              </Text>
            </Box>
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">
                Expected Year 1 PAT
              </Text>
              <Text fontSize="32px" fontWeight={700} color={kotakColors.success} margin="0">
                Rs 43 Cr
              </Text>
            </Box>
            <Box style={{ textAlign: 'right' }}>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">
                Year 1 ROI
              </Text>
              <Text fontSize="32px" fontWeight={700} color={kotakColors.primary} margin="0">
                81%
              </Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Five asks: Rs 53 Cr Year 1 budget, product launch Q1, AI development, doorstep pilot Q2,
        and 15-member dedicated team. Year 1 investment of Rs 53 Cr expected to deliver
        Rs 43 Cr PAT = 81% ROI in first year itself.
      </Notes>
    </Slide>
  );
};

export default TheAskSlide;
