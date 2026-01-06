import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const contextComparisons = [
  { item: 'Total Investment', value: 'Rs 148 Cr', percentage: '100%', color: kotakColors.primary },
  { item: '% of Annual ICT Budget', value: 'Rs 148 Cr', percentage: '3.2%', color: kotakColors.gold },
  { item: '% of Quarterly PAT', value: 'Rs 148 Cr', percentage: '1.6%', color: kotakColors.success },
  { item: '% of Marketing Budget', value: 'Rs 148 Cr', percentage: '18%', color: kotakColors.secondary },
];

const investmentBreakdown = [
  { category: 'Customer Acquisition', amount: 'Rs 70 Cr', percentage: 47, icon: '👥' },
  { category: 'Technology & AI', amount: 'Rs 38 Cr', percentage: 26, icon: '🤖' },
  { category: 'Operations & Training', amount: 'Rs 25 Cr', percentage: 17, icon: '⚙️' },
  { category: 'Marketing & Brand', amount: 'Rs 15 Cr', percentage: 10, icon: '📢' },
];

export const InvestmentContextSlide: React.FC = () => {
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
            INVESTMENT CONTEXT
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Rs 148 Cr in Perspective
          </Heading>
        </MotionBox>

        <FlexBox flex={1} style={{ gap: '24px' }}>
          {/* Left: Context Comparisons */}
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
              }}
            >
              <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0 0 20px 0">
                As % of Kotak Financials
              </Text>

              <FlexBox flexDirection="column" style={{ gap: '12px' }}>
                {contextComparisons.slice(1).map((item, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.4, delay: 0.3 + index * 0.1 }}
                  >
                    <Box
                      style={{
                        background: 'rgba(255,255,255,0.03)',
                        borderRadius: '10px',
                        padding: '16px',
                      }}
                    >
                      <FlexBox alignItems="center" justifyContent="space-between" style={{ marginBottom: '10px' }}>
                        <Text fontSize="14px" color={kotakColors.white} margin="0">{item.item}</Text>
                        <Text fontSize="28px" fontWeight={700} color={item.color} margin="0">{item.percentage}</Text>
                      </FlexBox>
                      <Box style={{ width: '100%', height: '8px', background: 'rgba(255,255,255,0.1)', borderRadius: '4px', overflow: 'hidden' }}>
                        <Box
                          style={{
                            width: item.percentage === '3.2%' ? '32%' : item.percentage === '1.6%' ? '16%' : '100%',
                            height: '100%',
                            background: item.color,
                            borderRadius: '4px',
                          }}
                        />
                      </Box>
                    </Box>
                  </MotionBox>
                ))}
              </FlexBox>

              {/* Key Point */}
              <Box style={{ marginTop: '16px', padding: '14px', background: `${kotakColors.success}10`, borderRadius: '10px', border: `1px solid ${kotakColors.success}30` }}>
                <Text fontSize="14px" color={kotakColors.white} margin="0">
                  <Text fontSize="14px" fontWeight={700} color={kotakColors.success} style={{ display: 'inline' }}>Just 1.6%</Text> of quarterly PAT
                  to build a Rs 8,500 Cr AUM business.
                </Text>
              </Box>
            </Box>
          </MotionBox>

          {/* Right: Investment Breakdown */}
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
              }}
            >
              <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0 0 20px 0">
                Investment Breakdown by Category
              </Text>

              <FlexBox flexDirection="column" style={{ gap: '14px' }}>
                {investmentBreakdown.map((item, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.3, delay: 0.4 + index * 0.1 }}
                  >
                    <FlexBox alignItems="center" style={{ gap: '12px', marginBottom: '8px' }}>
                      <Text fontSize="24px" margin="0">{item.icon}</Text>
                      <Box style={{ flex: 1 }}>
                        <FlexBox justifyContent="space-between" alignItems="center">
                          <Text fontSize="14px" fontWeight={500} color={kotakColors.white} margin="0">{item.category}</Text>
                          <Text fontSize="15px" fontWeight={700} color={kotakColors.gold} margin="0">{item.amount}</Text>
                        </FlexBox>
                      </Box>
                    </FlexBox>
                    <Box style={{ width: '100%', height: '10px', background: 'rgba(255,255,255,0.1)', borderRadius: '5px', overflow: 'hidden' }}>
                      <Box
                        style={{
                          width: `${item.percentage}%`,
                          height: '100%',
                          background: `linear-gradient(90deg, ${kotakColors.primary}, ${kotakColors.gold})`,
                          borderRadius: '5px',
                        }}
                      />
                    </Box>
                    <Text fontSize="11px" color={kotakColors.textMuted} margin="4px 0 0 0">{item.percentage}% of total</Text>
                  </MotionBox>
                ))}
              </FlexBox>
            </Box>
          </MotionBox>
        </FlexBox>

        {/* Bottom Summary */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.7 }}
          style={{ marginTop: '16px' }}
        >
          <FlexBox
            style={{
              background: kotakColors.darkCard,
              borderRadius: '12px',
              padding: '14px 24px',
            }}
            justifyContent="space-around"
            alignItems="center"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="18px" fontWeight={700} color={kotakColors.gold} margin="0">Rs 148 Cr</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Total 3-Year Investment</Text>
            </Box>
            <Box style={{ width: '1px', height: '35px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="18px" fontWeight={700} color={kotakColors.success} margin="0">Rs 363 Cr</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Expected PAT Returns</Text>
            </Box>
            <Box style={{ width: '1px', height: '35px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="18px" fontWeight={700} color={kotakColors.primary} margin="0">Rs 8,500 Cr</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">New AUM Created</Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Rs 148 Cr investment is just 3.2% of ICT budget, 1.6% of quarterly PAT.
        Breakdown: 47% customer acquisition, 26% technology, 17% operations, 10% marketing.
        Minimal relative investment for significant new business line creation.
      </Notes>
    </Slide>
  );
};

export default InvestmentContextSlide;
