import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const processSteps = [
  {
    step: 1,
    title: 'Pre-Approval',
    desc: 'Upload old loan details, get instant pre-approval on app',
    icon: '📱',
    time: '5 mins',
  },
  {
    step: 2,
    title: 'Document Collection',
    desc: 'Dedicated RM collects all docs from you',
    icon: '📋',
    time: '1 day',
  },
  {
    step: 3,
    title: 'Old Loan Closure',
    desc: 'We coordinate with NBFC and close your loan',
    icon: '🤝',
    time: '1 day',
  },
  {
    step: 4,
    title: 'Secure Transfer',
    desc: 'Insured gold transit to Kotak branch',
    icon: '🔒',
    time: '1 day',
  },
];

const services = [
  { icon: '👤', text: 'Dedicated Switch Manager assigned' },
  { icon: '📞', text: 'Single point of contact throughout' },
  { icon: '🚗', text: 'Doorstep document pickup' },
  { icon: '📦', text: 'Rs 10L transit insurance included' },
  { icon: '💬', text: 'Real-time status updates on app' },
];

export const SwitchAssistSlide: React.FC = () => {
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
            SERVICE DIFFERENTIATOR
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Switch Assist: Concierge Service
          </Heading>
        </MotionBox>

        <FlexBox flex={1} style={{ gap: '32px' }}>
          {/* Left: Process Flow */}
          <MotionBox
            initial={{ opacity: 0, x: -30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            style={{ flex: 1.3 }}
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
                4-Step Hassle-Free Process
              </Text>

              <FlexBox flexDirection="column" style={{ gap: '12px', position: 'relative' }}>
                {/* Connecting Line */}
                <Box
                  style={{
                    position: 'absolute',
                    left: '26px',
                    top: '40px',
                    bottom: '40px',
                    width: '2px',
                    background: `linear-gradient(180deg, ${kotakColors.primary}, ${kotakColors.success})`,
                  }}
                />

                {processSteps.map((step, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ duration: 0.4, delay: 0.3 + index * 0.1 }}
                    style={{
                      background: 'rgba(255,255,255,0.03)',
                      borderRadius: '12px',
                      padding: '16px 20px',
                      position: 'relative',
                      zIndex: 1,
                    }}
                  >
                    <FlexBox alignItems="center" style={{ gap: '16px' }}>
                      <Box
                        style={{
                          width: '52px',
                          height: '52px',
                          borderRadius: '50%',
                          background: kotakColors.darkCard,
                          border: `3px solid ${kotakColors.primary}`,
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                          flexShrink: 0,
                        }}
                      >
                        <Text fontSize="24px" margin="0">{step.icon}</Text>
                      </Box>
                      <Box style={{ flex: 1 }}>
                        <FlexBox alignItems="center" style={{ gap: '8px' }}>
                          <Text fontSize="12px" fontWeight={700} color={kotakColors.primary} margin="0">
                            STEP {step.step}
                          </Text>
                          <Box style={{ padding: '2px 8px', background: `${kotakColors.success}20`, borderRadius: '4px' }}>
                            <Text fontSize="11px" fontWeight={600} color={kotakColors.success} margin="0">
                              {step.time}
                            </Text>
                          </Box>
                        </FlexBox>
                        <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="4px 0 2px 0">
                          {step.title}
                        </Text>
                        <Text fontSize="13px" color={kotakColors.textMuted} margin="0">
                          {step.desc}
                        </Text>
                      </Box>
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>
            </Box>
          </MotionBox>

          {/* Right: Service Features */}
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
                What's Included
              </Text>

              <FlexBox flexDirection="column" style={{ gap: '12px' }}>
                {services.map((service, index) => (
                  <MotionBox
                    key={index}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.3, delay: 0.5 + index * 0.08 }}
                  >
                    <FlexBox
                      alignItems="center"
                      style={{
                        gap: '14px',
                        padding: '14px 16px',
                        background: `${kotakColors.success}08`,
                        borderRadius: '10px',
                        border: `1px solid ${kotakColors.success}20`,
                      }}
                    >
                      <Text fontSize="22px" margin="0">{service.icon}</Text>
                      <Text fontSize="15px" color={kotakColors.white} margin="0">
                        {service.text}
                      </Text>
                    </FlexBox>
                  </MotionBox>
                ))}
              </FlexBox>

              {/* Key Differentiator */}
              <Box style={{ marginTop: '20px', padding: '16px', background: `${kotakColors.primary}15`, borderRadius: '10px' }}>
                <Text fontSize="14px" fontWeight={600} color={kotakColors.primary} margin="0 0 8px 0">
                  WHY IT MATTERS
                </Text>
                <Text fontSize="14px" color={kotakColors.textMuted} margin="0" style={{ lineHeight: 1.5 }}>
                  NBFCs don't offer any switching support. Customers are on their own to
                  navigate the complex process. Switch Assist removes all friction.
                </Text>
              </Box>
            </Box>
          </MotionBox>
        </FlexBox>

        {/* Bottom Metric */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.8 }}
          style={{ marginTop: '16px' }}
        >
          <FlexBox
            style={{
              background: kotakColors.darkCard,
              borderRadius: '12px',
              padding: '16px 24px',
            }}
            justifyContent="space-around"
            alignItems="center"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="28px" fontWeight={700} color={kotakColors.gold} margin="0">3 Days</Text>
              <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">Total Time</Text>
            </Box>
            <Box style={{ width: '1px', height: '40px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="28px" fontWeight={700} color={kotakColors.success} margin="0">1 Visit</Text>
              <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">Branch Visit Only</Text>
            </Box>
            <Box style={{ width: '1px', height: '40px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="28px" fontWeight={700} color={kotakColors.primary} margin="0">0 Hassle</Text>
              <Text fontSize="13px" color={kotakColors.textMuted} margin="4px 0 0 0">We Handle Everything</Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Switch Assist is our concierge service - dedicated manager, doorstep pickup,
        NBFC coordination, insured transit. Total process: 3 days, 1 branch visit.
        This is a major differentiator - NBFCs don't offer any switching support.
      </Notes>
    </Slide>
  );
};

export default SwitchAssistSlide;
