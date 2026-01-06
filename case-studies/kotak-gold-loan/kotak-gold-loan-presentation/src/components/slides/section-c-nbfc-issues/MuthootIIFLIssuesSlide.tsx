import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { muthootIssues, iiflCrisis } from '../../../data/competitorData';

const MotionBox = motion(Box);

export const MuthootIIFLIssuesSlide: React.FC = () => {
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
            COMPETITOR ANALYSIS: MUTHOOT & IIFL
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            More Cracks in the NBFC Armor
          </Heading>
        </MotionBox>

        <FlexBox flex={1} style={{ gap: '24px' }}>
          {/* Left: Muthoot */}
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
                borderTop: `4px solid ${kotakColors.gold}`,
              }}
            >
              <FlexBox alignItems="center" justifyContent="space-between" style={{ marginBottom: '20px' }}>
                <Text fontSize="22px" fontWeight={700} color={kotakColors.white} margin="0">
                  Muthoot Finance
                </Text>
                <Box
                  style={{
                    background: `${kotakColors.danger}20`,
                    padding: '8px 16px',
                    borderRadius: '20px',
                  }}
                >
                  <FlexBox alignItems="center" style={{ gap: '6px' }}>
                    <Text fontSize="16px" margin="0">⭐</Text>
                    <Text fontSize="18px" fontWeight={700} color={kotakColors.danger} margin="0">
                      {muthootIssues.rating.score}/{muthootIssues.rating.maxScore}
                    </Text>
                  </FlexBox>
                </Box>
              </FlexBox>

              <Text fontSize="14px" fontWeight={600} color={kotakColors.textMuted} margin="0 0 12px 0">
                TOP CUSTOMER COMPLAINTS
              </Text>

              <FlexBox flexDirection="column" style={{ gap: '10px' }}>
                {muthootIssues.complaints.map((complaint, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.3, delay: 0.4 + index * 0.1 }}
                  >
                    <FlexBox
                      alignItems="center"
                      style={{
                        gap: '12px',
                        padding: '12px 16px',
                        background: 'rgba(255,255,255,0.03)',
                        borderRadius: '8px',
                        borderLeft: `3px solid ${kotakColors.danger}`,
                      }}
                    >
                      <Text fontSize="18px" margin="0">❌</Text>
                      <Box>
                        <Text fontSize="15px" fontWeight={600} color={kotakColors.white} margin="0">
                          {complaint.type}
                        </Text>
                        {complaint.quote && (
                          <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">
                            {complaint.quote}
                          </Text>
                        )}
                      </Box>
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>

              <Box
                style={{
                  marginTop: '20px',
                  padding: '16px',
                  background: `${kotakColors.gold}10`,
                  borderRadius: '10px',
                }}
              >
                <Text fontSize="14px" color={kotakColors.gold} margin="0" style={{ lineHeight: 1.5 }}>
                  "Largest gold loan NBFC with worst customer satisfaction scores" — Market analysts note the disconnect between size and service quality.
                </Text>
              </Box>
            </Box>
          </MotionBox>

          {/* Right: IIFL */}
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
                borderTop: `4px solid ${kotakColors.danger}`,
              }}
            >
              <FlexBox alignItems="center" justifyContent="space-between" style={{ marginBottom: '20px' }}>
                <Text fontSize="22px" fontWeight={700} color={kotakColors.white} margin="0">
                  IIFL Finance
                </Text>
                <Box
                  style={{
                    background: `${kotakColors.danger}30`,
                    padding: '8px 16px',
                    borderRadius: '20px',
                  }}
                >
                  <Text fontSize="14px" fontWeight={700} color={kotakColors.danger} margin="0">
                    🚫 BANNED
                  </Text>
                </Box>
              </FlexBox>

              <Text fontSize="14px" fontWeight={600} color={kotakColors.textMuted} margin="0 0 12px 0">
                RBI ACTION: {iiflCrisis.rbiAction.action}
              </Text>

              <Text fontSize="14px" fontWeight={600} color={kotakColors.textMuted} margin="20px 0 12px 0">
                KEY VIOLATIONS
              </Text>

              <FlexBox flexDirection="column" style={{ gap: '10px' }}>
                {iiflCrisis.violations.map((violation, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.3, delay: 0.5 + index * 0.1 }}
                  >
                    <FlexBox
                      alignItems="center"
                      style={{
                        gap: '12px',
                        padding: '12px 16px',
                        background: 'rgba(255,255,255,0.03)',
                        borderRadius: '8px',
                        borderLeft: `3px solid ${kotakColors.warning}`,
                      }}
                    >
                      <Text fontSize="18px" margin="0">⚠️</Text>
                      <Box>
                        <Text fontSize="15px" fontWeight={600} color={kotakColors.white} margin="0">
                          {violation.type}
                        </Text>
                        <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">
                          {violation.description}
                        </Text>
                      </Box>
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>

              <Box
                style={{
                  marginTop: '20px',
                  padding: '16px',
                  background: `${kotakColors.danger}10`,
                  borderRadius: '10px',
                }}
              >
                <FlexBox alignItems="center" style={{ gap: '12px' }}>
                  <Text fontSize="28px" margin="0">👥</Text>
                  <Box>
                    <Text fontSize="24px" fontWeight={700} color={kotakColors.danger} margin="0">
                      5.5M+ Customers
                    </Text>
                    <Text fontSize="14px" color={kotakColors.textMuted} margin="4px 0 0 0">
                      Left stranded by the RBI ban
                    </Text>
                  </Box>
                </FlexBox>
              </Box>
            </Box>
          </MotionBox>
        </FlexBox>

        {/* Bottom Summary */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.9 }}
          style={{ marginTop: '20px' }}
        >
          <Box
            style={{
              background: `linear-gradient(90deg, ${kotakColors.success}15, ${kotakColors.success}05)`,
              borderRadius: '12px',
              padding: '16px 24px',
              border: `1px solid ${kotakColors.success}30`,
            }}
          >
            <FlexBox alignItems="center" justifyContent="center" style={{ gap: '16px' }}>
              <Text fontSize="28px" margin="0">🏦</Text>
              <Text fontSize="18px" color={kotakColors.white} margin="0">
                <Text fontSize="18px" fontWeight={700} color={kotakColors.success} style={{ display: 'inline' }}>
                  Kotak Opportunity:
                </Text>
                {' '}Positioned as the trusted bank alternative to failing NBFCs
              </Text>
            </FlexBox>
          </Box>
        </MotionBox>
      </FlexBox>

      <Notes>
        Muthoot has the worst customer satisfaction at 2.19/5 despite being the largest.
        IIFL was banned by RBI in March 2024 for LTV breaches, cash violations, and improper valuations.
        Over 5.5 million IIFL customers are now looking for alternatives. We can position Kotak
        as the safe, trusted bank option.
      </Notes>
    </Slide>
  );
};

export default MuthootIIFLIssuesSlide;
