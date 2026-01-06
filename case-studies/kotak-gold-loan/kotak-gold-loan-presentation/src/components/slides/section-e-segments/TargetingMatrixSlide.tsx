import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { customerSegments } from '../../../data/segmentData';

const MotionBox = motion(Box);

const matrixData = [
  {
    segment: 'P1',
    name: 'High-Value',
    channels: ['Business Banking', 'CA Network', 'Trade Associations'],
    cac: 'Rs 2,500',
    message: 'Save Rs 1L+ annually on working capital',
    color: customerSegments[0].color,
  },
  {
    segment: 'P2',
    name: 'Trust-Seekers',
    channels: ['Branch Referrals', 'News/PR', 'Local Radio'],
    cac: 'Rs 1,800',
    message: 'Your gold deserves bank-grade security',
    color: customerSegments[1].color,
  },
  {
    segment: 'P3',
    name: 'Women (Shakti)',
    channels: ['SHG Partners', 'Women Associations', 'Shakti Events'],
    cac: 'Rs 1,500',
    message: 'Empowering your business dreams',
    color: customerSegments[2].color,
  },
  {
    segment: 'P4',
    name: 'Digital Native',
    channels: ['811 App', 'Digital Ads', 'Influencers'],
    cac: 'Rs 800',
    message: 'Gold loan in 3 clicks, doorstep pickup',
    color: customerSegments[3].color,
  },
];

export const TargetingMatrixSlide: React.FC = () => {
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
            TARGETING STRATEGY
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Segment Targeting Matrix
          </Heading>
        </MotionBox>

        {/* Matrix Table */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.2 }}
          style={{ flex: 1 }}
        >
          <Box
            style={{
              background: kotakColors.darkCard,
              borderRadius: '16px',
              overflow: 'hidden',
            }}
          >
            {/* Header */}
            <FlexBox
              style={{
                background: kotakColors.primary,
                padding: '16px 20px',
              }}
            >
              <Box style={{ width: '100px' }}>
                <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0">Segment</Text>
              </Box>
              <Box style={{ flex: 1.5 }}>
                <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0">Channels</Text>
              </Box>
              <Box style={{ width: '100px', textAlign: 'center' }}>
                <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0">Target CAC</Text>
              </Box>
              <Box style={{ flex: 2 }}>
                <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0">Key Message</Text>
              </Box>
            </FlexBox>

            {/* Rows */}
            {matrixData.map((row, index) => (
              <MotionBox
                key={index}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.4, delay: 0.3 + index * 0.1 }}
              >
                <FlexBox
                  style={{
                    padding: '18px 20px',
                    borderBottom: index < matrixData.length - 1 ? '1px solid rgba(255,255,255,0.05)' : 'none',
                    background: index % 2 === 1 ? 'rgba(255,255,255,0.02)' : 'transparent',
                  }}
                  alignItems="center"
                >
                  {/* Segment */}
                  <Box style={{ width: '100px' }}>
                    <FlexBox alignItems="center" style={{ gap: '8px' }}>
                      <Box
                        style={{
                          width: '32px',
                          height: '32px',
                          borderRadius: '8px',
                          background: `${row.color}20`,
                          border: `2px solid ${row.color}`,
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                        }}
                      >
                        <Text fontSize="12px" fontWeight={700} color={row.color} margin="0">{row.segment}</Text>
                      </Box>
                      <Text fontSize="14px" fontWeight={500} color={kotakColors.white} margin="0">{row.name}</Text>
                    </FlexBox>
                  </Box>

                  {/* Channels */}
                  <Box style={{ flex: 1.5 }}>
                    <FlexBox style={{ gap: '6px' }} flexWrap="wrap">
                      {row.channels.map((ch, i) => (
                        <Box
                          key={i}
                          style={{
                            padding: '4px 10px',
                            background: 'rgba(255,255,255,0.05)',
                            borderRadius: '4px',
                          }}
                        >
                          <Text fontSize="12px" color={kotakColors.textMuted} margin="0">{ch}</Text>
                        </Box>
                      ))}
                    </FlexBox>
                  </Box>

                  {/* CAC */}
                  <Box style={{ width: '100px', textAlign: 'center' }}>
                    <Text fontSize="16px" fontWeight={700} color={row.color} margin="0">{row.cac}</Text>
                  </Box>

                  {/* Message */}
                  <Box style={{ flex: 2 }}>
                    <Text fontSize="14px" color={kotakColors.white} margin="0" style={{ fontStyle: 'italic' }}>
                      "{row.message}"
                    </Text>
                  </Box>
                </FlexBox>
              </MotionBox>
            ))}
          </Box>
        </MotionBox>

        {/* Bottom Summary */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.7 }}
          style={{ marginTop: '20px' }}
        >
          <FlexBox
            style={{
              background: `linear-gradient(90deg, ${kotakColors.gold}15, ${kotakColors.gold}05)`,
              borderRadius: '12px',
              padding: '16px 24px',
            }}
            justifyContent="space-around"
            alignItems="center"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.gold} margin="0">
                Rs 1,650
              </Text>
              <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">
                Blended CAC Target
              </Text>
            </Box>
            <Box style={{ width: '1px', height: '40px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.success} margin="0">
                4+ Years
              </Text>
              <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">
                Avg Customer Lifetime
              </Text>
            </Box>
            <Box style={{ width: '1px', height: '40px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.primary} margin="0">
                Rs 8,000+
              </Text>
              <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">
                LTV per Customer
              </Text>
            </Box>
            <Box style={{ width: '1px', height: '40px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.gold} margin="0">
                4.8x
              </Text>
              <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">
                LTV:CAC Ratio
              </Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        The targeting matrix shows channels, CAC, and messaging by segment.
        Blended CAC target is Rs 1,650 with 4+ year average lifetime.
        LTV:CAC ratio of 4.8x shows strong unit economics across all segments.
      </Notes>
    </Slide>
  );
};

export default TargetingMatrixSlide;
