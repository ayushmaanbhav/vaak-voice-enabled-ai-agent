import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const savingsExamples = [
  { loan: 'Rs 2 Lakh', nbfcInterest: 'Rs 44,000', kotakInterest: 'Rs 19,000', savings: 'Rs 25,000', years: 1 },
  { loan: 'Rs 5 Lakh', nbfcInterest: 'Rs 1,10,000', kotakInterest: 'Rs 47,500', savings: 'Rs 62,500', years: 1 },
  { loan: 'Rs 10 Lakh', nbfcInterest: 'Rs 2,20,000', kotakInterest: 'Rs 95,000', savings: 'Rs 1,25,000', years: 1 },
  { loan: 'Rs 25 Lakh', nbfcInterest: 'Rs 5,50,000', kotakInterest: 'Rs 2,37,500', savings: 'Rs 3,12,500', years: 1 },
];

export const SavingsCalculatorSlide: React.FC = () => {
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
            SAVINGS IMPACT
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 8px 0">
            Real Savings, Real Impact
          </Heading>
          <Text fontSize="18px" color={kotakColors.textMuted} margin="0 0 32px 0">
            Comparing NBFC rate (22% avg) vs Kotak rate (9.5%)
          </Text>
        </MotionBox>

        {/* Savings Table */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.2 }}
          style={{ flex: 1 }}
        >
          <Box
            style={{
              background: kotakColors.darkCard,
              borderRadius: '16px',
              overflow: 'hidden',
            }}
          >
            {/* Header */}
            <FlexBox
              style={{
                background: kotakColors.primary,
                padding: '16px 24px',
              }}
            >
              <Box style={{ flex: 1 }}>
                <Text fontSize="15px" fontWeight={600} color={kotakColors.white} margin="0">
                  Loan Amount
                </Text>
              </Box>
              <Box style={{ flex: 1, textAlign: 'center' }}>
                <Text fontSize="15px" fontWeight={600} color={kotakColors.white} margin="0">
                  NBFC Interest (22%)
                </Text>
              </Box>
              <Box style={{ flex: 1, textAlign: 'center' }}>
                <Text fontSize="15px" fontWeight={600} color={kotakColors.white} margin="0">
                  Kotak Interest (9.5%)
                </Text>
              </Box>
              <Box style={{ flex: 1, textAlign: 'right' }}>
                <Text fontSize="15px" fontWeight={600} color={kotakColors.white} margin="0">
                  Annual Savings
                </Text>
              </Box>
            </FlexBox>

            {/* Rows */}
            {savingsExamples.map((row, index) => (
              <MotionBox
                key={index}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.4, delay: 0.3 + index * 0.1 }}
              >
                <FlexBox
                  style={{
                    padding: '20px 24px',
                    borderBottom: index < savingsExamples.length - 1 ? '1px solid rgba(255,255,255,0.1)' : 'none',
                    background: index % 2 === 1 ? 'rgba(255,255,255,0.02)' : 'transparent',
                  }}
                  alignItems="center"
                >
                  <Box style={{ flex: 1 }}>
                    <Text fontSize="18px" fontWeight={600} color={kotakColors.white} margin="0">
                      {row.loan}
                    </Text>
                  </Box>
                  <Box style={{ flex: 1, textAlign: 'center' }}>
                    <Text fontSize="18px" color={kotakColors.danger} margin="0">
                      {row.nbfcInterest}
                    </Text>
                  </Box>
                  <Box style={{ flex: 1, textAlign: 'center' }}>
                    <Text fontSize="18px" color={kotakColors.success} margin="0">
                      {row.kotakInterest}
                    </Text>
                  </Box>
                  <Box style={{ flex: 1, textAlign: 'right' }}>
                    <Text fontSize="20px" fontWeight={700} color={kotakColors.gold} margin="0">
                      {row.savings}
                    </Text>
                  </Box>
                </FlexBox>
              </MotionBox>
            ))}
          </Box>
        </MotionBox>

        {/* Bottom Callout */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.7 }}
          style={{ marginTop: '24px' }}
        >
          <FlexBox
            style={{
              background: `linear-gradient(90deg, ${kotakColors.gold}15, ${kotakColors.gold}05)`,
              borderRadius: '12px',
              padding: '20px 28px',
              border: `1px solid ${kotakColors.gold}30`,
            }}
            alignItems="center"
            justifyContent="space-between"
          >
            <Box>
              <Text fontSize="16px" color={kotakColors.textMuted} margin="0 0 4px 0">
                Average NBFC Customer Loan
              </Text>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.white} margin="0">
                Rs 1.5 - 3 Lakh
              </Text>
            </Box>
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="48px" margin="0">💡</Text>
            </Box>
            <Box style={{ textAlign: 'right' }}>
              <Text fontSize="16px" color={kotakColors.textMuted} margin="0 0 4px 0">
                Typical Annual Savings
              </Text>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.gold} margin="0">
                Rs 20,000 - Rs 40,000
              </Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        This table shows concrete savings at different loan amounts.
        Average NBFC customer has Rs 1.5-3 lakh loan, saving Rs 20-40K annually.
        For a small business owner, this is significant working capital freed up.
      </Notes>
    </Slide>
  );
};

export default SavingsCalculatorSlide;
