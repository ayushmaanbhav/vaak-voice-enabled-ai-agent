import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const pitchMatrix = [
  {
    segment: 'P1',
    name: 'High-Value',
    primaryPitch: 'Working capital savings',
    secondaryPitch: 'Business growth reinvestment',
    channel: 'RM Direct Call',
    timing: 'Post EMI date',
    color: kotakColors.primary,
  },
  {
    segment: 'P2',
    name: 'Trust-Seekers',
    primaryPitch: 'Bank security guarantee',
    secondaryPitch: 'RBI oversight messaging',
    channel: 'News-led outreach',
    timing: 'After NBFC news',
    color: kotakColors.gold,
  },
  {
    segment: 'P3',
    name: 'Women (Shakti)',
    primaryPitch: 'Women empowerment',
    secondaryPitch: 'Female RM comfort',
    channel: 'SHG partnerships',
    timing: 'Festival seasons',
    color: '#EC4899',
  },
  {
    segment: 'P4',
    name: 'Digital Native',
    primaryPitch: '3-click convenience',
    secondaryPitch: 'Doorstep service',
    channel: '811 App push',
    timing: 'App engagement peak',
    color: '#8B5CF6',
  },
];

export const PersonalizedPitchingSlide: React.FC = () => {
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
            AI SOLUTION #3
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Personalized Pitching Engine
          </Heading>
        </MotionBox>

        {/* Pitch Matrix Table */}
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
                padding: '14px 20px',
              }}
            >
              <Box style={{ width: '120px' }}><Text fontSize="13px" fontWeight={600} color={kotakColors.white} margin="0">Segment</Text></Box>
              <Box style={{ flex: 1 }}><Text fontSize="13px" fontWeight={600} color={kotakColors.white} margin="0">Primary Pitch</Text></Box>
              <Box style={{ flex: 1 }}><Text fontSize="13px" fontWeight={600} color={kotakColors.white} margin="0">Secondary Pitch</Text></Box>
              <Box style={{ flex: 0.8 }}><Text fontSize="13px" fontWeight={600} color={kotakColors.white} margin="0">Channel</Text></Box>
              <Box style={{ flex: 0.8 }}><Text fontSize="13px" fontWeight={600} color={kotakColors.white} margin="0">Optimal Timing</Text></Box>
            </FlexBox>

            {/* Rows */}
            {pitchMatrix.map((row, index) => (
              <MotionBox
                key={index}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.4, delay: 0.3 + index * 0.1 }}
              >
                <FlexBox
                  style={{
                    padding: '16px 20px',
                    borderBottom: index < pitchMatrix.length - 1 ? '1px solid rgba(255,255,255,0.05)' : 'none',
                    background: index % 2 === 1 ? 'rgba(255,255,255,0.02)' : 'transparent',
                  }}
                  alignItems="center"
                >
                  <Box style={{ width: '120px' }}>
                    <FlexBox alignItems="center" style={{ gap: '8px' }}>
                      <Box
                        style={{
                          width: '28px',
                          height: '28px',
                          borderRadius: '6px',
                          background: `${row.color}20`,
                          border: `2px solid ${row.color}`,
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                        }}
                      >
                        <Text fontSize="11px" fontWeight={700} color={row.color} margin="0">{row.segment}</Text>
                      </Box>
                      <Text fontSize="13px" fontWeight={500} color={kotakColors.white} margin="0">{row.name}</Text>
                    </FlexBox>
                  </Box>
                  <Box style={{ flex: 1 }}>
                    <Text fontSize="14px" fontWeight={500} color={kotakColors.white} margin="0">{row.primaryPitch}</Text>
                  </Box>
                  <Box style={{ flex: 1 }}>
                    <Text fontSize="14px" color={kotakColors.textMuted} margin="0">{row.secondaryPitch}</Text>
                  </Box>
                  <Box style={{ flex: 0.8 }}>
                    <Box style={{ padding: '4px 8px', background: 'rgba(255,255,255,0.05)', borderRadius: '4px', display: 'inline-block' }}>
                      <Text fontSize="12px" color={kotakColors.white} margin="0">{row.channel}</Text>
                    </Box>
                  </Box>
                  <Box style={{ flex: 0.8 }}>
                    <Text fontSize="13px" color={row.color} margin="0">{row.timing}</Text>
                  </Box>
                </FlexBox>
              </MotionBox>
            ))}
          </Box>
        </MotionBox>

        {/* How It Works */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.7 }}
          style={{ marginTop: '16px' }}
        >
          <FlexBox style={{ gap: '16px' }}>
            <Box style={{ flex: 1, background: kotakColors.darkCard, borderRadius: '12px', padding: '16px' }}>
              <FlexBox alignItems="center" style={{ gap: '10px', marginBottom: '8px' }}>
                <Text fontSize="22px" margin="0">🧠</Text>
                <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0">AI-Powered Selection</Text>
              </FlexBox>
              <Text fontSize="12px" color={kotakColors.textMuted} margin="0">
                ML model selects pitch, channel, and timing based on customer profile and behavioral signals
              </Text>
            </Box>
            <Box style={{ flex: 1, background: kotakColors.darkCard, borderRadius: '12px', padding: '16px' }}>
              <FlexBox alignItems="center" style={{ gap: '10px', marginBottom: '8px' }}>
                <Text fontSize="22px" margin="0">📊</Text>
                <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0">A/B Testing</Text>
              </FlexBox>
              <Text fontSize="12px" color={kotakColors.textMuted} margin="0">
                Continuous optimization through controlled experiments across segments and channels
              </Text>
            </Box>
            <Box style={{ flex: 1, background: kotakColors.darkCard, borderRadius: '12px', padding: '16px' }}>
              <FlexBox alignItems="center" style={{ gap: '10px', marginBottom: '8px' }}>
                <Text fontSize="22px" margin="0">🎯</Text>
                <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0">35% Lift</Text>
              </FlexBox>
              <Text fontSize="12px" color={kotakColors.textMuted} margin="0">
                Expected conversion lift vs generic messaging through personalization
              </Text>
            </Box>
          </FlexBox>
        </MotionBox>

        {/* Investment */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.9 }}
          style={{ marginTop: '16px' }}
        >
          <FlexBox
            style={{
              background: `${kotakColors.success}10`,
              borderRadius: '12px',
              padding: '12px 24px',
              border: `1px solid ${kotakColors.success}30`,
            }}
            justifyContent="space-around"
            alignItems="center"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="18px" fontWeight={700} color={kotakColors.success} margin="0">Rs 3.5 Cr</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Investment</Text>
            </Box>
            <Box style={{ width: '1px', height: '30px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="18px" fontWeight={700} color={kotakColors.gold} margin="0">6 Months</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Development</Text>
            </Box>
            <Box style={{ width: '1px', height: '30px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="18px" fontWeight={700} color={kotakColors.primary} margin="0">3.8x ROI</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Expected Return</Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Personalized pitching engine selects optimal message, channel, and timing per segment.
        AI-powered selection based on behavioral signals, continuous A/B testing.
        35% conversion lift vs generic messaging. Rs 3.5 Cr investment, 3.8x ROI.
      </Notes>
    </Slide>
  );
};

export default PersonalizedPitchingSlide;
