import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { doorstepPilot } from '../../../data/doorstepData';

const MotionBox = motion(Box);

const opportunityPoints = [
  { icon: '💰', point: 'High-value tickets (Rs 50K+) justify service cost', impact: 'Economics work' },
  { icon: '🏠', point: 'Customers prefer home convenience over branch visits', impact: 'Higher conversion' },
  { icon: '⏱️', point: 'Working professionals can\'t visit during banking hours', impact: 'New segment access' },
  { icon: '🔒', point: 'Security concerns traveling with gold', impact: 'Trust building' },
];

export const DoorstepOpportunitySlide: React.FC = () => {
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
            DOORSTEP SERVICE
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            The Doorstep Opportunity
          </Heading>
        </MotionBox>

        <FlexBox flex={1} style={{ gap: '24px' }}>
          {/* Left: Why Doorstep */}
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
                Why Doorstep Service?
              </Text>

              <FlexBox flexDirection="column" style={{ gap: '12px' }}>
                {opportunityPoints.map((item, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.4, delay: 0.3 + index * 0.1 }}
                    style={{
                      background: 'rgba(255,255,255,0.03)',
                      borderRadius: '10px',
                      padding: '16px',
                      borderLeft: `3px solid ${kotakColors.gold}`,
                    }}
                  >
                    <FlexBox alignItems="center" style={{ gap: '12px' }}>
                      <Text fontSize="24px" margin="0">{item.icon}</Text>
                      <Box style={{ flex: 1 }}>
                        <Text fontSize="14px" fontWeight={500} color={kotakColors.white} margin="0 0 4px 0">
                          {item.point}
                        </Text>
                        <Text fontSize="12px" color={kotakColors.success} margin="0">
                          → {item.impact}
                        </Text>
                      </Box>
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>
            </Box>
          </MotionBox>

          {/* Right: Pilot Cities Map */}
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
                5-City Pilot Program
              </Text>

              <FlexBox flexDirection="column" style={{ gap: '10px' }}>
                {doorstepPilot.pilotCities.map((city, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.3, delay: 0.4 + index * 0.08 }}
                  >
                    <FlexBox
                      alignItems="center"
                      justifyContent="space-between"
                      style={{
                        padding: '14px 16px',
                        background: index === 0 ? `${kotakColors.primary}15` : 'rgba(255,255,255,0.03)',
                        borderRadius: '10px',
                        border: index === 0 ? `1px solid ${kotakColors.primary}30` : 'none',
                      }}
                    >
                      <FlexBox alignItems="center" style={{ gap: '12px' }}>
                        <Text fontSize="20px" margin="0">📍</Text>
                        <Box>
                          <Text fontSize="15px" fontWeight={600} color={kotakColors.white} margin="0">
                            {city.city}
                          </Text>
                          <Text fontSize="12px" color={kotakColors.textMuted} margin="0">
                            {city.branches} branches
                          </Text>
                        </Box>
                      </FlexBox>
                      <Box style={{ textAlign: 'right' }}>
                        <Text fontSize="14px" fontWeight={700} color={kotakColors.gold} margin="0">
                          {city.targetCustomers}
                        </Text>
                        <Text fontSize="11px" color={kotakColors.textMuted} margin="0">
                          target customers
                        </Text>
                      </Box>
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>

              {/* Total */}
              <Box style={{ marginTop: '16px', padding: '14px', background: `${kotakColors.gold}15`, borderRadius: '10px' }}>
                <FlexBox justifyContent="space-between" alignItems="center">
                  <Text fontSize="14px" fontWeight={600} color={kotakColors.gold} margin="0">PILOT TOTAL</Text>
                  <Text fontSize="20px" fontWeight={700} color={kotakColors.gold} margin="0">5,500 Customers</Text>
                </FlexBox>
              </Box>
            </Box>
          </MotionBox>
        </FlexBox>

        {/* Bottom Insight */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.7 }}
          style={{ marginTop: '16px' }}
        >
          <Box
            style={{
              background: kotakColors.darkCard,
              borderRadius: '12px',
              padding: '16px 24px',
            }}
          >
            <FlexBox alignItems="center" style={{ gap: '16px' }}>
              <Text fontSize="28px" margin="0">🎯</Text>
              <Box>
                <Text fontSize="15px" fontWeight={600} color={kotakColors.white} margin="0 0 4px 0">
                  NBFC Gap: Manappuram doorstep app unavailable
                </Text>
                <Text fontSize="13px" color={kotakColors.textMuted} margin="0">
                  Competitors have scaled back doorstep services due to cost pressures — opportunity to differentiate.
                </Text>
              </Box>
            </FlexBox>
          </Box>
        </MotionBox>
      </FlexBox>

      <Notes>
        Doorstep service addresses key customer needs: convenience, security, accessibility for working professionals.
        5-city pilot: Mumbai (lead), Chennai, Hyderabad, Pune, Bangalore.
        Target: 5,500 customers in pilot phase. NBFC gap: Manappuram doorstep app unavailable.
      </Notes>
    </Slide>
  );
};

export default DoorstepOpportunitySlide;
