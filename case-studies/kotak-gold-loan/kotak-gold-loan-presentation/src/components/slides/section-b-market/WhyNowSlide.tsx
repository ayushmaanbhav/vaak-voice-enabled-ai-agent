import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const timelineEvents = [
  {
    date: 'March 2024',
    event: 'IIFL Gold Loan Ban',
    description: 'RBI bars IIFL from gold loan operations due to LTV breaches, cash violations, improper valuations',
    impact: '5.5M+ customers suddenly uncertain',
    icon: '🚫',
    color: kotakColors.danger,
  },
  {
    date: 'Sept 2024',
    event: 'New RBI Circular',
    description: 'Stricter gold loan regulations announced - NBFCs face compliance burden',
    impact: 'Level playing field for banks',
    icon: '📋',
    color: kotakColors.gold,
  },
  {
    date: 'Oct 2024',
    event: 'Asirvad Embargo',
    description: 'Manappuram subsidiary barred from lending - trust crisis deepens',
    impact: 'NBFC reputation further damaged',
    icon: '⚠️',
    color: kotakColors.warning,
  },
];

export const WhyNowSlide: React.FC = () => {
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
            TIMING
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 40px 0">
            Why Now? The Perfect Storm
          </Heading>
        </MotionBox>

        {/* Timeline */}
        <FlexBox flex={1} alignItems="center" style={{ position: 'relative' }}>
          {/* Timeline Line */}
          <MotionBox
            initial={{ scaleX: 0 }}
            animate={{ scaleX: 1 }}
            transition={{ duration: 0.8, delay: 0.2 }}
            style={{
              position: 'absolute',
              top: '50%',
              left: '5%',
              right: '5%',
              height: '4px',
              background: `linear-gradient(90deg, ${kotakColors.danger}, ${kotakColors.gold}, ${kotakColors.warning})`,
              borderRadius: '2px',
              transformOrigin: 'left',
            }}
          />

          {/* Timeline Events */}
          <FlexBox width="100%" justifyContent="space-around" style={{ position: 'relative', zIndex: 1 }}>
            {timelineEvents.map((event, index) => (
              <MotionBox
                key={index}
                initial={{ opacity: 0, y: 30 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: 0.4 + index * 0.2 }}
                style={{
                  width: '280px',
                  textAlign: 'center',
                }}
              >
                {/* Event Circle */}
                <Box
                  style={{
                    width: '64px',
                    height: '64px',
                    borderRadius: '50%',
                    background: kotakColors.dark,
                    border: `4px solid ${event.color}`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    margin: '0 auto 16px',
                  }}
                >
                  <Text fontSize="28px" margin="0">{event.icon}</Text>
                </Box>

                {/* Event Card */}
                <Box
                  style={{
                    background: kotakColors.darkCard,
                    borderRadius: '12px',
                    padding: '20px',
                    borderTop: `3px solid ${event.color}`,
                  }}
                >
                  <Text fontSize="13px" fontWeight={600} color={event.color} margin="0 0 8px 0">
                    {event.date}
                  </Text>
                  <Text fontSize="18px" fontWeight={700} color={kotakColors.white} margin="0 0 8px 0">
                    {event.event}
                  </Text>
                  <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 12px 0" style={{ lineHeight: 1.4 }}>
                    {event.description}
                  </Text>
                  <Box
                    style={{
                      background: `${event.color}15`,
                      borderRadius: '6px',
                      padding: '8px 12px',
                    }}
                  >
                    <Text fontSize="13px" fontWeight={600} color={event.color} margin="0">
                      {event.impact}
                    </Text>
                  </Box>
                </Box>
              </MotionBox>
            ))}
          </FlexBox>
        </FlexBox>

        {/* Bottom Insight */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 1.2 }}
          style={{
            marginTop: '32px',
            padding: '20px 28px',
            background: `linear-gradient(90deg, ${kotakColors.primary}15, ${kotakColors.primary}05)`,
            borderRadius: '12px',
            border: `1px solid ${kotakColors.primary}30`,
          }}
        >
          <FlexBox alignItems="center" justifyContent="center" style={{ gap: '16px' }}>
            <Text fontSize="36px" margin="0">🎯</Text>
            <Text fontSize="20px" fontWeight={600} color={kotakColors.white} margin="0">
              Window of opportunity: NBFC customers are actively looking for alternatives
            </Text>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        The timing is perfect for acquisition. IIFL ban created 5.5M uncertain customers.
        New RBI regulations are leveling the playing field for banks.
        Manappuram's Asirvad subsidiary embargo further damaged NBFC reputation.
        Customers are actively seeking safer alternatives - we need to capture this momentum.
      </Notes>
    </Slide>
  );
};

export default WhyNowSlide;
