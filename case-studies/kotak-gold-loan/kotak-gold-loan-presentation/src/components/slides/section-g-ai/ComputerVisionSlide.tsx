import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const processSteps = [
  { step: 1, title: 'App Capture', desc: 'Customer photographs gold using Kotak app', icon: '📱', time: '2 mins' },
  { step: 2, title: 'AI Analysis', desc: 'YOLO v8 detects items, estimates purity', icon: '🤖', time: '10 secs' },
  { step: 3, title: 'Pre-Approval', desc: 'Instant indicative valuation shown', icon: '✅', time: '30 secs' },
  { step: 4, title: 'Branch Verification', desc: 'XRF confirmation + final disbursement', icon: '🏦', time: '45 mins' },
];

export const ComputerVisionSlide: React.FC = () => {
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
            AI SOLUTION #1
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Computer Vision Gold Appraisal
          </Heading>
        </MotionBox>

        {/* Process Flow */}
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
            <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0 0 20px 0">
              4-Step Process: App to Disbursement in 90 Minutes
            </Text>

            <FlexBox style={{ gap: '16px', position: 'relative' }}>
              {/* Connecting Line */}
              <Box
                style={{
                  position: 'absolute',
                  top: '35px',
                  left: '70px',
                  right: '70px',
                  height: '3px',
                  background: `linear-gradient(90deg, ${kotakColors.primary}, ${kotakColors.success})`,
                }}
              />

              {processSteps.map((step, index) => (
                <MotionBox
                  key={index}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.4, delay: 0.3 + index * 0.1 }}
                  style={{ flex: 1, textAlign: 'center', position: 'relative', zIndex: 1 }}
                >
                  <Box
                    style={{
                      width: '70px',
                      height: '70px',
                      borderRadius: '50%',
                      background: kotakColors.dark,
                      border: `3px solid ${kotakColors.primary}`,
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      margin: '0 auto 12px',
                    }}
                  >
                    <Text fontSize="32px" margin="0">{step.icon}</Text>
                  </Box>
                  <Box style={{ padding: '2px 8px', background: `${kotakColors.success}20`, borderRadius: '4px', display: 'inline-block', marginBottom: '8px' }}>
                    <Text fontSize="11px" fontWeight={600} color={kotakColors.success} margin="0">{step.time}</Text>
                  </Box>
                  <Text fontSize="15px" fontWeight={600} color={kotakColors.white} margin="0 0 4px 0">{step.title}</Text>
                  <Text fontSize="12px" color={kotakColors.textMuted} margin="0">{step.desc}</Text>
                </MotionBox>
              ))}
            </FlexBox>
          </Box>
        </MotionBox>

        {/* Technical Specs & Benefits */}
        <FlexBox flex={1} style={{ gap: '20px' }}>
          {/* Technical Specs */}
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
                Technical Specifications
              </Text>
              <FlexBox flexDirection="column" style={{ gap: '10px' }}>
                {[
                  { label: 'Model', value: 'YOLO v8 + Custom Training' },
                  { label: 'Purity Detection', value: '18K, 22K, 24K classification' },
                  { label: 'Accuracy', value: '97% for item detection' },
                  { label: 'Final Verification', value: 'XRF spectrometry at branch' },
                  { label: 'Training Data', value: '50,000+ gold images' },
                ].map((spec, i) => (
                  <FlexBox key={i} justifyContent="space-between" style={{ padding: '8px 12px', background: 'rgba(255,255,255,0.03)', borderRadius: '6px' }}>
                    <Text fontSize="13px" color={kotakColors.textMuted} margin="0">{spec.label}</Text>
                    <Text fontSize="13px" fontWeight={500} color={kotakColors.white} margin="0">{spec.value}</Text>
                  </FlexBox>
                ))}
              </FlexBox>
            </Box>
          </MotionBox>

          {/* Benefits */}
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
                Business Benefits
              </Text>
              <FlexBox flexDirection="column" style={{ gap: '10px' }}>
                {[
                  { benefit: 'Pre-qualified customers before branch visit', icon: '✅' },
                  { benefit: '60% reduction in branch processing time', icon: '⏱️' },
                  { benefit: 'Enables doorstep service model', icon: '🏠' },
                  { benefit: 'Reduces fraud via standardized detection', icon: '🔒' },
                  { benefit: 'Better customer experience: know before you go', icon: '😊' },
                ].map((item, i) => (
                  <FlexBox key={i} alignItems="center" style={{ gap: '10px', padding: '8px 12px', background: `${kotakColors.success}08`, borderRadius: '6px' }}>
                    <Text fontSize="18px" margin="0">{item.icon}</Text>
                    <Text fontSize="13px" color={kotakColors.white} margin="0">{item.benefit}</Text>
                  </FlexBox>
                ))}
              </FlexBox>
            </Box>
          </MotionBox>
        </FlexBox>

        {/* Investment & Timeline */}
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
              padding: '14px 24px',
            }}
            justifyContent="space-around"
            alignItems="center"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="20px" fontWeight={700} color={kotakColors.gold} margin="0">Rs 6 Cr</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Investment</Text>
            </Box>
            <Box style={{ width: '1px', height: '35px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="20px" fontWeight={700} color={kotakColors.success} margin="0">12 Months</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Development</Text>
            </Box>
            <Box style={{ width: '1px', height: '35px', background: 'rgba(255,255,255,0.1)' }} />
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="20px" fontWeight={700} color={kotakColors.primary} margin="0">2.8x ROI</Text>
              <Text fontSize="11px" color={kotakColors.textMuted} margin="2px 0 0 0">Expected Return</Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Computer Vision appraisal uses YOLO v8 for gold detection with 97% accuracy.
        Process: App capture → AI analysis → Pre-approval → Branch XRF verification.
        Total time: 90 minutes vs 4+ hours traditional. Investment: Rs 6 Cr, 2.8x ROI.
      </Notes>
    </Slide>
  );
};

export default ComputerVisionSlide;
