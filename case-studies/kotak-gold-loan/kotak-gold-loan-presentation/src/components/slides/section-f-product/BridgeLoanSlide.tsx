import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const oldProcess = [
  { step: 1, action: 'Arrange cash', issue: 'Need full amount upfront', icon: '💸' },
  { step: 2, action: 'Visit old NBFC', issue: 'Wait in queue', icon: '🏢' },
  { step: 3, action: 'Close loan', issue: 'Paperwork hassle', icon: '📝' },
  { step: 4, action: 'Transport gold', issue: 'Security risk', icon: '🚗' },
  { step: 5, action: 'Re-apply at bank', issue: 'Start from scratch', icon: '🔄' },
];

const newProcess = [
  { step: 1, action: 'Get pre-approved', benefit: 'Instant decision', icon: '✅' },
  { step: 2, action: 'Bridge loan disbursed', benefit: 'Zero interest, 7 days', icon: '🌉' },
  { step: 3, action: 'We close old loan', benefit: 'Switch Assist handles it', icon: '🤝' },
  { step: 4, action: 'Gold transferred securely', benefit: 'Insured transit', icon: '🔒' },
  { step: 5, action: 'Full loan disbursed', benefit: 'Same day', icon: '💰' },
];

export const BridgeLoanSlide: React.FC = () => {
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
            color={kotakColors.gold}
            margin="0 0 8px 0"
            style={{ letterSpacing: '2px', textTransform: 'uppercase' }}
          >
            KEY INNOVATION
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 8px 0">
            The Bridge Loan: Eliminating Cash Flow Fear
          </Heading>
          <Text fontSize="18px" color={kotakColors.textMuted} margin="0 0 24px 0">
            7-day zero-interest loan to close existing NBFC loan before gold transfer
          </Text>
        </MotionBox>

        {/* Two Process Flows */}
        <FlexBox flex={1} style={{ gap: '32px' }}>
          {/* Old Process */}
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
                padding: '20px',
                height: '100%',
                borderTop: `4px solid ${kotakColors.danger}`,
              }}
            >
              <FlexBox alignItems="center" style={{ gap: '10px', marginBottom: '16px' }}>
                <Text fontSize="24px" margin="0">😰</Text>
                <Text fontSize="18px" fontWeight={700} color={kotakColors.danger} margin="0">
                  Without Bridge Loan
                </Text>
              </FlexBox>

              <FlexBox flexDirection="column" style={{ gap: '8px' }}>
                {oldProcess.map((item, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.3, delay: 0.3 + index * 0.08 }}
                  >
                    <FlexBox
                      alignItems="center"
                      style={{
                        gap: '12px',
                        padding: '12px',
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
                      <Text fontSize="18px" margin="0">{item.icon}</Text>
                      <Box style={{ flex: 1 }}>
                        <Text fontSize="14px" fontWeight={500} color={kotakColors.white} margin="0">{item.action}</Text>
                        <Text fontSize="12px" color={kotakColors.textMuted} margin="0">{item.issue}</Text>
                      </Box>
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>

              <Box style={{ marginTop: '12px', padding: '10px', background: `${kotakColors.danger}15`, borderRadius: '8px', textAlign: 'center' }}>
                <Text fontSize="14px" fontWeight={600} color={kotakColors.danger} margin="0">
                  10-14 days + Cash arrangement stress
                </Text>
              </Box>
            </Box>
          </MotionBox>

          {/* New Process */}
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
                padding: '20px',
                height: '100%',
                borderTop: `4px solid ${kotakColors.success}`,
              }}
            >
              <FlexBox alignItems="center" style={{ gap: '10px', marginBottom: '16px' }}>
                <Text fontSize="24px" margin="0">😊</Text>
                <Text fontSize="18px" fontWeight={700} color={kotakColors.success} margin="0">
                  With Kotak Bridge Loan
                </Text>
              </FlexBox>

              <FlexBox flexDirection="column" style={{ gap: '8px' }}>
                {newProcess.map((item, index) => (
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
                        padding: '12px',
                        background: `${kotakColors.success}08`,
                        borderRadius: '8px',
                        border: `1px solid ${kotakColors.success}20`,
                      }}
                    >
                      <Box
                        style={{
                          width: '28px',
                          height: '28px',
                          borderRadius: '50%',
                          background: kotakColors.success,
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                          flexShrink: 0,
                        }}
                      >
                        <Text fontSize="12px" fontWeight={700} color={kotakColors.white} margin="0">
                          {item.step}
                        </Text>
                      </Box>
                      <Text fontSize="18px" margin="0">{item.icon}</Text>
                      <Box style={{ flex: 1 }}>
                        <Text fontSize="14px" fontWeight={500} color={kotakColors.white} margin="0">{item.action}</Text>
                        <Text fontSize="12px" color={kotakColors.success} margin="0">{item.benefit}</Text>
                      </Box>
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>

              <Box style={{ marginTop: '12px', padding: '10px', background: `${kotakColors.success}15`, borderRadius: '8px', textAlign: 'center' }}>
                <Text fontSize="14px" fontWeight={600} color={kotakColors.success} margin="0">
                  3 days + Zero upfront cash needed
                </Text>
              </Box>
            </Box>
          </MotionBox>
        </FlexBox>

        {/* Bridge Loan Details */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.8 }}
          style={{ marginTop: '16px' }}
        >
          <FlexBox
            style={{
              background: `linear-gradient(90deg, ${kotakColors.gold}20, ${kotakColors.gold}05)`,
              borderRadius: '12px',
              padding: '16px 24px',
              border: `1px solid ${kotakColors.gold}30`,
            }}
            justifyContent="space-around"
            alignItems="center"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.gold} margin="0">7 Days</Text>
              <Text fontSize="12px" color={kotakColors.textMuted} margin="2px 0 0 0">Tenure</Text>
            </Box>
            <Box style={{ width: '1px', height: '40px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.success} margin="0">0%</Text>
              <Text fontSize="12px" color={kotakColors.textMuted} margin="2px 0 0 0">Interest Rate</Text>
            </Box>
            <Box style={{ width: '1px', height: '40px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.white} margin="0">Up to Rs 25L</Text>
              <Text fontSize="12px" color={kotakColors.textMuted} margin="2px 0 0 0">Max Amount</Text>
            </Box>
            <Box style={{ width: '1px', height: '40px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.primary} margin="0">Auto-Close</Text>
              <Text fontSize="12px" color={kotakColors.textMuted} margin="2px 0 0 0">When gold arrives</Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        The Bridge Loan is our key innovation. It's a 7-day zero-interest loan up to Rs 25L
        that allows customers to close their NBFC loan without arranging cash upfront.
        It auto-closes when gold is transferred to Kotak. This removes the #1 switching barrier.
      </Notes>
    </Slide>
  );
};

export default BridgeLoanSlide;
