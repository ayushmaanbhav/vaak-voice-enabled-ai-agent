import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const investmentBreakdown = [
  { category: 'Computer Vision Appraisal', amount: 'Rs 6 Cr', percentage: 41, color: kotakColors.primary, priority: 'P0' },
  { category: 'Predictive Acquisition Model', amount: 'Rs 5 Cr', percentage: 34, color: kotakColors.gold, priority: 'P1' },
  { category: 'Personalized Pitching', amount: 'Rs 3.5 Cr', percentage: 24, color: kotakColors.success, priority: 'P1' },
];

const timeline = [
  { quarter: 'Q1', activities: ['Model training data collection', 'CV model development start'], status: 'Foundation' },
  { quarter: 'Q2-Q3', activities: ['CV pilot in 10 branches', 'Predictive model training'], status: 'Development' },
  { quarter: 'Q4', activities: ['CV rollout 100 branches', 'Pitching engine pilot'], status: 'Deployment' },
  { quarter: 'Y2', activities: ['Full AI stack operational', 'Continuous optimization'], status: 'Scale' },
];

export const TechInvestmentSlide: React.FC = () => {
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
            TECHNOLOGY INVESTMENT
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Rs 14.5 Cr AI Investment Summary
          </Heading>
        </MotionBox>

        <FlexBox flex={1} style={{ gap: '24px' }}>
          {/* Left: Investment Breakdown */}
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
                Investment by Solution
              </Text>

              <FlexBox flexDirection="column" style={{ gap: '16px' }}>
                {investmentBreakdown.map((item, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.4, delay: 0.3 + index * 0.1 }}
                  >
                    <FlexBox alignItems="center" justifyContent="space-between" style={{ marginBottom: '8px' }}>
                      <FlexBox alignItems="center" style={{ gap: '8px' }}>
                        <Box style={{ padding: '2px 6px', background: `${item.color}30`, borderRadius: '4px' }}>
                          <Text fontSize="10px" fontWeight={700} color={item.color} margin="0">{item.priority}</Text>
                        </Box>
                        <Text fontSize="14px" fontWeight={500} color={kotakColors.white} margin="0">{item.category}</Text>
                      </FlexBox>
                      <Text fontSize="16px" fontWeight={700} color={item.color} margin="0">{item.amount}</Text>
                    </FlexBox>
                    <Box style={{ width: '100%', height: '8px', background: 'rgba(255,255,255,0.1)', borderRadius: '4px', overflow: 'hidden' }}>
                      <Box style={{ width: `${item.percentage}%`, height: '100%', background: item.color, borderRadius: '4px' }} />
                    </Box>
                    <Text fontSize="11px" color={kotakColors.textMuted} margin="4px 0 0 0">{item.percentage}% of total</Text>
                  </MotionBox>
                ))}
              </FlexBox>

              {/* Total */}
              <Box style={{ marginTop: '20px', padding: '16px', background: 'rgba(255,255,255,0.03)', borderRadius: '10px' }}>
                <FlexBox justifyContent="space-between" alignItems="center">
                  <Text fontSize="14px" fontWeight={600} color={kotakColors.textMuted} margin="0">TOTAL INVESTMENT</Text>
                  <Text fontSize="24px" fontWeight={700} color={kotakColors.gold} margin="0">Rs 14.5 Cr</Text>
                </FlexBox>
              </Box>
            </Box>
          </MotionBox>

          {/* Right: Timeline */}
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
                Implementation Timeline
              </Text>

              <FlexBox flexDirection="column" style={{ gap: '12px' }}>
                {timeline.map((item, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.3, delay: 0.4 + index * 0.1 }}
                    style={{
                      background: 'rgba(255,255,255,0.03)',
                      borderRadius: '10px',
                      padding: '14px 16px',
                      borderLeft: `3px solid ${index === 0 ? kotakColors.primary : index === 1 ? kotakColors.gold : index === 2 ? kotakColors.success : kotakColors.secondary}`,
                    }}
                  >
                    <FlexBox alignItems="center" justifyContent="space-between" style={{ marginBottom: '8px' }}>
                      <Text fontSize="16px" fontWeight={700} color={kotakColors.white} margin="0">{item.quarter}</Text>
                      <Box style={{ padding: '4px 10px', background: `${kotakColors.primary}20`, borderRadius: '4px' }}>
                        <Text fontSize="11px" fontWeight={600} color={kotakColors.primary} margin="0">{item.status}</Text>
                      </Box>
                    </FlexBox>
                    <FlexBox flexDirection="column" style={{ gap: '4px' }}>
                      {item.activities.map((activity, i) => (
                        <FlexBox key={i} alignItems="center" style={{ gap: '8px' }}>
                          <Box style={{ width: '5px', height: '5px', borderRadius: '50%', background: kotakColors.textMuted }} />
                          <Text fontSize="12px" color={kotakColors.textMuted} margin="0">{activity}</Text>
                        </FlexBox>
                      ))}
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>
            </Box>
          </MotionBox>
        </FlexBox>

        {/* Bottom Context */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.8 }}
          style={{ marginTop: '16px' }}
        >
          <FlexBox
            style={{
              background: `linear-gradient(90deg, ${kotakColors.gold}15, ${kotakColors.gold}05)`,
              borderRadius: '12px',
              padding: '14px 24px',
              border: `1px solid ${kotakColors.gold}30`,
            }}
            justifyContent="space-around"
            alignItems="center"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="18px" fontWeight={700} color={kotakColors.gold} margin="0">9.8%</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">of Total Program Investment</Text>
            </Box>
            <Box style={{ width: '1px', height: '35px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="18px" fontWeight={700} color={kotakColors.success} margin="0">3.2x</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Blended ROI</Text>
            </Box>
            <Box style={{ width: '1px', height: '35px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="18px" fontWeight={700} color={kotakColors.primary} margin="0">18 Months</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Full Stack Operational</Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Total AI investment: Rs 14.5 Cr over 18 months. Computer Vision is highest priority (Rs 6 Cr),
        followed by Predictive Model (Rs 5 Cr) and Personalized Pitching (Rs 3.5 Cr).
        This is 9.8% of total program investment with 3.2x blended ROI.
      </Notes>
    </Slide>
  );
};

export default TechInvestmentSlide;
