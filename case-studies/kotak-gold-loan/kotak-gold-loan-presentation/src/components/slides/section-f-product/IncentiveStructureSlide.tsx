import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const whatWeOffer = [
  { item: 'Zero processing fee (6 months)', value: 'Worth Rs 2,500-5,000', icon: '✅' },
  { item: 'Rate match guarantee', value: 'Match competitor rates if needed', icon: '✅' },
  { item: 'Referral bonus', value: 'Rs 500 per successful referral', icon: '✅' },
  { item: 'Loyalty rewards', value: 'Rate reduction on renewal', icon: '✅' },
  { item: 'Festival offers', value: 'Seasonal promotions', icon: '✅' },
];

const whatWeAvoid = [
  { item: 'Cash incentives', reason: 'Compliance risk, wrong customer type', icon: '❌' },
  { item: 'Gold buyback schemes', reason: 'Regulatory grey area', icon: '❌' },
  { item: 'Aggressive agent commissions', reason: 'Quality vs quantity', icon: '❌' },
  { item: 'Rate undercutting wars', reason: 'Margin erosion', icon: '❌' },
];

export const IncentiveStructureSlide: React.FC = () => {
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
            INCENTIVE STRATEGY
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Smart Incentives: Quality Over Gimmicks
          </Heading>
        </MotionBox>

        <FlexBox flex={1} style={{ gap: '32px' }}>
          {/* What We Offer */}
          <MotionBox
            initial={{ opacity: 0, x: -30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            style={{ flex: 1 }}
          >
            <Box
              style={{
                background: kotakColors.darkCard,
                borderRadius: '16px',
                padding: '24px',
                height: '100%',
                borderTop: `4px solid ${kotakColors.success}`,
              }}
            >
              <FlexBox alignItems="center" style={{ gap: '12px', marginBottom: '20px' }}>
                <Text fontSize="28px" margin="0">✨</Text>
                <Text fontSize="20px" fontWeight={700} color={kotakColors.success} margin="0">
                  What We Offer
                </Text>
              </FlexBox>

              <FlexBox flexDirection="column" style={{ gap: '12px' }}>
                {whatWeOffer.map((item, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.3, delay: 0.3 + index * 0.08 }}
                  >
                    <FlexBox
                      alignItems="center"
                      style={{
                        gap: '12px',
                        padding: '14px 16px',
                        background: `${kotakColors.success}08`,
                        borderRadius: '10px',
                        border: `1px solid ${kotakColors.success}20`,
                      }}
                    >
                      <Text fontSize="20px" margin="0">{item.icon}</Text>
                      <Box style={{ flex: 1 }}>
                        <Text fontSize="15px" fontWeight={500} color={kotakColors.white} margin="0">
                          {item.item}
                        </Text>
                        <Text fontSize="13px" color={kotakColors.success} margin="2px 0 0 0">
                          {item.value}
                        </Text>
                      </Box>
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>
            </Box>
          </MotionBox>

          {/* What We Avoid */}
          <MotionBox
            initial={{ opacity: 0, x: 30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            style={{ flex: 1 }}
          >
            <Box
              style={{
                background: kotakColors.darkCard,
                borderRadius: '16px',
                padding: '24px',
                height: '100%',
                borderTop: `4px solid ${kotakColors.danger}`,
              }}
            >
              <FlexBox alignItems="center" style={{ gap: '12px', marginBottom: '20px' }}>
                <Text fontSize="28px" margin="0">🚫</Text>
                <Text fontSize="20px" fontWeight={700} color={kotakColors.danger} margin="0">
                  What We Avoid
                </Text>
              </FlexBox>

              <FlexBox flexDirection="column" style={{ gap: '12px' }}>
                {whatWeAvoid.map((item, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.3, delay: 0.4 + index * 0.08 }}
                  >
                    <FlexBox
                      alignItems="center"
                      style={{
                        gap: '12px',
                        padding: '14px 16px',
                        background: 'rgba(255,255,255,0.03)',
                        borderRadius: '10px',
                      }}
                    >
                      <Text fontSize="20px" margin="0">{item.icon}</Text>
                      <Box style={{ flex: 1 }}>
                        <Text fontSize="15px" fontWeight={500} color={kotakColors.white} margin="0">
                          {item.item}
                        </Text>
                        <Text fontSize="13px" color={kotakColors.textMuted} margin="2px 0 0 0">
                          {item.reason}
                        </Text>
                      </Box>
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>
            </Box>
          </MotionBox>
        </FlexBox>

        {/* Philosophy */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.7 }}
          style={{ marginTop: '20px' }}
        >
          <Box
            style={{
              background: `linear-gradient(90deg, ${kotakColors.gold}15, ${kotakColors.gold}05)`,
              borderRadius: '12px',
              padding: '20px 28px',
              border: `1px solid ${kotakColors.gold}30`,
            }}
          >
            <FlexBox alignItems="center" style={{ gap: '16px' }}>
              <Text fontSize="32px" margin="0">💡</Text>
              <Box>
                <Text fontSize="16px" fontWeight={600} color={kotakColors.gold} margin="0 0 4px 0">
                  Core Philosophy
                </Text>
                <Text fontSize="15px" color={kotakColors.white} margin="0">
                  Value comes from lower rates + better service, not gimmicks. Customers switching for cash incentives
                  switch again. Customers switching for real value stay.
                </Text>
              </Box>
            </FlexBox>
          </Box>
        </MotionBox>
      </FlexBox>

      <Notes>
        Our incentive structure focuses on sustainable value, not gimmicks.
        We offer processing fee waivers, rate match, referral bonuses - things that drive quality.
        We avoid cash incentives and aggressive commissions that attract wrong customers.
      </Notes>
    </Slide>
  );
};

export default IncentiveStructureSlide;
