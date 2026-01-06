import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { lakshmiPersona } from '../../../data/customerQuotes';

const MotionBox = motion(Box);

export const CustomerStorySlide: React.FC = () => {
  const metrics = [
    { label: 'Business', value: lakshmiPersona.business, color: kotakColors.textPrimary },
    { label: 'Current Rate', value: `${lakshmiPersona.currentRate}% p.a.`, color: kotakColors.danger },
    { label: 'Annual Interest', value: `Rs ${lakshmiPersona.annualInterest.toLocaleString()}`, color: kotakColors.danger },
    { label: 'Potential Savings', value: `Rs ${lakshmiPersona.potentialSavings.toLocaleString()}/yr`, color: kotakColors.success },
  ];

  return (
    <Slide backgroundColor={kotakColors.dark}>
      <FlexBox flexDirection="column" height="100%" padding="40px 60px">
        {/* Section Label */}
        <MotionBox
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.4 }}
        >
          <Text
            fontSize="14px"
            fontWeight={600}
            color={kotakColors.primary}
            margin="0 0 8px 0"
            style={{ letterSpacing: '2px', textTransform: 'uppercase' }}
          >
            THE CUSTOMER STORY
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 40px 0">
            Meet {lakshmiPersona.name}: A {lakshmiPersona.title}
          </Heading>
        </MotionBox>

        {/* Content */}
        <FlexBox flex={1} alignItems="center" style={{ gap: '60px' }}>
          {/* Avatar and Quote */}
          <MotionBox
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            style={{ flex: '0 0 auto' }}
          >
            <Box
              style={{
                width: '140px',
                height: '140px',
                borderRadius: '50%',
                background: `linear-gradient(135deg, ${kotakColors.gold}30, ${kotakColors.gold}10)`,
                border: `3px solid ${kotakColors.gold}`,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Text fontSize="72px" margin="0">{lakshmiPersona.avatar}</Text>
            </Box>
          </MotionBox>

          {/* Quote and Metrics */}
          <FlexBox flexDirection="column" flex={1}>
            {/* Quote */}
            <MotionBox
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.5, delay: 0.3 }}
              style={{
                background: `linear-gradient(90deg, ${kotakColors.darkCard}, transparent)`,
                borderLeft: `4px solid ${kotakColors.gold}`,
                padding: '24px 32px',
                marginBottom: '32px',
              }}
            >
              <Text
                fontSize="24px"
                color={kotakColors.white}
                margin="0"
                style={{ fontStyle: 'italic', lineHeight: 1.5 }}
              >
                "{lakshmiPersona.quote}"
              </Text>
            </MotionBox>

            {/* Metrics */}
            <FlexBox style={{ gap: '16px' }}>
              {metrics.map((metric, index) => (
                <MotionBox
                  key={index}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.4, delay: 0.4 + index * 0.1 }}
                  style={{
                    flex: 1,
                    background: kotakColors.darkCard,
                    borderRadius: '12px',
                    padding: '20px',
                    textAlign: 'center',
                  }}
                >
                  <Text fontSize="13px" color={kotakColors.textMuted} margin="0 0 8px 0">
                    {metric.label}
                  </Text>
                  <Text fontSize="22px" fontWeight={700} color={metric.color} margin="0">
                    {metric.value}
                  </Text>
                </MotionBox>
              ))}
            </FlexBox>
          </FlexBox>
        </FlexBox>

        {/* Bottom Insight */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.8 }}
          style={{ marginTop: '32px', textAlign: 'center' }}
        >
          <Text fontSize="20px" color={kotakColors.textMuted} margin="0">
            There are{' '}
            <Text fontSize="20px" fontWeight={700} color={kotakColors.gold} style={{ display: 'inline' }}>
              millions of Lakshmis
            </Text>{' '}
            across India. Today's strategy is about reaching them.
          </Text>
        </MotionBox>
      </FlexBox>

      <Notes>
        This is Lakshmi - she represents millions of small business owners across India
        who are paying 40-50% more than they need to for their gold loans. She's been
        with the same NBFC for years, not because she's loyal, but because switching
        seems complicated. Rs 40,000 per year in savings is life-changing for her business.
      </Notes>
    </Slide>
  );
};

export default CustomerStorySlide;
