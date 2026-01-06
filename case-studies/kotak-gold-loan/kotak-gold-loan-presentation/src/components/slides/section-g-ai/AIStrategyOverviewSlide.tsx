import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { aiSolutions } from '../../../data/aiData';

const MotionBox = motion(Box);

export const AIStrategyOverviewSlide: React.FC = () => {
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
            AI & TECHNOLOGY
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Three AI Solutions for Competitive Advantage
          </Heading>
        </MotionBox>

        {/* Solution Cards */}
        <FlexBox flex={1} style={{ gap: '20px' }}>
          {aiSolutions.map((solution, index) => (
            <MotionBox
              key={index}
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.2 + index * 0.15 }}
              style={{
                flex: 1,
                background: kotakColors.darkCard,
                borderRadius: '16px',
                padding: '24px',
                borderTop: `4px solid ${solution.color}`,
                display: 'flex',
                flexDirection: 'column',
              }}
            >
              {/* Header */}
              <FlexBox alignItems="center" style={{ gap: '12px', marginBottom: '16px' }}>
                <Box
                  style={{
                    width: '52px',
                    height: '52px',
                    borderRadius: '14px',
                    background: `${solution.color}20`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  <Text fontSize="26px" margin="0">{solution.icon}</Text>
                </Box>
                <Box>
                  <Text fontSize="18px" fontWeight={700} color={kotakColors.white} margin="0">
                    {solution.name}
                  </Text>
                  <Box style={{ padding: '2px 8px', background: `${solution.color}20`, borderRadius: '4px', marginTop: '4px', display: 'inline-block' }}>
                    <Text fontSize="11px" fontWeight={600} color={solution.color} margin="0">
                      {solution.priority}
                    </Text>
                  </Box>
                </Box>
              </FlexBox>

              {/* Description */}
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 16px 0" style={{ lineHeight: 1.5, flex: 1 }}>
                {solution.description}
              </Text>

              {/* Key Benefits */}
              <Box style={{ marginBottom: '16px' }}>
                <Text fontSize="12px" fontWeight={600} color={kotakColors.textMuted} margin="0 0 8px 0">
                  KEY BENEFITS
                </Text>
                <FlexBox flexDirection="column" style={{ gap: '6px' }}>
                  {solution.benefits.slice(0, 3).map((benefit, i) => (
                    <FlexBox key={i} alignItems="center" style={{ gap: '8px' }}>
                      <Box style={{ width: '6px', height: '6px', borderRadius: '50%', background: solution.color }} />
                      <Text fontSize="13px" color={kotakColors.white} margin="0">{benefit}</Text>
                    </FlexBox>
                  ))}
                </FlexBox>
              </Box>

              {/* ROI & Investment */}
              <FlexBox style={{ gap: '12px' }}>
                <Box style={{ flex: 1, background: 'rgba(255,255,255,0.03)', borderRadius: '8px', padding: '12px', textAlign: 'center' }}>
                  <Text fontSize="18px" fontWeight={700} color={solution.color} margin="0">{solution.roi}</Text>
                  <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">ROI</Text>
                </Box>
                <Box style={{ flex: 1, background: 'rgba(255,255,255,0.03)', borderRadius: '8px', padding: '12px', textAlign: 'center' }}>
                  <Text fontSize="18px" fontWeight={700} color={kotakColors.white} margin="0">{solution.investment}</Text>
                  <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Investment</Text>
                </Box>
              </FlexBox>
            </MotionBox>
          ))}
        </FlexBox>

        {/* Total Investment */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.7 }}
          style={{ marginTop: '16px' }}
        >
          <FlexBox
            style={{
              background: `linear-gradient(90deg, ${kotakColors.gold}15, ${kotakColors.gold}05)`,
              borderRadius: '12px',
              padding: '16px 24px',
              border: `1px solid ${kotakColors.gold}30`,
            }}
            justifyContent="space-around"
            alignItems="center"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.gold} margin="0">Rs 14.5 Cr</Text>
              <Text fontSize="12px" color={kotakColors.textMuted} margin="4px 0 0 0">Total AI Investment (3 Years)</Text>
            </Box>
            <Box style={{ width: '1px', height: '40px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.success} margin="0">3.2x</Text>
              <Text fontSize="12px" color={kotakColors.textMuted} margin="4px 0 0 0">Expected Blended ROI</Text>
            </Box>
            <Box style={{ width: '1px', height: '40px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.primary} margin="0">18 Months</Text>
              <Text fontSize="12px" color={kotakColors.textMuted} margin="4px 0 0 0">Full Deployment Timeline</Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Three AI solutions: Computer Vision for appraisal (highest priority), Predictive Model for
        acquisition targeting, Personalized Pitching for segment-specific offers.
        Total investment Rs 14.5 Cr over 3 years with 3.2x blended ROI.
      </Notes>
    </Slide>
  );
};

export default AIStrategyOverviewSlide;
