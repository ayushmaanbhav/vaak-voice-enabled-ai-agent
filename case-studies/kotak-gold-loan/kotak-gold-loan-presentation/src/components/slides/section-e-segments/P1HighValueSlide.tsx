import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { customerSegments } from '../../../data/segmentData';

const MotionBox = motion(Box);

const p1 = customerSegments[0]; // P1: High-Value Switchers

const savingsTable = [
  { loan: 'Rs 5 Lakh', nbfc: 'Rs 1,10,000', kotak: 'Rs 47,500', savings: 'Rs 62,500' },
  { loan: 'Rs 10 Lakh', nbfc: 'Rs 2,20,000', kotak: 'Rs 95,000', savings: 'Rs 1,25,000' },
  { loan: 'Rs 25 Lakh', nbfc: 'Rs 5,50,000', kotak: 'Rs 2,37,500', savings: 'Rs 3,12,500' },
];

const targetingChannels = [
  { channel: 'Kotak Business Banking', icon: '🏦', reach: 'Existing MSME customers' },
  { channel: 'CA/Tax Consultant Network', icon: '📊', reach: 'Referral partnerships' },
  { channel: 'Trade Association Events', icon: '🤝', reach: 'Industry associations' },
];

export const P1HighValueSlide: React.FC = () => {
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
            color={p1.color}
            margin="0 0 8px 0"
            style={{ letterSpacing: '2px', textTransform: 'uppercase' }}
          >
            SEGMENT DEEP DIVE: {p1.id}
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            {p1.name}: The Premium Opportunity
          </Heading>
        </MotionBox>

        <FlexBox flex={1} style={{ gap: '24px' }}>
          {/* Left: Profile Card */}
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
                borderTop: `4px solid ${p1.color}`,
              }}
            >
              {/* Profile Header */}
              <FlexBox alignItems="center" style={{ gap: '16px', marginBottom: '20px' }}>
                <Box
                  style={{
                    width: '64px',
                    height: '64px',
                    borderRadius: '16px',
                    background: `${p1.color}20`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  <Text fontSize="32px" margin="0">{p1.icon}</Text>
                </Box>
                <Box>
                  <Text fontSize="20px" fontWeight={700} color={kotakColors.white} margin="0">
                    {p1.profile}
                  </Text>
                  <Text fontSize="14px" color={kotakColors.textMuted} margin="4px 0 0 0">
                    Loan Range: {p1.loanRange}
                  </Text>
                </Box>
              </FlexBox>

              {/* Key Metrics */}
              <FlexBox style={{ gap: '12px', marginBottom: '20px' }}>
                <Box style={{ flex: 1, background: 'rgba(255,255,255,0.03)', borderRadius: '10px', padding: '14px', textAlign: 'center' }}>
                  <Text fontSize="24px" fontWeight={700} color={p1.color} margin="0">{p1.conversionRate}</Text>
                  <Text fontSize="12px" color={kotakColors.textMuted} margin="4px 0 0 0">Conversion Rate</Text>
                </Box>
                <Box style={{ flex: 1, background: 'rgba(255,255,255,0.03)', borderRadius: '10px', padding: '14px', textAlign: 'center' }}>
                  <Text fontSize="24px" fontWeight={700} color={kotakColors.gold} margin="0">Rs 2,500</Text>
                  <Text fontSize="12px" color={kotakColors.textMuted} margin="4px 0 0 0">Target CAC</Text>
                </Box>
              </FlexBox>

              {/* Targeting Channels */}
              <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0 0 12px 0">
                TARGETING CHANNELS
              </Text>
              <FlexBox flexDirection="column" style={{ gap: '8px' }}>
                {targetingChannels.map((ch, i) => (
                  <FlexBox key={i} alignItems="center" style={{ gap: '12px', padding: '10px 12px', background: 'rgba(255,255,255,0.03)', borderRadius: '8px' }}>
                    <Text fontSize="20px" margin="0">{ch.icon}</Text>
                    <Box>
                      <Text fontSize="14px" fontWeight={500} color={kotakColors.white} margin="0">{ch.channel}</Text>
                      <Text fontSize="12px" color={kotakColors.textMuted} margin="0">{ch.reach}</Text>
                    </Box>
                  </FlexBox>
                ))}
              </FlexBox>
            </Box>
          </MotionBox>

          {/* Right: Savings Impact */}
          <MotionBox
            initial={{ opacity: 0, x: 30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            style={{ flex: 1.2 }}
          >
            <Box
              style={{
                background: kotakColors.darkCard,
                borderRadius: '16px',
                padding: '24px',
                height: '100%',
              }}
            >
              <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0 0 16px 0">
                Annual Savings Impact (NBFC 22% vs Kotak 9.5%)
              </Text>

              {/* Table Header */}
              <FlexBox style={{ padding: '12px 16px', background: p1.color, borderRadius: '8px 8px 0 0' }}>
                <Box style={{ flex: 1 }}><Text fontSize="13px" fontWeight={600} color={kotakColors.white} margin="0">Loan Amount</Text></Box>
                <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="13px" fontWeight={600} color={kotakColors.white} margin="0">NBFC Interest</Text></Box>
                <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="13px" fontWeight={600} color={kotakColors.white} margin="0">Kotak Interest</Text></Box>
                <Box style={{ flex: 1, textAlign: 'right' }}><Text fontSize="13px" fontWeight={600} color={kotakColors.white} margin="0">Annual Savings</Text></Box>
              </FlexBox>

              {/* Table Rows */}
              {savingsTable.map((row, i) => (
                <FlexBox
                  key={i}
                  style={{
                    padding: '14px 16px',
                    background: i % 2 === 0 ? 'rgba(255,255,255,0.02)' : 'transparent',
                    borderBottom: '1px solid rgba(255,255,255,0.05)',
                  }}
                  alignItems="center"
                >
                  <Box style={{ flex: 1 }}><Text fontSize="15px" fontWeight={600} color={kotakColors.white} margin="0">{row.loan}</Text></Box>
                  <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="15px" color={kotakColors.danger} margin="0">{row.nbfc}</Text></Box>
                  <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="15px" color={kotakColors.success} margin="0">{row.kotak}</Text></Box>
                  <Box style={{ flex: 1, textAlign: 'right' }}><Text fontSize="17px" fontWeight={700} color={kotakColors.gold} margin="0">{row.savings}</Text></Box>
                </FlexBox>
              ))}

              {/* Key Message */}
              <Box style={{ marginTop: '20px', padding: '16px', background: `${kotakColors.gold}15`, borderRadius: '10px' }}>
                <FlexBox alignItems="center" style={{ gap: '12px' }}>
                  <Text fontSize="28px" margin="0">💼</Text>
                  <Box>
                    <Text fontSize="15px" fontWeight={600} color={kotakColors.gold} margin="0">
                      Key Pitch for MSME Owners
                    </Text>
                    <Text fontSize="14px" color={kotakColors.textMuted} margin="4px 0 0 0">
                      "Rs 1-3 lakh annual savings = One more employee or inventory"
                    </Text>
                  </Box>
                </FlexBox>
              </Box>
            </Box>
          </MotionBox>
        </FlexBox>
      </FlexBox>

      <Notes>
        P1 is our highest-value segment: MSME owners with Rs 5-25 lakh loans.
        At 8-12% conversion rate with Rs 2,500 CAC, they deliver best unit economics.
        Key pitch: savings translate to business reinvestment - one more employee, more inventory.
      </Notes>
    </Slide>
  );
};

export default P1HighValueSlide;
