import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const phases = [
  {
    phase: 'Phase 1',
    timeline: 'Month 1-3',
    city: 'Mumbai',
    activities: ['20 field executives', 'CV app integration', '500 customers'],
    color: kotakColors.primary,
  },
  {
    phase: 'Phase 2',
    timeline: 'Month 4-6',
    city: 'Chennai + Hyderabad',
    activities: ['Expand to 50 executives', 'Process optimization', '2,000 customers'],
    color: kotakColors.gold,
  },
  {
    phase: 'Phase 3',
    timeline: 'Month 7-12',
    city: 'All 5 Cities',
    activities: ['100+ field force', 'Full CV deployment', '5,500 customers'],
    color: kotakColors.success,
  },
];

const successMetrics = [
  { metric: 'Customer NPS', target: '> 70', icon: '😊' },
  { metric: 'TAT (Request to Disbursal)', target: '< 4 hours', icon: '⏱️' },
  { metric: 'Gold Handling Incidents', target: '0', icon: '🔒' },
  { metric: 'Unit Economics', target: '> 50% margin', icon: '💰' },
];

export const DoorstepPilotPlanSlide: React.FC = () => {
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
            PILOT ROLLOUT
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            12-Month Pilot Plan
          </Heading>
        </MotionBox>

        {/* Phases Timeline */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.2 }}
          style={{ marginBottom: '24px' }}
        >
          <Box
            style={{
              background: kotakColors.darkCard,
              borderRadius: '16px',
              padding: '24px',
            }}
          >
            <FlexBox style={{ gap: '16px', position: 'relative' }}>
              {/* Connecting Line */}
              <Box
                style={{
                  position: 'absolute',
                  top: '40px',
                  left: '80px',
                  right: '80px',
                  height: '4px',
                  background: `linear-gradient(90deg, ${kotakColors.primary}, ${kotakColors.gold}, ${kotakColors.success})`,
                  borderRadius: '2px',
                }}
              />

              {phases.map((phase, index) => (
                <MotionBox
                  key={index}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.4, delay: 0.3 + index * 0.15 }}
                  style={{ flex: 1, textAlign: 'center', position: 'relative', zIndex: 1 }}
                >
                  {/* Phase Circle */}
                  <Box
                    style={{
                      width: '80px',
                      height: '80px',
                      borderRadius: '50%',
                      background: kotakColors.dark,
                      border: `4px solid ${phase.color}`,
                      display: 'flex',
                      flexDirection: 'column',
                      alignItems: 'center',
                      justifyContent: 'center',
                      margin: '0 auto 16px',
                    }}
                  >
                    <Text fontSize="12px" fontWeight={700} color={phase.color} margin="0">{phase.phase}</Text>
                    <Text fontSize="10px" color={kotakColors.textMuted} margin="0">{phase.timeline}</Text>
                  </Box>

                  {/* Phase Card */}
                  <Box
                    style={{
                      background: 'rgba(255,255,255,0.03)',
                      borderRadius: '12px',
                      padding: '16px',
                      borderTop: `3px solid ${phase.color}`,
                    }}
                  >
                    <Text fontSize="16px" fontWeight={700} color={kotakColors.white} margin="0 0 12px 0">
                      {phase.city}
                    </Text>
                    <FlexBox flexDirection="column" style={{ gap: '6px' }}>
                      {phase.activities.map((activity, i) => (
                        <FlexBox key={i} alignItems="center" style={{ gap: '6px' }}>
                          <Box style={{ width: '5px', height: '5px', borderRadius: '50%', background: phase.color }} />
                          <Text fontSize="12px" color={kotakColors.textMuted} margin="0" style={{ textAlign: 'left' }}>{activity}</Text>
                        </FlexBox>
                      ))}
                    </FlexBox>
                  </Box>
                </MotionBox>
              ))}
            </FlexBox>
          </Box>
        </MotionBox>

        {/* Success Metrics & Investment */}
        <FlexBox flex={1} style={{ gap: '24px' }}>
          {/* Success Metrics */}
          <MotionBox
            initial={{ opacity: 0, x: -30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.5 }}
            style={{ flex: 1 }}
          >
            <Box
              style={{
                background: kotakColors.darkCard,
                borderRadius: '16px',
                padding: '20px',
                height: '100%',
              }}
            >
              <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0 0 16px 0">
                Success Metrics
              </Text>
              <FlexBox flexDirection="column" style={{ gap: '10px' }}>
                {successMetrics.map((item, i) => (
                  <FlexBox key={i} alignItems="center" justifyContent="space-between" style={{ padding: '12px', background: 'rgba(255,255,255,0.03)', borderRadius: '8px' }}>
                    <FlexBox alignItems="center" style={{ gap: '10px' }}>
                      <Text fontSize="20px" margin="0">{item.icon}</Text>
                      <Text fontSize="13px" color={kotakColors.white} margin="0">{item.metric}</Text>
                    </FlexBox>
                    <Box style={{ padding: '4px 10px', background: `${kotakColors.success}20`, borderRadius: '4px' }}>
                      <Text fontSize="12px" fontWeight={600} color={kotakColors.success} margin="0">{item.target}</Text>
                    </Box>
                  </FlexBox>
                ))}
              </FlexBox>
            </Box>
          </MotionBox>

          {/* Pilot Investment */}
          <MotionBox
            initial={{ opacity: 0, x: 30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.6 }}
            style={{ flex: 1 }}
          >
            <Box
              style={{
                background: kotakColors.darkCard,
                borderRadius: '16px',
                padding: '20px',
                height: '100%',
              }}
            >
              <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0 0 16px 0">
                Pilot Investment
              </Text>
              <FlexBox flexDirection="column" style={{ gap: '10px' }}>
                {[
                  { item: 'Field Force Training', cost: 'Rs 50L' },
                  { item: 'Equipment & Vehicles', cost: 'Rs 80L' },
                  { item: 'Technology Integration', cost: 'Rs 40L' },
                  { item: 'Insurance & Security', cost: 'Rs 30L' },
                ].map((item, i) => (
                  <FlexBox key={i} justifyContent="space-between" style={{ padding: '10px 12px', background: 'rgba(255,255,255,0.03)', borderRadius: '6px' }}>
                    <Text fontSize="13px" color={kotakColors.textMuted} margin="0">{item.item}</Text>
                    <Text fontSize="13px" fontWeight={600} color={kotakColors.gold} margin="0">{item.cost}</Text>
                  </FlexBox>
                ))}
              </FlexBox>
              <Box style={{ marginTop: '12px', padding: '12px', background: `${kotakColors.gold}15`, borderRadius: '8px' }}>
                <FlexBox justifyContent="space-between" alignItems="center">
                  <Text fontSize="13px" fontWeight={600} color={kotakColors.gold} margin="0">PILOT TOTAL</Text>
                  <Text fontSize="18px" fontWeight={700} color={kotakColors.gold} margin="0">Rs 2 Cr</Text>
                </FlexBox>
              </Box>
            </Box>
          </MotionBox>
        </FlexBox>

        {/* Go/No-Go Decision */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.8 }}
          style={{ marginTop: '16px' }}
        >
          <Box
            style={{
              background: `linear-gradient(90deg, ${kotakColors.primary}15, ${kotakColors.primary}05)`,
              borderRadius: '12px',
              padding: '14px 24px',
              border: `1px solid ${kotakColors.primary}30`,
            }}
          >
            <FlexBox alignItems="center" justifyContent="center" style={{ gap: '16px' }}>
              <Text fontSize="24px" margin="0">🚀</Text>
              <Text fontSize="15px" color={kotakColors.white} margin="0">
                <Text fontSize="15px" fontWeight={700} color={kotakColors.primary} style={{ display: 'inline' }}>Go/No-Go Decision:</Text>{' '}
                Month 6 review based on unit economics & customer feedback. Scale to 20 cities if targets met.
              </Text>
            </FlexBox>
          </Box>
        </MotionBox>
      </FlexBox>

      <Notes>
        12-month pilot in 3 phases: Mumbai first, then Chennai/Hyderabad, then all 5 cities.
        Success metrics: NPS greater than 70, TAT under 4 hours, zero incidents, over 50% margin.
        Pilot investment: Rs 2 Cr. Go/No-Go at Month 6.
      </Notes>
    </Slide>
  );
};

export default DoorstepPilotPlanSlide;
