import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { customerSegments } from '../../../data/segmentData';

const MotionBox = motion(Box);

const p3 = customerSegments[2]; // P3: Women Segment
const p4 = customerSegments[3]; // P4: Young Digital

const p3Features = [
  { feature: 'Female relationship managers', icon: '👩‍💼' },
  { feature: 'Private consultation rooms', icon: '🚪' },
  { feature: 'Shakti Gold Loan branding', icon: '💪' },
  { feature: 'SHG partnerships', icon: '🤝' },
];

const p4Features = [
  { feature: '100% digital journey', icon: '📱' },
  { feature: '811 account integration', icon: '🔗' },
  { feature: 'AI-powered pre-approval', icon: '🤖' },
  { feature: 'Same-day doorstep pickup', icon: '🏠' },
];

export const P3P4OverviewSlide: React.FC = () => {
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
            GROWTH SEGMENTS: P3 & P4
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Future Growth: Women & Digital Native
          </Heading>
        </MotionBox>

        <FlexBox flex={1} style={{ gap: '24px' }}>
          {/* P3: Women Segment */}
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
                borderTop: `4px solid ${p3.color}`,
              }}
            >
              {/* Header */}
              <FlexBox alignItems="center" style={{ gap: '16px', marginBottom: '16px' }}>
                <Box
                  style={{
                    width: '56px',
                    height: '56px',
                    borderRadius: '14px',
                    background: `${p3.color}20`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  <Text fontSize="28px" margin="0">{p3.icon}</Text>
                </Box>
                <Box>
                  <FlexBox alignItems="center" style={{ gap: '8px' }}>
                    <Text fontSize="12px" fontWeight={700} color={p3.color} margin="0">{p3.id}</Text>
                    <Text fontSize="18px" fontWeight={700} color={kotakColors.white} margin="0">{p3.name}</Text>
                  </FlexBox>
                  <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">{p3.profile}</Text>
                </Box>
              </FlexBox>

              {/* Metrics */}
              <FlexBox style={{ gap: '10px', marginBottom: '16px' }}>
                <Box style={{ flex: 1, background: 'rgba(255,255,255,0.03)', borderRadius: '8px', padding: '12px', textAlign: 'center' }}>
                  <Text fontSize="20px" fontWeight={700} color={p3.color} margin="0">{p3.conversionRate}</Text>
                  <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Conversion</Text>
                </Box>
                <Box style={{ flex: 1, background: 'rgba(255,255,255,0.03)', borderRadius: '8px', padding: '12px', textAlign: 'center' }}>
                  <Text fontSize="20px" fontWeight={700} color={kotakColors.white} margin="0">{p3.loanRange}</Text>
                  <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Loan Range</Text>
                </Box>
              </FlexBox>

              {/* Key Features */}
              <Text fontSize="13px" fontWeight={600} color={kotakColors.white} margin="0 0 10px 0">
                SHAKTI GOLD LOAN FEATURES
              </Text>
              <FlexBox flexDirection="column" style={{ gap: '8px' }}>
                {p3Features.map((item, i) => (
                  <FlexBox key={i} alignItems="center" style={{ gap: '10px', padding: '10px 12px', background: 'rgba(255,255,255,0.03)', borderRadius: '6px' }}>
                    <Text fontSize="18px" margin="0">{item.icon}</Text>
                    <Text fontSize="14px" color={kotakColors.white} margin="0">{item.feature}</Text>
                  </FlexBox>
                ))}
              </FlexBox>

              {/* Tagline */}
              <Box style={{ marginTop: '16px', padding: '12px', background: `${p3.color}20`, borderRadius: '8px', textAlign: 'center' }}>
                <Text fontSize="14px" fontWeight={600} color={p3.color} margin="0">
                  "Empowering Women Entrepreneurs"
                </Text>
              </Box>
            </Box>
          </MotionBox>

          {/* P4: Young Digital */}
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
                borderTop: `4px solid ${p4.color}`,
              }}
            >
              {/* Header */}
              <FlexBox alignItems="center" style={{ gap: '16px', marginBottom: '16px' }}>
                <Box
                  style={{
                    width: '56px',
                    height: '56px',
                    borderRadius: '14px',
                    background: `${p4.color}20`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  <Text fontSize="28px" margin="0">{p4.icon}</Text>
                </Box>
                <Box>
                  <FlexBox alignItems="center" style={{ gap: '8px' }}>
                    <Text fontSize="12px" fontWeight={700} color={p4.color} margin="0">{p4.id}</Text>
                    <Text fontSize="18px" fontWeight={700} color={kotakColors.white} margin="0">{p4.name}</Text>
                  </FlexBox>
                  <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">{p4.profile}</Text>
                </Box>
              </FlexBox>

              {/* Metrics */}
              <FlexBox style={{ gap: '10px', marginBottom: '16px' }}>
                <Box style={{ flex: 1, background: 'rgba(255,255,255,0.03)', borderRadius: '8px', padding: '12px', textAlign: 'center' }}>
                  <Text fontSize="20px" fontWeight={700} color={p4.color} margin="0">{p4.conversionRate}</Text>
                  <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Conversion</Text>
                </Box>
                <Box style={{ flex: 1, background: 'rgba(255,255,255,0.03)', borderRadius: '8px', padding: '12px', textAlign: 'center' }}>
                  <Text fontSize="20px" fontWeight={700} color={kotakColors.white} margin="0">{p4.loanRange}</Text>
                  <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Loan Range</Text>
                </Box>
              </FlexBox>

              {/* Key Features */}
              <Text fontSize="13px" fontWeight={600} color={kotakColors.white} margin="0 0 10px 0">
                811 GOLD LOAN FEATURES
              </Text>
              <FlexBox flexDirection="column" style={{ gap: '8px' }}>
                {p4Features.map((item, i) => (
                  <FlexBox key={i} alignItems="center" style={{ gap: '10px', padding: '10px 12px', background: 'rgba(255,255,255,0.03)', borderRadius: '6px' }}>
                    <Text fontSize="18px" margin="0">{item.icon}</Text>
                    <Text fontSize="14px" color={kotakColors.white} margin="0">{item.feature}</Text>
                  </FlexBox>
                ))}
              </FlexBox>

              {/* Tagline */}
              <Box style={{ marginTop: '16px', padding: '12px', background: `${p4.color}20`, borderRadius: '8px', textAlign: 'center' }}>
                <Text fontSize="14px" fontWeight={600} color={p4.color} margin="0">
                  "Gold Loan in 3 Clicks"
                </Text>
              </Box>
            </Box>
          </MotionBox>
        </FlexBox>

        {/* Timeline */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.6 }}
          style={{ marginTop: '20px' }}
        >
          <FlexBox
            style={{
              background: kotakColors.darkCard,
              borderRadius: '12px',
              padding: '16px 24px',
            }}
            justifyContent="space-around"
            alignItems="center"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">Year 1 Focus</Text>
              <Text fontSize="18px" fontWeight={700} color={kotakColors.primary} margin="0">P1 + P2</Text>
            </Box>
            <Text fontSize="24px" color={kotakColors.textMuted} margin="0">→</Text>
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">Year 2 Expansion</Text>
              <Text fontSize="18px" fontWeight={700} color={p3.color} margin="0">+ P3 (Shakti)</Text>
            </Box>
            <Text fontSize="24px" color={kotakColors.textMuted} margin="0">→</Text>
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">Year 3 Scale</Text>
              <Text fontSize="18px" fontWeight={700} color={p4.color} margin="0">+ P4 (Digital)</Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        P3 Women segment through Shakti branding - female RMs, private rooms, SHG partnerships.
        P4 Digital Native through 811 integration - 100% digital, AI pre-approval, doorstep service.
        Phased approach: P1/P2 Year 1, add P3 Year 2, scale P4 Year 3.
      </Notes>
    </Slide>
  );
};

export default P3P4OverviewSlide;
