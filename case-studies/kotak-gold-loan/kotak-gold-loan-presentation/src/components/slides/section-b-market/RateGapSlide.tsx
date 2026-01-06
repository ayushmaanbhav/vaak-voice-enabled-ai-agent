import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { RateComparisonChart } from '../../charts/HorizontalBarChart';
import { interestRateComparison } from '../../../data/marketData';

const MotionBox = motion(Box);

export const RateGapSlide: React.FC = () => {
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
            THE RATE ADVANTAGE
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 16px 0">
            The 40-50% Interest Rate Gap
          </Heading>
          <Text fontSize="20px" color={kotakColors.textMuted} margin="0 0 32px 0">
            NBFC customers are paying significantly more than bank rates
          </Text>
        </MotionBox>

        {/* Chart */}
        <MotionBox
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.5, delay: 0.2 }}
          style={{ width: '100%' }}
        >
          <Box
            style={{
              background: kotakColors.darkCard,
              borderRadius: '16px',
              padding: '32px',
            }}
          >
            <RateComparisonChart data={interestRateComparison} height={300} />
          </Box>
        </MotionBox>

        {/* Bottom Stats */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.5 }}
          style={{ marginTop: '24px' }}
        >
          <FlexBox style={{ gap: '24px' }}>
            <Box
              style={{
                flex: 1,
                background: `linear-gradient(135deg, ${kotakColors.success}15, ${kotakColors.success}05)`,
                borderRadius: '12px',
                padding: '20px 24px',
                borderLeft: `4px solid ${kotakColors.success}`,
              }}
            >
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">
                Kotak Rate
              </Text>
              <Text fontSize="28px" fontWeight={700} color={kotakColors.success} margin="0">
                9.5% p.a.
              </Text>
            </Box>
            <Box
              style={{
                flex: 1,
                background: `linear-gradient(135deg, ${kotakColors.danger}15, ${kotakColors.danger}05)`,
                borderRadius: '12px',
                padding: '20px 24px',
                borderLeft: `4px solid ${kotakColors.danger}`,
              }}
            >
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">
                NBFC Average Rate
              </Text>
              <Text fontSize="28px" fontWeight={700} color={kotakColors.danger} margin="0">
                18-26% p.a.
              </Text>
            </Box>
            <Box
              style={{
                flex: 1,
                background: `linear-gradient(135deg, ${kotakColors.gold}15, ${kotakColors.gold}05)`,
                borderRadius: '12px',
                padding: '20px 24px',
                borderLeft: `4px solid ${kotakColors.gold}`,
              }}
            >
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">
                Customer Savings Potential
              </Text>
              <Text fontSize="28px" fontWeight={700} color={kotakColors.gold} margin="0">
                Rs 40,000/year
              </Text>
              <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">
                On a Rs 5L loan
              </Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        The rate differential is stark: Banks charge 9-12% while NBFCs charge 18-26%.
        On a Rs 5 lakh loan, this translates to Rs 40,000 annual savings.
        This is the core value proposition for our Switch & Save product.
      </Notes>
    </Slide>
  );
};

export default RateGapSlide;
