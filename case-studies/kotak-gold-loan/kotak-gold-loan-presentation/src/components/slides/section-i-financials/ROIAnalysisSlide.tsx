import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { roiAnalysis } from '../../../data/financialData';

const MotionBox = motion(Box);

const sensitivityScenarios = [
  { scenario: 'Conservative', customers: '340K', aum: 'Rs 6,800 Cr', pat: 'Rs 290 Cr', roi: '1.96x' },
  { scenario: 'Base Case', customers: '425K', aum: 'Rs 8,500 Cr', pat: 'Rs 363 Cr', roi: '2.45x' },
  { scenario: 'Optimistic', customers: '510K', aum: 'Rs 10,200 Cr', pat: 'Rs 435 Cr', roi: '2.94x' },
];

export const ROIAnalysisSlide: React.FC = () => {
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
            color={kotakColors.success}
            margin="0 0 8px 0"
            style={{ letterSpacing: '2px', textTransform: 'uppercase' }}
          >
            RETURN ON INVESTMENT
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Investment Returns & Sensitivity
          </Heading>
        </MotionBox>

        <FlexBox flex={1} style={{ gap: '24px' }}>
          {/* Left: ROI Metrics */}
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
                Key ROI Metrics
              </Text>

              <FlexBox flexDirection="column" style={{ gap: '16px' }}>
                {[
                  { label: 'Total Investment', value: roiAnalysis.totalInvestment, sublabel: 'Over 3 years', color: kotakColors.primary, icon: '💰' },
                  { label: 'Total PAT Returns', value: roiAnalysis.totalPAT, sublabel: 'Cumulative profit', color: kotakColors.success, icon: '📈' },
                  { label: 'ROI Multiple', value: roiAnalysis.roi, sublabel: 'Return on investment', color: kotakColors.gold, icon: '🎯' },
                  { label: 'Payback Period', value: roiAnalysis.paybackPeriod, sublabel: 'Time to break even', color: kotakColors.primary, icon: '⏱️' },
                  { label: 'IRR', value: roiAnalysis.irr, sublabel: 'Internal rate of return', color: kotakColors.success, icon: '📊' },
                ].map((item, index) => (
                  <Box
                    key={index}
                    style={{
                      background: 'rgba(255,255,255,0.03)',
                      borderRadius: '10px',
                      padding: '16px',
                      borderLeft: `3px solid ${item.color}`,
                    }}
                  >
                    <FlexBox alignItems="center" justifyContent="space-between">
                      <FlexBox alignItems="center" style={{ gap: '12px' }}>
                        <Text fontSize="24px" margin="0">{item.icon}</Text>
                        <Box>
                          <Text fontSize="13px" color={kotakColors.textMuted} margin="0">{item.label}</Text>
                          <Text fontSize="11px" color={kotakColors.textMuted} margin="0">{item.sublabel}</Text>
                        </Box>
                      </FlexBox>
                      <Text fontSize="24px" fontWeight={700} color={item.color} margin="0">
                        {item.value}
                      </Text>
                    </FlexBox>
                  </Box>
                ))}
              </FlexBox>
            </Box>
          </MotionBox>

          {/* Right: Sensitivity Analysis */}
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
                Sensitivity Analysis (±20% Variance)
              </Text>

              {/* Table Header */}
              <FlexBox style={{ padding: '10px 14px', background: kotakColors.primary, borderRadius: '8px 8px 0 0' }}>
                <Box style={{ flex: 1 }}><Text fontSize="11px" fontWeight={600} color={kotakColors.white} margin="0">Scenario</Text></Box>
                <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="11px" fontWeight={600} color={kotakColors.white} margin="0">Customers</Text></Box>
                <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="11px" fontWeight={600} color={kotakColors.white} margin="0">AUM</Text></Box>
                <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="11px" fontWeight={600} color={kotakColors.white} margin="0">PAT</Text></Box>
                <Box style={{ flex: 0.8, textAlign: 'right' }}><Text fontSize="11px" fontWeight={600} color={kotakColors.white} margin="0">ROI</Text></Box>
              </FlexBox>

              {/* Rows */}
              {sensitivityScenarios.map((row, index) => (
                <FlexBox
                  key={index}
                  style={{
                    padding: '14px',
                    background: index === 1 ? `${kotakColors.success}10` : 'rgba(255,255,255,0.02)',
                    border: index === 1 ? `1px solid ${kotakColors.success}30` : 'none',
                    borderBottom: '1px solid rgba(255,255,255,0.05)',
                  }}
                  alignItems="center"
                >
                  <Box style={{ flex: 1 }}>
                    <Text fontSize="13px" fontWeight={index === 1 ? 700 : 500} color={kotakColors.white} margin="0">
                      {row.scenario}
                    </Text>
                  </Box>
                  <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="13px" color={kotakColors.textMuted} margin="0">{row.customers}</Text></Box>
                  <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="13px" color={kotakColors.textMuted} margin="0">{row.aum}</Text></Box>
                  <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="13px" fontWeight={600} color={kotakColors.success} margin="0">{row.pat}</Text></Box>
                  <Box style={{ flex: 0.8, textAlign: 'right' }}>
                    <Text fontSize="15px" fontWeight={700} color={kotakColors.gold} margin="0">{row.roi}</Text>
                  </Box>
                </FlexBox>
              ))}

              {/* Risk Note */}
              <Box style={{ marginTop: '16px', padding: '14px', background: `${kotakColors.gold}10`, borderRadius: '10px' }}>
                <Text fontSize="13px" color={kotakColors.textMuted} margin="0">
                  <Text fontSize="13px" fontWeight={600} color={kotakColors.gold} style={{ display: 'inline' }}>Even in conservative scenario:</Text>{' '}
                  ROI of 1.96x with Rs 290 Cr PAT still exceeds hurdle rate.
                </Text>
              </Box>
            </Box>
          </MotionBox>
        </FlexBox>
      </FlexBox>

      <Notes>
        Base case: Rs 148 Cr investment generates Rs 363 Cr PAT = 2.45x ROI.
        Payback in ~20 months, 75% IRR. Even conservative scenario delivers 1.96x ROI.
        Risk-adjusted returns are attractive across all scenarios.
      </Notes>
    </Slide>
  );
};

export default ROIAnalysisSlide;
