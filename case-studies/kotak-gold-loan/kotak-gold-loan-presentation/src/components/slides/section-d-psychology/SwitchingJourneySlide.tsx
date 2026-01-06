import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const traditionalSteps = [
  { step: '1', label: 'Apply at new lender', duration: 'Day 1-2' },
  { step: '2', label: 'Wait for approval', duration: 'Day 3-5' },
  { step: '3', label: 'Arrange cash to close old loan', duration: 'Day 6-8' },
  { step: '4', label: 'Visit old NBFC, close loan', duration: 'Day 9-10' },
  { step: '5', label: 'Transport gold to new lender', duration: 'Day 11-12' },
  { step: '6', label: 'Re-verification & disbursement', duration: 'Day 13-14' },
];

const kotakSteps = [
  { step: '1', label: 'Pre-approval online', duration: 'Day 1' },
  { step: '2', label: 'Bridge loan disbursed', duration: 'Day 1' },
  { step: '3', label: 'Close old loan & transfer gold', duration: 'Day 2' },
  { step: '4', label: 'Full loan disbursed', duration: 'Day 3' },
];

export const SwitchingJourneySlide: React.FC = () => {
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
            THE FRICTION PROBLEM
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 32px 0">
            Reducing the Switching Journey
          </Heading>
        </MotionBox>

        {/* Two Journeys Side by Side */}
        <FlexBox flex={1} style={{ gap: '32px' }}>
          {/* Traditional Journey */}
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
                borderTop: `4px solid ${kotakColors.danger}`,
              }}
            >
              <FlexBox alignItems="center" style={{ gap: '12px', marginBottom: '20px' }}>
                <Text fontSize="28px" margin="0">😰</Text>
                <Box>
                  <Text fontSize="18px" fontWeight={700} color={kotakColors.danger} margin="0">
                    Traditional Process
                  </Text>
                  <Text fontSize="14px" color={kotakColors.textMuted} margin="0">
                    10-14 days, high friction
                  </Text>
                </Box>
              </FlexBox>

              <FlexBox flexDirection="column" style={{ gap: '8px' }}>
                {traditionalSteps.map((item, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.3, delay: 0.4 + index * 0.08 }}
                  >
                    <FlexBox
                      alignItems="center"
                      style={{
                        gap: '12px',
                        padding: '10px 14px',
                        background: 'rgba(255,255,255,0.03)',
                        borderRadius: '8px',
                      }}
                    >
                      <Box
                        style={{
                          width: '28px',
                          height: '28px',
                          borderRadius: '50%',
                          background: `${kotakColors.danger}20`,
                          border: `2px solid ${kotakColors.danger}`,
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                          flexShrink: 0,
                        }}
                      >
                        <Text fontSize="12px" fontWeight={700} color={kotakColors.danger} margin="0">
                          {item.step}
                        </Text>
                      </Box>
                      <Box style={{ flex: 1 }}>
                        <Text fontSize="14px" color={kotakColors.white} margin="0">
                          {item.label}
                        </Text>
                      </Box>
                      <Text fontSize="12px" color={kotakColors.textMuted} margin="0">
                        {item.duration}
                      </Text>
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>

              <Box style={{ marginTop: '16px', padding: '12px', background: `${kotakColors.danger}15`, borderRadius: '8px' }}>
                <Text fontSize="14px" fontWeight={600} color={kotakColors.danger} margin="0" style={{ textAlign: 'center' }}>
                  Total: 10-14 Days + Cash Arrangement Stress
                </Text>
              </Box>
            </Box>
          </MotionBox>

          {/* Kotak Journey */}
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
                borderTop: `4px solid ${kotakColors.success}`,
              }}
            >
              <FlexBox alignItems="center" style={{ gap: '12px', marginBottom: '20px' }}>
                <Text fontSize="28px" margin="0">😊</Text>
                <Box>
                  <Text fontSize="18px" fontWeight={700} color={kotakColors.success} margin="0">
                    Kotak Switch & Save
                  </Text>
                  <Text fontSize="14px" color={kotakColors.textMuted} margin="0">
                    3 days, zero stress
                  </Text>
                </Box>
              </FlexBox>

              <FlexBox flexDirection="column" style={{ gap: '12px' }}>
                {kotakSteps.map((item, index) => (
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
                        padding: '14px 18px',
                        background: `${kotakColors.success}10`,
                        borderRadius: '10px',
                        border: `1px solid ${kotakColors.success}30`,
                      }}
                    >
                      <Box
                        style={{
                          width: '32px',
                          height: '32px',
                          borderRadius: '50%',
                          background: kotakColors.success,
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                          flexShrink: 0,
                        }}
                      >
                        <Text fontSize="14px" fontWeight={700} color={kotakColors.white} margin="0">
                          {item.step}
                        </Text>
                      </Box>
                      <Box style={{ flex: 1 }}>
                        <Text fontSize="16px" fontWeight={500} color={kotakColors.white} margin="0">
                          {item.label}
                        </Text>
                      </Box>
                      <Box
                        style={{
                          padding: '4px 10px',
                          background: `${kotakColors.success}30`,
                          borderRadius: '12px',
                        }}
                      >
                        <Text fontSize="12px" fontWeight={600} color={kotakColors.success} margin="0">
                          {item.duration}
                        </Text>
                      </Box>
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>

              <Box style={{ marginTop: '20px', padding: '14px', background: `${kotakColors.success}15`, borderRadius: '10px' }}>
                <Text fontSize="16px" fontWeight={600} color={kotakColors.success} margin="0" style={{ textAlign: 'center' }}>
                  Total: 3 Days + Zero Cash Needed Upfront
                </Text>
              </Box>
            </Box>
          </MotionBox>
        </FlexBox>

        {/* Improvement Metric */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.9 }}
          style={{ marginTop: '20px' }}
        >
          <FlexBox
            style={{
              background: `linear-gradient(90deg, ${kotakColors.gold}15, ${kotakColors.gold}05)`,
              borderRadius: '12px',
              padding: '16px 32px',
              gap: '40px',
            }}
            justifyContent="center"
            alignItems="center"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="36px" fontWeight={700} color={kotakColors.gold} margin="0">
                78%
              </Text>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="4px 0 0 0">
                Faster Process
              </Text>
            </Box>
            <Box style={{ width: '1px', height: '50px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="36px" fontWeight={700} color={kotakColors.gold} margin="0">
                100%
              </Text>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="4px 0 0 0">
                No Upfront Cash
              </Text>
            </Box>
            <Box style={{ width: '1px', height: '50px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="36px" fontWeight={700} color={kotakColors.gold} margin="0">
                1
              </Text>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="4px 0 0 0">
                Branch Visit Only
              </Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        The traditional switching process takes 10-14 days and requires customers to arrange
        cash upfront. Our process cuts this to 3 days with zero upfront cash needed.
        The Bridge Loan is the key innovation - it provides liquidity to close the old loan.
      </Notes>
    </Slide>
  );
};

export default SwitchingJourneySlide;
