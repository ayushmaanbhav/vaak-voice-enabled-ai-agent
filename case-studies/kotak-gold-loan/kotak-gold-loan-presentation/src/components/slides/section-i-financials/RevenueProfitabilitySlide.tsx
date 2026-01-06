import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { threeYearProjections } from '../../../data/financialData';

const MotionBox = motion(Box);

// Prepare chart data
const chartData = threeYearProjections.years.map(y => ({
  year: `Y${y.year}`,
  NII: y.nii,
  PPOP: y.ppop,
  PAT: y.pat,
}));

export const RevenueProfitabilitySlide: React.FC = () => {
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
            REVENUE & PROFITABILITY
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Income & Profit Trajectory
          </Heading>
        </MotionBox>

        {/* Chart Area - Visual bars showing growth */}
        <MotionBox
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
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
              3-Year Growth Trajectory (Rs Cr)
            </Text>

            <FlexBox style={{ gap: '24px', height: '200px' }} alignItems="flex-end">
              {chartData.map((item, index) => (
                <MotionBox
                  key={index}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.4, delay: 0.3 + index * 0.1 }}
                  style={{ flex: 1, textAlign: 'center' }}
                >
                  <FlexBox flexDirection="column" style={{ gap: '4px' }} alignItems="center">
                    {/* NII Bar */}
                    <Box
                      style={{
                        width: '50px',
                        height: `${(item.NII / 500) * 150}px`,
                        background: kotakColors.gold,
                        borderRadius: '4px 4px 0 0',
                      }}
                    />
                    <Text fontSize="12px" color={kotakColors.gold} margin="0">NII: {item.NII}</Text>
                  </FlexBox>
                  <FlexBox flexDirection="column" style={{ gap: '4px', marginTop: '8px' }} alignItems="center">
                    {/* PAT Bar */}
                    <Box
                      style={{
                        width: '50px',
                        height: `${(item.PAT / 500) * 150}px`,
                        background: kotakColors.success,
                        borderRadius: '4px 4px 0 0',
                      }}
                    />
                    <Text fontSize="12px" color={kotakColors.success} margin="0">PAT: {item.PAT}</Text>
                  </FlexBox>
                  <Text fontSize="16px" fontWeight={700} color={kotakColors.white} margin="12px 0 0 0">
                    {item.year}
                  </Text>
                </MotionBox>
              ))}
            </FlexBox>
          </Box>
        </MotionBox>

        {/* Bottom Metrics */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.5 }}
          style={{ marginTop: '20px' }}
        >
          <FlexBox style={{ gap: '16px' }}>
            {threeYearProjections.years.map((year, index) => (
              <Box
                key={index}
                style={{
                  flex: 1,
                  background: kotakColors.darkCard,
                  borderRadius: '12px',
                  padding: '16px',
                  borderTop: `3px solid ${index === 0 ? kotakColors.primary : index === 1 ? kotakColors.gold : kotakColors.success}`,
                }}
              >
                <Text fontSize="14px" fontWeight={700} color={kotakColors.white} margin="0 0 12px 0">
                  Year {year.year}
                </Text>
                <FlexBox flexDirection="column" style={{ gap: '8px' }}>
                  <FlexBox justifyContent="space-between">
                    <Text fontSize="12px" color={kotakColors.textMuted} margin="0">NII</Text>
                    <Text fontSize="13px" fontWeight={600} color={kotakColors.gold} margin="0">Rs {year.nii} Cr</Text>
                  </FlexBox>
                  <FlexBox justifyContent="space-between">
                    <Text fontSize="12px" color={kotakColors.textMuted} margin="0">PPOP</Text>
                    <Text fontSize="13px" fontWeight={600} color={kotakColors.primary} margin="0">Rs {year.ppop} Cr</Text>
                  </FlexBox>
                  <FlexBox justifyContent="space-between">
                    <Text fontSize="12px" color={kotakColors.textMuted} margin="0">PAT</Text>
                    <Text fontSize="13px" fontWeight={600} color={kotakColors.success} margin="0">Rs {year.pat} Cr</Text>
                  </FlexBox>
                </FlexBox>
              </Box>
            ))}
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Revenue grows from Rs 87 Cr NII in Y1 to Rs 480 Cr in Y3.
        PAT grows from Rs 23 Cr to Rs 241 Cr.
        Strong operating leverage as portfolio scales.
      </Notes>
    </Slide>
  );
};

export default RevenueProfitabilitySlide;
