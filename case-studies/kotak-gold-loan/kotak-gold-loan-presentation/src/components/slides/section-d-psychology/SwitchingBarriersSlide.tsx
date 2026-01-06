import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { HorizontalBarChart } from '../../charts/HorizontalBarChart';
import { switchingBarriers } from '../../../data/customerQuotes';

const MotionBox = motion(Box);

const barriersData = switchingBarriers.map(b => ({
  name: b.barrier.length > 25 ? b.barrier.substring(0, 25) + '...' : b.barrier,
  value: b.impactScore,
  fill: b.impactScore >= 8 ? kotakColors.danger : b.impactScore >= 6 ? kotakColors.gold : kotakColors.success,
}));

export const SwitchingBarriersSlide: React.FC = () => {
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
            SWITCHING PSYCHOLOGY
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 8px 0">
            Why Customers Don't Switch
          </Heading>
          <Text fontSize="18px" color={kotakColors.textMuted} margin="0 0 32px 0">
            Understanding the psychological barriers to conversion
          </Text>
        </MotionBox>

        {/* Chart and Details */}
        <FlexBox flex={1} style={{ gap: '32px' }}>
          {/* Left: Bar Chart */}
          <MotionBox
            initial={{ opacity: 0, x: -30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
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
              <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0 0 20px 0">
                Barrier Impact Score (1-10)
              </Text>
              <HorizontalBarChart
                data={barriersData}
                height={280}
                showValues
                valueFormatter={(v) => `${v}/10`}
              />
            </Box>
          </MotionBox>

          {/* Right: Barrier Details */}
          <MotionBox
            initial={{ opacity: 0, x: 30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            style={{ flex: 1 }}
          >
            <FlexBox flexDirection="column" style={{ gap: '12px' }}>
              {switchingBarriers.map((barrier, index) => (
                <MotionBox
                  key={index}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.4, delay: 0.4 + index * 0.08 }}
                  style={{
                    background: kotakColors.darkCard,
                    borderRadius: '10px',
                    padding: '14px 18px',
                    borderLeft: `3px solid ${barrier.impactScore >= 8 ? kotakColors.danger : barrier.impactScore >= 6 ? kotakColors.gold : kotakColors.success}`,
                  }}
                >
                  <FlexBox justifyContent="space-between" alignItems="center">
                    <Box style={{ flex: 1 }}>
                      <Text fontSize="15px" fontWeight={600} color={kotakColors.white} margin="0 0 4px 0">
                        {barrier.barrier}
                      </Text>
                      <Text fontSize="13px" color={kotakColors.textMuted} margin="0">
                        Solution: {barrier.mitigation}
                      </Text>
                    </Box>
                    <Box
                      style={{
                        width: '40px',
                        height: '40px',
                        borderRadius: '50%',
                        background: `${barrier.impactScore >= 8 ? kotakColors.danger : barrier.impactScore >= 6 ? kotakColors.gold : kotakColors.success}20`,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        flexShrink: 0,
                        marginLeft: '12px',
                      }}
                    >
                      <Text fontSize="14px" fontWeight={700} color={barrier.impactScore >= 8 ? kotakColors.danger : barrier.impactScore >= 6 ? kotakColors.gold : kotakColors.success} margin="0">
                        {barrier.impactScore}
                      </Text>
                    </Box>
                  </FlexBox>
                </MotionBox>
              ))}
            </FlexBox>
          </MotionBox>
        </FlexBox>

        {/* Bottom Insight */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.8 }}
          style={{ marginTop: '20px' }}
        >
          <Box
            style={{
              background: `linear-gradient(90deg, ${kotakColors.primary}15, transparent)`,
              borderRadius: '12px',
              padding: '16px 24px',
              border: `1px solid ${kotakColors.primary}30`,
            }}
          >
            <FlexBox alignItems="center" style={{ gap: '12px' }}>
              <Text fontSize="24px" margin="0">💡</Text>
              <Text fontSize="16px" color={kotakColors.white} margin="0">
                <Text fontSize="16px" fontWeight={700} color={kotakColors.primary} style={{ display: 'inline' }}>
                  Key Insight:
                </Text>
                {' '}The #1 barrier is cash flow fear during transition — this directly shapes our Bridge Loan solution.
              </Text>
            </FlexBox>
          </Box>
        </MotionBox>
      </FlexBox>

      <Notes>
        Understanding psychology is key to conversion. The biggest barrier is cash flow fear -
        customers worry about the 10-14 day gap between paying off old loan and getting new one.
        Our Bridge Loan directly addresses this. Other barriers like documentation hassle and
        trust in new provider also need systematic solutions.
      </Notes>
    </Slide>
  );
};

export default SwitchingBarriersSlide;
