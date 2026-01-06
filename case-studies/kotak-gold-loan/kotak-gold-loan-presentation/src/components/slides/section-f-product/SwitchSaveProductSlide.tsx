import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const features = [
  { icon: '💰', title: 'Zero Processing Fee', desc: 'No charges for first 6 months', color: kotakColors.success },
  { icon: '📉', title: '9.5% Interest Rate', desc: 'vs 18-26% at NBFCs', color: kotakColors.primary },
  { icon: '🌉', title: '7-Day Bridge Loan', desc: 'Zero interest to close old loan', color: kotakColors.gold },
  { icon: '🔒', title: 'Rate Lock Guarantee', desc: '12-month rate protection', color: kotakColors.secondary },
  { icon: '🚗', title: 'Transit Insurance', desc: 'Rs 10L coverage during transfer', color: kotakColors.success },
  { icon: '📋', title: 'Switch Assist Service', desc: 'We handle old loan closure', color: kotakColors.gold },
];

export const SwitchSaveProductSlide: React.FC = () => {
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
            THE PRODUCT
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <FlexBox alignItems="center" style={{ gap: '16px', marginBottom: '24px' }}>
            <Text fontSize="48px" margin="0">🏆</Text>
            <Box>
              <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0">
                Switch & Save Gold Loan
              </Heading>
              <Text fontSize="18px" color={kotakColors.gold} margin="8px 0 0 0">
                India's first comprehensive balance transfer product for gold loans
              </Text>
            </Box>
          </FlexBox>
        </MotionBox>

        {/* Feature Grid */}
        <FlexBox flex={1} style={{ gap: '16px' }} flexWrap="wrap">
          {features.map((feature, index) => (
            <MotionBox
              key={index}
              initial={{ opacity: 0, y: 20, scale: 0.95 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              transition={{ duration: 0.4, delay: 0.2 + index * 0.08 }}
              style={{
                flex: '1 1 calc(33.33% - 12px)',
                minWidth: '280px',
                background: kotakColors.darkCard,
                borderRadius: '16px',
                padding: '24px',
                borderTop: `4px solid ${feature.color}`,
              }}
            >
              <FlexBox alignItems="center" style={{ gap: '16px', marginBottom: '12px' }}>
                <Box
                  style={{
                    width: '48px',
                    height: '48px',
                    borderRadius: '12px',
                    background: `${feature.color}15`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  <Text fontSize="24px" margin="0">{feature.icon}</Text>
                </Box>
                <Text fontSize="18px" fontWeight={700} color={kotakColors.white} margin="0">
                  {feature.title}
                </Text>
              </FlexBox>
              <Text fontSize="15px" color={kotakColors.textMuted} margin="0">
                {feature.desc}
              </Text>
            </MotionBox>
          ))}
        </FlexBox>

        {/* Bottom CTA */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.7 }}
          style={{ marginTop: '20px' }}
        >
          <FlexBox
            style={{
              background: `linear-gradient(90deg, ${kotakColors.primary}, ${kotakColors.primaryDark})`,
              borderRadius: '12px',
              padding: '20px 32px',
            }}
            justifyContent="space-between"
            alignItems="center"
          >
            <Box>
              <Text fontSize="20px" fontWeight={700} color={kotakColors.white} margin="0">
                Total Customer Savings
              </Text>
              <Text fontSize="14px" color="rgba(255,255,255,0.8)" margin="4px 0 0 0">
                On average Rs 5L loan for 1 year
              </Text>
            </Box>
            <FlexBox alignItems="baseline" style={{ gap: '8px' }}>
              <Text fontSize="40px" fontWeight={700} color={kotakColors.white} margin="0">
                Rs 62,500
              </Text>
              <Text fontSize="16px" color="rgba(255,255,255,0.8)" margin="0">
                / year
              </Text>
            </FlexBox>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Switch & Save is our flagship product with 6 key features: zero processing fee,
        9.5% rate, 7-day bridge loan, rate lock, transit insurance, and switch assist.
        On a Rs 5L loan, customer saves Rs 62,500/year compared to NBFC rates.
      </Notes>
    </Slide>
  );
};

export default SwitchSaveProductSlide;
