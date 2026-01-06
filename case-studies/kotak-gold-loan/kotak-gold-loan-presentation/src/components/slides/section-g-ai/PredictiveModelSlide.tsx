import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { predictiveModel } from '../../../data/aiData';

const MotionBox = motion(Box);

export const PredictiveModelSlide: React.FC = () => {
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
            AI SOLUTION #2
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Predictive Acquisition Model
          </Heading>
        </MotionBox>

        <FlexBox flex={1} style={{ gap: '24px' }}>
          {/* Left: Signal Categories */}
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
                Propensity Signal Categories
              </Text>

              <FlexBox flexDirection="column" style={{ gap: '12px' }}>
                {predictiveModel.signalCategories.map((category, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.3, delay: 0.3 + index * 0.08 }}
                    style={{
                      background: 'rgba(255,255,255,0.03)',
                      borderRadius: '10px',
                      padding: '14px 16px',
                      borderLeft: `3px solid ${category.color}`,
                    }}
                  >
                    <FlexBox alignItems="center" justifyContent="space-between">
                      <FlexBox alignItems="center" style={{ gap: '12px' }}>
                        <Text fontSize="22px" margin="0">{category.icon}</Text>
                        <Box>
                          <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0">
                            {category.category}
                          </Text>
                          <Text fontSize="12px" color={kotakColors.textMuted} margin="2px 0 0 0">
                            {category.signals.slice(0, 2).join(', ')}
                          </Text>
                        </Box>
                      </FlexBox>
                      <Box style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                        <Box style={{ width: '60px', height: '6px', background: 'rgba(255,255,255,0.1)', borderRadius: '3px', overflow: 'hidden' }}>
                          <Box style={{ width: `${category.weight}%`, height: '100%', background: category.color, borderRadius: '3px' }} />
                        </Box>
                        <Text fontSize="12px" fontWeight={600} color={category.color} margin="0">{category.weight}%</Text>
                      </Box>
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>
            </Box>
          </MotionBox>

          {/* Right: Model Output & Metrics */}
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
                Model Performance
              </Text>

              {/* Key Metrics */}
              <FlexBox style={{ gap: '12px', marginBottom: '20px' }}>
                {[
                  { value: '16x', label: 'Conversion Lift', desc: 'vs random targeting', color: kotakColors.success },
                  { value: '72%', label: 'AUC Score', desc: 'Model accuracy', color: kotakColors.gold },
                  { value: '45%', label: 'CAC Reduction', desc: 'Lower acquisition cost', color: kotakColors.primary },
                ].map((metric, i) => (
                  <Box key={i} style={{ flex: 1, background: 'rgba(255,255,255,0.03)', borderRadius: '10px', padding: '16px', textAlign: 'center' }}>
                    <Text fontSize="28px" fontWeight={700} color={metric.color} margin="0">{metric.value}</Text>
                    <Text fontSize="13px" fontWeight={600} color={kotakColors.white} margin="4px 0 2px 0">{metric.label}</Text>
                    <Text fontSize="11px" color={kotakColors.textMuted} margin="0">{metric.desc}</Text>
                  </Box>
                ))}
              </FlexBox>

              {/* Example Output */}
              <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0 0 12px 0">
                Sample Model Output
              </Text>
              <Box
                style={{
                  background: 'rgba(255,255,255,0.03)',
                  borderRadius: '10px',
                  padding: '16px',
                  fontFamily: 'monospace',
                }}
              >
                <FlexBox justifyContent="space-between" style={{ marginBottom: '8px' }}>
                  <Text fontSize="12px" color={kotakColors.textMuted} margin="0">Customer ID</Text>
                  <Text fontSize="12px" color={kotakColors.white} margin="0">KOT-8523145</Text>
                </FlexBox>
                <FlexBox justifyContent="space-between" style={{ marginBottom: '8px' }}>
                  <Text fontSize="12px" color={kotakColors.textMuted} margin="0">Propensity Score</Text>
                  <Text fontSize="12px" fontWeight={700} color={kotakColors.success} margin="0">0.87 (High)</Text>
                </FlexBox>
                <FlexBox justifyContent="space-between" style={{ marginBottom: '8px' }}>
                  <Text fontSize="12px" color={kotakColors.textMuted} margin="0">Top Signal</Text>
                  <Text fontSize="12px" color={kotakColors.gold} margin="0">NBFC EMI detected</Text>
                </FlexBox>
                <FlexBox justifyContent="space-between">
                  <Text fontSize="12px" color={kotakColors.textMuted} margin="0">Recommended Action</Text>
                  <Text fontSize="12px" color={kotakColors.primary} margin="0">Branch RM outreach</Text>
                </FlexBox>
              </Box>

              {/* Data Sources */}
              <Box style={{ marginTop: '16px', padding: '12px', background: `${kotakColors.gold}10`, borderRadius: '8px' }}>
                <Text fontSize="12px" fontWeight={600} color={kotakColors.gold} margin="0 0 4px 0">
                  DATA SOURCES
                </Text>
                <Text fontSize="12px" color={kotakColors.textMuted} margin="0">
                  Account transactions • UPI patterns • Loan payments • Credit bureau • App behavior
                </Text>
              </Box>
            </Box>
          </MotionBox>
        </FlexBox>

        {/* Bottom Stats */}
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
              <Text fontSize="20px" fontWeight={700} color={kotakColors.gold} margin="0">Rs 5 Cr</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Investment</Text>
            </Box>
            <Box style={{ width: '1px', height: '35px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="20px" fontWeight={700} color={kotakColors.success} margin="0">8 Months</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Development</Text>
            </Box>
            <Box style={{ width: '1px', height: '35px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="20px" fontWeight={700} color={kotakColors.primary} margin="0">3.5x ROI</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Expected Return</Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Predictive model uses 5 signal categories to score propensity.
        16x conversion lift vs random targeting, 72% AUC score.
        Identifies existing Kotak customers likely to have NBFC gold loans.
        Investment: Rs 5 Cr, 8 months development, 3.5x ROI.
      </Notes>
    </Slide>
  );
};

export default PredictiveModelSlide;
