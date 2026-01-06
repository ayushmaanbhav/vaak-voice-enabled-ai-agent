import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { SegmentDonut } from '../../charts/DonutChart';
import { customerSegments, segmentOverview } from '../../../data/segmentData';

// Prepare data for donut chart
const donutData = segmentOverview.segments.map(s => ({
  name: s.name,
  value: s.share,
  color: s.color,
  description: `${s.share}% potential`,
}));

const MotionBox = motion(Box);

export const SegmentOverviewSlide: React.FC = () => {
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
            CUSTOMER SEGMENTS
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 32px 0">
            Four Priority Segments
          </Heading>
        </MotionBox>

        {/* Main Content */}
        <FlexBox flex={1} style={{ gap: '40px' }}>
          {/* Left: Donut Chart */}
          <MotionBox
            initial={{ opacity: 0, scale: 0.9 }}
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
              <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0 0 16px 0" style={{ textAlign: 'center' }}>
                Segment Distribution by Potential
              </Text>
              <SegmentDonut data={donutData} height={280} />
            </Box>
          </MotionBox>

          {/* Right: Segment Cards */}
          <MotionBox
            initial={{ opacity: 0, x: 30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            style={{ flex: 1.2 }}
          >
            <FlexBox flexDirection="column" style={{ gap: '12px' }}>
              {customerSegments.map((segment, index) => (
                <MotionBox
                  key={index}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.4, delay: 0.4 + index * 0.1 }}
                  style={{
                    background: kotakColors.darkCard,
                    borderRadius: '12px',
                    padding: '16px 20px',
                    borderLeft: `4px solid ${segment.color}`,
                  }}
                >
                  <FlexBox alignItems="center" justifyContent="space-between">
                    <FlexBox alignItems="center" style={{ gap: '16px' }}>
                      <Box
                        style={{
                          width: '44px',
                          height: '44px',
                          borderRadius: '10px',
                          background: `${segment.color}20`,
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                        }}
                      >
                        <Text fontSize="22px" margin="0">{segment.icon}</Text>
                      </Box>
                      <Box>
                        <FlexBox alignItems="center" style={{ gap: '8px' }}>
                          <Text fontSize="12px" fontWeight={700} color={segment.color} margin="0">
                            {segment.id}
                          </Text>
                          <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0">
                            {segment.name}
                          </Text>
                        </FlexBox>
                        <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">
                          {segment.profile}
                        </Text>
                      </Box>
                    </FlexBox>
                    <FlexBox style={{ gap: '16px' }}>
                      <Box style={{ textAlign: 'center' }}>
                        <Text fontSize="18px" fontWeight={700} color={segment.color} margin="0">
                          {segment.conversionRate}
                        </Text>
                        <Text fontSize="11px" color={kotakColors.textMuted} margin="0">
                          Conversion
                        </Text>
                      </Box>
                      <Box style={{ textAlign: 'center' }}>
                        <Text fontSize="18px" fontWeight={700} color={kotakColors.white} margin="0">
                          {segment.loanRange}
                        </Text>
                        <Text fontSize="11px" color={kotakColors.textMuted} margin="0">
                          Loan Range
                        </Text>
                      </Box>
                    </FlexBox>
                  </FlexBox>
                </MotionBox>
              ))}
            </FlexBox>
          </MotionBox>
        </FlexBox>

        {/* Bottom Strategy */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.8 }}
          style={{ marginTop: '20px' }}
        >
          <FlexBox
            style={{
              background: kotakColors.darkCard,
              borderRadius: '12px',
              padding: '16px 24px',
            }}
            justifyContent="space-around"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="28px" fontWeight={700} color={kotakColors.primary} margin="0">
                50,000
              </Text>
              <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">
                Year 1 Target Customers
              </Text>
            </Box>
            <Box style={{ width: '1px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="28px" fontWeight={700} color={kotakColors.gold} margin="0">
                Rs 1,000 Cr
              </Text>
              <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">
                Year 1 AUM Target
              </Text>
            </Box>
            <Box style={{ width: '1px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="28px" fontWeight={700} color={kotakColors.success} margin="0">
                P1 + P2
              </Text>
              <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">
                Initial Focus Segments
              </Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        We've identified 4 priority segments. P1 (High-Value Switchers) and P2 (Trust-Seekers)
        are our initial focus with highest conversion potential. P3 (Women - Shakti) and
        P4 (Young Digital) are growth segments for Year 2+. Year 1 target: 50K customers, Rs 1000 Cr AUM.
      </Notes>
    </Slide>
  );
};

export default SegmentOverviewSlide;
