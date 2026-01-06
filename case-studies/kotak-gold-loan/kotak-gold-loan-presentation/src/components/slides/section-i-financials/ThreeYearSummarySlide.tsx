import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { YearCards } from '../../charts/StackedBarChart';
import { threeYearProjections } from '../../../data/financialData';

const MotionBox = motion(Box);

export const ThreeYearSummarySlide: React.FC = () => {
  const summaryMetrics = [
    { label: 'Total Customers', value: '425K', color: kotakColors.primary },
    { label: 'Total AUM', value: 'Rs 8,500 Cr', color: kotakColors.gold },
    { label: 'Cumulative PAT', value: 'Rs 363 Cr', color: kotakColors.success },
  ];

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
            FINANCIAL PROJECTIONS
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            3-Year Financial Summary
          </Heading>
        </MotionBox>

        {/* Summary Metrics Row */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.2 }}
          style={{ marginBottom: '24px' }}
        >
          <FlexBox style={{ gap: '20px' }}>
            {summaryMetrics.map((metric, index) => (
              <Box
                key={index}
                style={{
                  flex: 1,
                  background: `linear-gradient(135deg, ${metric.color}20, ${metric.color}05)`,
                  borderRadius: '16px',
                  padding: '24px',
                  textAlign: 'center',
                  border: `2px solid ${metric.color}40`,
                }}
              >
                <Text fontSize="40px" fontWeight={700} color={metric.color} margin="0">
                  {metric.value}
                </Text>
                <Text fontSize="16px" color={kotakColors.textMuted} margin="8px 0 0 0">
                  {metric.label}
                </Text>
              </Box>
            ))}
          </FlexBox>
        </MotionBox>

        {/* Year Cards */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.3 }}
          style={{ flex: 1 }}
        >
          <YearCards data={threeYearProjections.years} />
        </MotionBox>

        {/* Growth Trajectory */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.6 }}
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
              <Text fontSize="18px" fontWeight={700} color={kotakColors.primary} margin="0">2.7x</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Customer Growth (Y1→Y3)</Text>
            </Box>
            <Box style={{ width: '1px', height: '35px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="18px" fontWeight={700} color={kotakColors.gold} margin="0">3.4x</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">AUM Growth (Y1→Y3)</Text>
            </Box>
            <Box style={{ width: '1px', height: '35px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="18px" fontWeight={700} color={kotakColors.success} margin="0">3.5x</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">PAT Growth (Y1→Y3)</Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        3-year summary: 425K customers, Rs 8,500 Cr AUM, Rs 363 Cr cumulative PAT.
        Strong growth trajectory: 2.7x customer growth, 3.4x AUM growth, 3.5x PAT growth.
        Year 3 delivers the majority of returns as the portfolio matures.
      </Notes>
    </Slide>
  );
};

export default ThreeYearSummarySlide;
