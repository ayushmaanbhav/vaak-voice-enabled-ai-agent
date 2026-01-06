import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { doorstepPilot } from '../../../data/doorstepData';

const MotionBox = motion(Box);

const viabilityData = [
  { ticket: 'Rs 25K', cost: 'Rs 850', nii: 'Rs 625', viable: false, margin: '-Rs 225' },
  { ticket: 'Rs 50K', cost: 'Rs 850', nii: 'Rs 1,250', viable: true, margin: '+Rs 400' },
  { ticket: 'Rs 1L', cost: 'Rs 850', nii: 'Rs 2,500', viable: true, margin: '+Rs 1,650' },
  { ticket: 'Rs 2L', cost: 'Rs 850', nii: 'Rs 5,000', viable: true, margin: '+Rs 4,150' },
];

export const DoorstepEconomicsSlide: React.FC = () => {
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
            UNIT ECONOMICS
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Doorstep Service Economics
          </Heading>
        </MotionBox>

        <FlexBox flex={1} style={{ gap: '24px' }}>
          {/* Left: Cost Breakdown */}
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
              }}
            >
              <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0 0 20px 0">
                Cost per Transaction
              </Text>

              <FlexBox flexDirection="column" style={{ gap: '10px', marginBottom: '20px' }}>
                {doorstepPilot.costBreakdown.map((item, index) => (
                  <FlexBox
                    key={index}
                    justifyContent="space-between"
                    alignItems="center"
                    style={{ padding: '12px 14px', background: 'rgba(255,255,255,0.03)', borderRadius: '8px' }}
                  >
                    <FlexBox alignItems="center" style={{ gap: '10px' }}>
                      <Text fontSize="18px" margin="0">{item.icon}</Text>
                      <Text fontSize="14px" color={kotakColors.white} margin="0">{item.item}</Text>
                    </FlexBox>
                    <Text fontSize="15px" fontWeight={600} color={kotakColors.gold} margin="0">{item.cost}</Text>
                  </FlexBox>
                ))}
              </FlexBox>

              {/* Total */}
              <Box style={{ padding: '16px', background: `${kotakColors.gold}15`, borderRadius: '10px' }}>
                <FlexBox justifyContent="space-between" alignItems="center">
                  <Text fontSize="14px" fontWeight={600} color={kotakColors.gold} margin="0">TOTAL COST/VISIT</Text>
                  <Text fontSize="24px" fontWeight={700} color={kotakColors.gold} margin="0">{doorstepPilot.totalCostPerTransaction}</Text>
                </FlexBox>
              </Box>

              {/* Break-even */}
              <Box style={{ marginTop: '16px', padding: '14px', background: `${kotakColors.success}10`, borderRadius: '10px', border: `1px solid ${kotakColors.success}30` }}>
                <Text fontSize="13px" fontWeight={600} color={kotakColors.success} margin="0 0 4px 0">
                  BREAK-EVEN TICKET SIZE
                </Text>
                <Text fontSize="20px" fontWeight={700} color={kotakColors.white} margin="0">
                  Rs {doorstepPilot.breakEvenTicket.toLocaleString()}
                </Text>
                <Text fontSize="12px" color={kotakColors.textMuted} margin="4px 0 0 0">
                  Minimum loan amount for profitable doorstep service
                </Text>
              </Box>
            </Box>
          </MotionBox>

          {/* Right: Viability by Ticket Size */}
          <MotionBox
            initial={{ opacity: 0, x: 30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            style={{ flex: 1.2 }}
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
                Viability by Ticket Size (Annual NII vs Cost)
              </Text>

              {/* Table Header */}
              <FlexBox style={{ padding: '10px 14px', background: kotakColors.primary, borderRadius: '8px 8px 0 0' }}>
                <Box style={{ flex: 1 }}><Text fontSize="12px" fontWeight={600} color={kotakColors.white} margin="0">Ticket</Text></Box>
                <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="12px" fontWeight={600} color={kotakColors.white} margin="0">Cost</Text></Box>
                <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="12px" fontWeight={600} color={kotakColors.white} margin="0">Annual NII</Text></Box>
                <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="12px" fontWeight={600} color={kotakColors.white} margin="0">Margin</Text></Box>
                <Box style={{ flex: 0.6, textAlign: 'center' }}><Text fontSize="12px" fontWeight={600} color={kotakColors.white} margin="0">Viable?</Text></Box>
              </FlexBox>

              {/* Rows */}
              {viabilityData.map((row, index) => (
                <MotionBox
                  key={index}
                  initial={{ opacity: 0, x: -10 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.3, delay: 0.4 + index * 0.08 }}
                >
                  <FlexBox
                    style={{
                      padding: '14px',
                      borderBottom: '1px solid rgba(255,255,255,0.05)',
                      background: row.viable ? `${kotakColors.success}05` : `${kotakColors.danger}05`,
                    }}
                    alignItems="center"
                  >
                    <Box style={{ flex: 1 }}><Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0">{row.ticket}</Text></Box>
                    <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="14px" color={kotakColors.textMuted} margin="0">{row.cost}</Text></Box>
                    <Box style={{ flex: 1, textAlign: 'center' }}><Text fontSize="14px" color={kotakColors.white} margin="0">{row.nii}</Text></Box>
                    <Box style={{ flex: 1, textAlign: 'center' }}>
                      <Text fontSize="14px" fontWeight={600} color={row.viable ? kotakColors.success : kotakColors.danger} margin="0">
                        {row.margin}
                      </Text>
                    </Box>
                    <Box style={{ flex: 0.6, textAlign: 'center' }}>
                      <Text fontSize="18px" margin="0">{row.viable ? '✅' : '❌'}</Text>
                    </Box>
                  </FlexBox>
                </MotionBox>
              ))}

              {/* Key Insight */}
              <Box style={{ marginTop: '16px', padding: '14px', background: 'rgba(255,255,255,0.03)', borderRadius: '10px' }}>
                <Text fontSize="13px" color={kotakColors.textMuted} margin="0">
                  <Text fontSize="13px" fontWeight={600} color={kotakColors.white} style={{ display: 'inline' }}>Strategy:</Text>{' '}
                  Focus doorstep on Rs 50K+ tickets. Smaller tickets directed to branch channel.
                </Text>
              </Box>
            </Box>
          </MotionBox>
        </FlexBox>

        {/* Bottom Summary */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.7 }}
          style={{ marginTop: '16px' }}
        >
          <FlexBox
            style={{
              background: kotakColors.darkCard,
              borderRadius: '12px',
              padding: '14px 24px',
            }}
            justifyContent="space-around"
            alignItems="center"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="20px" fontWeight={700} color={kotakColors.gold} margin="0">Rs 50K+</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Minimum Viable Ticket</Text>
            </Box>
            <Box style={{ width: '1px', height: '35px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="20px" fontWeight={700} color={kotakColors.success} margin="0">Rs 1.5L</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Avg Doorstep Ticket Target</Text>
            </Box>
            <Box style={{ width: '1px', height: '35px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="20px" fontWeight={700} color={kotakColors.primary} margin="0">65%</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Target Gross Margin</Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Doorstep cost is Rs 850/transaction. Break-even at Rs 34K ticket size.
        Strategy: Focus doorstep on Rs 50K+ tickets for profitability.
        Target average ticket: Rs 1.5L with 65% gross margin.
      </Notes>
    </Slide>
  );
};

export default DoorstepEconomicsSlide;
