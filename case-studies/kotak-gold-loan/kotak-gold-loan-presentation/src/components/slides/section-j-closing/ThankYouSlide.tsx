import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const summaryMetrics = [
  { value: '425K', label: 'Customers', icon: '👥' },
  { value: 'Rs 8,500 Cr', label: 'AUM', icon: '📊' },
  { value: 'Rs 363 Cr', label: 'PAT', icon: '💰' },
  { value: '2.45x', label: 'ROI', icon: '📈' },
];

export const ThankYouSlide: React.FC = () => {
  return (
    <Slide backgroundColor={kotakColors.primary}>
      <FlexBox
        flexDirection="column"
        justifyContent="center"
        alignItems="center"
        height="100%"
        padding="40px 60px"
      >
        {/* Trophy Icon */}
        <MotionBox
          initial={{ scale: 0, rotate: -180 }}
          animate={{ scale: 1, rotate: 0 }}
          transition={{ duration: 0.8, type: 'spring' }}
        >
          <Text fontSize="80px" margin="0 0 24px 0">🏆</Text>
        </MotionBox>

        {/* Main Message */}
        <MotionBox
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.3 }}
        >
          <Heading fontSize="56px" fontWeight={700} color={kotakColors.white} margin="0 0 16px 0" textAlign="center">
            Let's Give Them a Better Choice
          </Heading>
        </MotionBox>

        {/* Subtitle */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.5 }}
        >
          <Text fontSize="24px" color={kotakColors.goldLight} margin="0 0 48px 0" textAlign="center">
            10 million customers deserve bank-grade security and fair rates
          </Text>
        </MotionBox>

        {/* Summary Metrics */}
        <MotionBox
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.7 }}
          style={{ width: '100%', maxWidth: '900px' }}
        >
          <FlexBox
            style={{
              background: 'rgba(255, 255, 255, 0.1)',
              borderRadius: '20px',
              padding: '24px 32px',
              backdropFilter: 'blur(10px)',
            }}
            justifyContent="space-around"
          >
            {summaryMetrics.map((metric, index) => (
              <MotionBox
                key={index}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.4, delay: 0.9 + index * 0.1 }}
                style={{ textAlign: 'center' }}
              >
                <Text fontSize="28px" margin="0 0 8px 0">{metric.icon}</Text>
                <Text fontSize="32px" fontWeight={700} color={kotakColors.white} margin="0">
                  {metric.value}
                </Text>
                <Text fontSize="14px" color="rgba(255, 255, 255, 0.8)" margin="4px 0 0 0">
                  {metric.label}
                </Text>
              </MotionBox>
            ))}
          </FlexBox>
        </MotionBox>

        {/* Divider */}
        <MotionBox
          initial={{ width: 0 }}
          animate={{ width: '120px' }}
          transition={{ duration: 0.8, delay: 1.3 }}
          style={{
            height: '3px',
            background: kotakColors.white,
            margin: '48px 0',
          }}
        />

        {/* Thank You */}
        <MotionBox
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.6, delay: 1.5 }}
        >
          <Text fontSize="28px" fontWeight={600} color={kotakColors.white} margin="0 0 8px 0">
            Thank You
          </Text>
          <Text fontSize="16px" color="rgba(255, 255, 255, 0.7)" margin="0">
            Questions & Discussion
          </Text>
        </MotionBox>

        {/* Kotak Logo */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 1.7 }}
          style={{ marginTop: '40px' }}
        >
          <FlexBox alignItems="center" style={{ gap: '12px' }}>
            <Box
              style={{
                width: '40px',
                height: '40px',
                borderRadius: '50%',
                background: kotakColors.white,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Text fontSize="18px" fontWeight={700} color={kotakColors.primary} margin="0">
                K
              </Text>
            </Box>
            <Text fontSize="18px" fontWeight={600} color={kotakColors.white} margin="0">
              KOTAK MAHINDRA BANK
            </Text>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Summary: 425K customers, Rs 8,500 Cr AUM, Rs 363 Cr PAT, 2.45x ROI over 3 years.
        Core message: 10 million customers deserve better. Let's give them that choice.
        Open for questions and discussion.
      </Notes>
    </Slide>
  );
};

export default ThankYouSlide;
