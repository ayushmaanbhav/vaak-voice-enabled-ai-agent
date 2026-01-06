import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { customerSegments } from '../../../data/segmentData';

const MotionBox = motion(Box);

const p2 = customerSegments[1]; // P2: Trust-Seekers

const motivationDrivers = [
  { driver: 'Recent NBFC negative news', score: 9, icon: '📰' },
  { driver: 'Gold security concerns', score: 9, icon: '🔒' },
  { driver: 'Bank credibility preference', score: 8, icon: '🏦' },
  { driver: 'Family pressure for safety', score: 7, icon: '👨‍👩‍👧' },
];

const targetProfile = [
  { label: 'Age Group', value: '35-55 years' },
  { label: 'Gold Holdings', value: 'Rs 3-10 Lakh' },
  { label: 'Decision Style', value: 'Risk-averse, family-influenced' },
  { label: 'Information Source', value: 'News, family, local advisors' },
];

export const P2TrustSeekersSlide: React.FC = () => {
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
            color={p2.color}
            margin="0 0 8px 0"
            style={{ letterSpacing: '2px', textTransform: 'uppercase' }}
          >
            SEGMENT DEEP DIVE: {p2.id}
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            {p2.name}: Safety Over Savings
          </Heading>
        </MotionBox>

        <FlexBox flex={1} style={{ gap: '24px' }}>
          {/* Left: Profile & Characteristics */}
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
                borderTop: `4px solid ${p2.color}`,
              }}
            >
              {/* Profile Header */}
              <FlexBox alignItems="center" style={{ gap: '16px', marginBottom: '20px' }}>
                <Box
                  style={{
                    width: '64px',
                    height: '64px',
                    borderRadius: '16px',
                    background: `${p2.color}20`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  <Text fontSize="32px" margin="0">{p2.icon}</Text>
                </Box>
                <Box>
                  <Text fontSize="20px" fontWeight={700} color={kotakColors.white} margin="0">
                    {p2.profile}
                  </Text>
                  <Text fontSize="14px" color={kotakColors.textMuted} margin="4px 0 0 0">
                    Loan Range: {p2.loanRange}
                  </Text>
                </Box>
              </FlexBox>

              {/* Target Profile */}
              <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0 0 12px 0">
                TARGET PROFILE
              </Text>
              <FlexBox flexDirection="column" style={{ gap: '8px', marginBottom: '20px' }}>
                {targetProfile.map((item, i) => (
                  <FlexBox key={i} justifyContent="space-between" style={{ padding: '10px 12px', background: 'rgba(255,255,255,0.03)', borderRadius: '6px' }}>
                    <Text fontSize="14px" color={kotakColors.textMuted} margin="0">{item.label}</Text>
                    <Text fontSize="14px" fontWeight={500} color={kotakColors.white} margin="0">{item.value}</Text>
                  </FlexBox>
                ))}
              </FlexBox>

              {/* Key Metrics */}
              <FlexBox style={{ gap: '12px' }}>
                <Box style={{ flex: 1, background: 'rgba(255,255,255,0.03)', borderRadius: '10px', padding: '14px', textAlign: 'center' }}>
                  <Text fontSize="24px" fontWeight={700} color={p2.color} margin="0">{p2.conversionRate}</Text>
                  <Text fontSize="12px" color={kotakColors.textMuted} margin="4px 0 0 0">Conversion Rate</Text>
                </Box>
                <Box style={{ flex: 1, background: 'rgba(255,255,255,0.03)', borderRadius: '10px', padding: '14px', textAlign: 'center' }}>
                  <Text fontSize="24px" fontWeight={700} color={kotakColors.gold} margin="0">Rs 1,800</Text>
                  <Text fontSize="12px" color={kotakColors.textMuted} margin="4px 0 0 0">Target CAC</Text>
                </Box>
              </FlexBox>
            </Box>
          </MotionBox>

          {/* Right: Motivation Drivers */}
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
                Switching Motivation Drivers
              </Text>

              <FlexBox flexDirection="column" style={{ gap: '12px' }}>
                {motivationDrivers.map((item, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, x: 20 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.4, delay: 0.4 + index * 0.1 }}
                    style={{
                      background: 'rgba(255,255,255,0.03)',
                      borderRadius: '10px',
                      padding: '16px',
                    }}
                  >
                    <FlexBox alignItems="center" justifyContent="space-between">
                      <FlexBox alignItems="center" style={{ gap: '12px' }}>
                        <Text fontSize="24px" margin="0">{item.icon}</Text>
                        <Text fontSize="16px" fontWeight={500} color={kotakColors.white} margin="0">
                          {item.driver}
                        </Text>
                      </FlexBox>
                      <Box style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <Box style={{ width: '100px', height: '8px', background: 'rgba(255,255,255,0.1)', borderRadius: '4px', overflow: 'hidden' }}>
                          <Box style={{ width: `${item.score * 10}%`, height: '100%', background: p2.color, borderRadius: '4px' }} />
                        </Box>
                        <Text fontSize="14px" fontWeight={700} color={p2.color} margin="0">
                          {item.score}/10
                        </Text>
                      </Box>
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>

              {/* Key Messaging */}
              <Box style={{ marginTop: '20px', padding: '16px', background: `${p2.color}15`, borderRadius: '10px' }}>
                <Text fontSize="14px" fontWeight={600} color={p2.color} margin="0 0 8px 0">
                  KEY MESSAGING STRATEGY
                </Text>
                <Text fontSize="15px" color={kotakColors.white} margin="0" style={{ lineHeight: 1.5 }}>
                  Lead with <Text fontSize="15px" fontWeight={700} color={p2.color} style={{ display: 'inline' }}>bank security</Text>, not interest rates.
                  Emphasize vault infrastructure, insurance coverage, and RBI oversight.
                </Text>
              </Box>

              {/* Campaign Tagline */}
              <Box style={{ marginTop: '16px', padding: '14px', background: kotakColors.primary, borderRadius: '10px', textAlign: 'center' }}>
                <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0">
                  "Your Gold Deserves a Bank"
                </Text>
              </Box>
            </Box>
          </MotionBox>
        </FlexBox>
      </FlexBox>

      <Notes>
        P2 Trust-Seekers prioritize safety over savings. They're driven by recent NBFC news,
        security concerns, and family pressure. Best conversion: 10-15% with trust-first messaging.
        Campaign: "Your Gold Deserves a Bank" - lead with security, vault infrastructure, RBI oversight.
      </Notes>
    </Slide>
  );
};

export default P2TrustSeekersSlide;
