import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const trustMessages = [
  {
    before: '"What if my gold gets lost in transfer?"',
    after: 'Bank-grade vault security + Rs 10L transit insurance included',
    icon: '🏦',
    theme: 'Security',
  },
  {
    before: '"NBFCs are everywhere, banks seem complicated"',
    after: '811+ branches with gold loan capability, same-day service',
    icon: '📍',
    theme: 'Accessibility',
  },
  {
    before: '"Will I get the same loan amount?"',
    after: 'Price match guarantee + 75% LTV on same gold',
    icon: '💰',
    theme: 'Value',
  },
  {
    before: '"I don\'t want to deal with new paperwork"',
    after: 'We handle old loan closure, minimal docs from you',
    icon: '📋',
    theme: 'Convenience',
  },
];

export const TrustMessagingSlide: React.FC = () => {
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
            TRUST BUILDING
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 8px 0">
            Implicit Trust Messaging
          </Heading>
          <Text fontSize="18px" color={kotakColors.textMuted} margin="0 0 32px 0">
            Addressing customer concerns before they voice them
          </Text>
        </MotionBox>

        {/* Message Cards */}
        <FlexBox flex={1} flexDirection="column" style={{ gap: '16px' }}>
          {trustMessages.map((msg, index) => (
            <MotionBox
              key={index}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.2 + index * 0.1 }}
              style={{
                background: kotakColors.darkCard,
                borderRadius: '12px',
                padding: '20px 24px',
                overflow: 'hidden',
              }}
            >
              <FlexBox alignItems="center" style={{ gap: '24px' }}>
                {/* Icon */}
                <Box
                  style={{
                    width: '56px',
                    height: '56px',
                    borderRadius: '12px',
                    background: `${kotakColors.primary}15`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    flexShrink: 0,
                  }}
                >
                  <Text fontSize="28px" margin="0">{msg.icon}</Text>
                </Box>

                {/* Before - Fear */}
                <Box style={{ flex: 1 }}>
                  <Text fontSize="11px" fontWeight={600} color={kotakColors.danger} margin="0 0 4px 0" style={{ textTransform: 'uppercase' }}>
                    Customer Fear
                  </Text>
                  <Text fontSize="16px" color={kotakColors.textMuted} margin="0" style={{ fontStyle: 'italic' }}>
                    {msg.before}
                  </Text>
                </Box>

                {/* Arrow */}
                <Box style={{ flexShrink: 0 }}>
                  <Text fontSize="24px" color={kotakColors.gold} margin="0">→</Text>
                </Box>

                {/* After - Our Response */}
                <Box style={{ flex: 1.2 }}>
                  <FlexBox alignItems="center" style={{ gap: '8px', marginBottom: '4px' }}>
                    <Text fontSize="11px" fontWeight={600} color={kotakColors.success} margin="0" style={{ textTransform: 'uppercase' }}>
                      Kotak Response
                    </Text>
                    <Box style={{ padding: '2px 8px', background: `${kotakColors.gold}20`, borderRadius: '4px' }}>
                      <Text fontSize="10px" fontWeight={600} color={kotakColors.gold} margin="0">
                        {msg.theme}
                      </Text>
                    </Box>
                  </FlexBox>
                  <Text fontSize="16px" fontWeight={500} color={kotakColors.white} margin="0">
                    {msg.after}
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
          transition={{ duration: 0.5, delay: 0.7 }}
          style={{ marginTop: '20px' }}
        >
          <FlexBox
            style={{
              background: `linear-gradient(90deg, ${kotakColors.secondary}20, ${kotakColors.secondary}05)`,
              borderRadius: '12px',
              padding: '20px 28px',
            }}
            alignItems="center"
            justifyContent="space-between"
          >
            <FlexBox alignItems="center" style={{ gap: '16px' }}>
              <Text fontSize="32px" margin="0">🎯</Text>
              <Box>
                <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0 0 4px 0">
                  Core Message Strategy
                </Text>
                <Text fontSize="14px" color={kotakColors.textMuted} margin="0">
                  Lead with bank safety, not rates — trust drives conversion
                </Text>
              </Box>
            </FlexBox>
            <Box
              style={{
                padding: '12px 20px',
                background: kotakColors.primary,
                borderRadius: '8px',
              }}
            >
              <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0">
                "Your Gold Deserves a Bank"
              </Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Trust messaging is crucial. We address the top 4 customer fears with concrete responses.
        The key insight: lead with bank safety and security, not just lower rates.
        "Your Gold Deserves a Bank" encapsulates our positioning against NBFCs.
      </Notes>
    </Slide>
  );
};

export default TrustMessagingSlide;
