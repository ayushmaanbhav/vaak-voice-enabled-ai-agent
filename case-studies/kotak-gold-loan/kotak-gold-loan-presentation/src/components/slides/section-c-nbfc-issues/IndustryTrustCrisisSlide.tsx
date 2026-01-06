import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const headlines = [
  {
    source: 'RBI Press Release',
    headline: 'RBI Bars IIFL Finance from Gold Loan Business',
    date: 'March 2024',
    icon: '🚫',
    color: kotakColors.danger,
    impact: '5.5M+ customers affected',
  },
  {
    source: 'Economic Times',
    headline: 'Manappuram AGM Arrested in Rs 20 Crore Tech Fraud',
    date: 'July 2024',
    icon: '👮',
    color: kotakColors.warning,
    impact: 'Management trust shaken',
  },
  {
    source: 'Business Standard',
    headline: 'Multiple RBI Penalties Against Gold NBFCs in 2023-24',
    date: 'Ongoing',
    icon: '💰',
    color: kotakColors.gold,
    impact: 'Regulatory scrutiny intensifies',
  },
  {
    source: 'TOI',
    headline: 'Fake Gold Fraud Cases Plague NBFC Branches',
    date: '2024',
    icon: '⚠️',
    color: kotakColors.danger,
    impact: 'Security concerns grow',
  },
];

export const IndustryTrustCrisisSlide: React.FC = () => {
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
            color={kotakColors.danger}
            margin="0 0 8px 0"
            style={{ letterSpacing: '2px', textTransform: 'uppercase' }}
          >
            INDUSTRY CRISIS
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 32px 0">
            The NBFC Trust Crisis
          </Heading>
        </MotionBox>

        {/* News Headlines */}
        <FlexBox flex={1} flexDirection="column" style={{ gap: '16px' }}>
          {headlines.map((item, index) => (
            <MotionBox
              key={index}
              initial={{ opacity: 0, x: -40 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.5, delay: 0.2 + index * 0.15 }}
              style={{
                background: kotakColors.darkCard,
                borderRadius: '12px',
                padding: '20px 24px',
                borderLeft: `4px solid ${item.color}`,
              }}
            >
              <FlexBox alignItems="center" style={{ gap: '20px' }}>
                <Box
                  style={{
                    width: '56px',
                    height: '56px',
                    borderRadius: '12px',
                    background: `${item.color}15`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    flexShrink: 0,
                  }}
                >
                  <Text fontSize="28px" margin="0">{item.icon}</Text>
                </Box>
                <Box style={{ flex: 1 }}>
                  <FlexBox style={{ gap: '12px', marginBottom: '4px' }} alignItems="center">
                    <Text fontSize="12px" fontWeight={600} color={item.color} margin="0">
                      {item.source}
                    </Text>
                    <Text fontSize="12px" color={kotakColors.textMuted} margin="0">
                      {item.date}
                    </Text>
                  </FlexBox>
                  <Text fontSize="18px" fontWeight={600} color={kotakColors.white} margin="0 0 4px 0">
                    {item.headline}
                  </Text>
                  <Text fontSize="14px" color={kotakColors.textMuted} margin="0">
                    Impact: {item.impact}
                  </Text>
                </Box>
              </FlexBox>
            </MotionBox>
          ))}
        </FlexBox>

        {/* Bottom Summary */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.9 }}
          style={{ marginTop: '20px' }}
        >
          <FlexBox
            style={{
              background: `linear-gradient(90deg, ${kotakColors.primary}20, ${kotakColors.primary}05)`,
              borderRadius: '12px',
              padding: '20px 28px',
            }}
            justifyContent="space-around"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="32px" fontWeight={700} color={kotakColors.danger} margin="0">
                3+
              </Text>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="4px 0 0 0">
                Major NBFC Crises in 2024
              </Text>
            </Box>
            <Box style={{ width: '1px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="32px" fontWeight={700} color={kotakColors.gold} margin="0">
                Rs 50+ Cr
              </Text>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="4px 0 0 0">
                Total Fraud Amount Reported
              </Text>
            </Box>
            <Box style={{ width: '1px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="32px" fontWeight={700} color={kotakColors.success} margin="0">
                10M+
              </Text>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="4px 0 0 0">
                Customers Seeking Alternatives
              </Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        The NBFC gold loan industry is facing a major trust crisis in 2024.
        IIFL ban, Manappuram fraud cases, multiple RBI penalties - these headlines
        have shaken customer confidence. Over 10 million customers are now actively
        looking for safer alternatives. This is our moment.
      </Notes>
    </Slide>
  );
};

export default IndustryTrustCrisisSlide;
