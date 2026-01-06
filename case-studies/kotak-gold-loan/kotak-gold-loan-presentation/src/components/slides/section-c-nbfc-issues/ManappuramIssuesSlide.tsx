import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { manappuramFraudCases, manappuramRBIPenalties, manappuramServiceIssues } from '../../../data/competitorData';

const MotionBox = motion(Box);

export const ManappuramIssuesSlide: React.FC = () => {
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
            color={kotakColors.danger}
            margin="0 0 8px 0"
            style={{ letterSpacing: '2px', textTransform: 'uppercase' }}
          >
            COMPETITOR ANALYSIS: MANAPPURAM
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Manappuram: A Pattern of Issues
          </Heading>
        </MotionBox>

        <FlexBox flex={1} style={{ gap: '24px' }}>
          {/* Left: Fraud Cases Timeline */}
          <MotionBox
            initial={{ opacity: 0, x: -30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            style={{ flex: 1.2 }}
          >
            <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0 0 16px 0">
              Fraud Cases Timeline
            </Text>
            <FlexBox flexDirection="column" style={{ gap: '12px' }}>
              {manappuramFraudCases.map((fraud, index) => (
                <MotionBox
                  key={index}
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.4, delay: 0.3 + index * 0.1 }}
                  style={{
                    background: kotakColors.darkCard,
                    borderRadius: '10px',
                    padding: '16px 20px',
                    borderLeft: `3px solid ${kotakColors.danger}`,
                  }}
                >
                  <FlexBox justifyContent="space-between" alignItems="flex-start">
                    <Box>
                      <FlexBox style={{ gap: '12px', marginBottom: '6px' }} alignItems="center">
                        <Text fontSize="12px" fontWeight={600} color={kotakColors.danger} margin="0">
                          {fraud.date}
                        </Text>
                        <Text fontSize="12px" color={kotakColors.textMuted} margin="0">
                          {fraud.location}
                        </Text>
                      </FlexBox>
                      <Text fontSize="15px" color={kotakColors.white} margin="0 0 4px 0">
                        {fraud.type}
                      </Text>
                    </Box>
                    <Box
                      style={{
                        background: `${kotakColors.danger}20`,
                        padding: '6px 12px',
                        borderRadius: '6px',
                      }}
                    >
                      <Text fontSize="14px" fontWeight={700} color={kotakColors.danger} margin="0">
                        Rs {fraud.amount}
                      </Text>
                    </Box>
                  </FlexBox>
                </MotionBox>
              ))}
            </FlexBox>
          </MotionBox>

          {/* Right: RBI Penalties & Service Issues */}
          <MotionBox
            initial={{ opacity: 0, x: 30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            style={{ flex: 1 }}
          >
            {/* RBI Penalties */}
            <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0 0 16px 0">
              RBI Penalties
            </Text>
            <FlexBox flexDirection="column" style={{ gap: '12px', marginBottom: '24px' }}>
              {manappuramRBIPenalties.map((penalty, index) => (
                <Box
                  key={index}
                  style={{
                    background: kotakColors.darkCard,
                    borderRadius: '10px',
                    padding: '16px 20px',
                    borderLeft: `3px solid ${kotakColors.gold}`,
                  }}
                >
                  <FlexBox justifyContent="space-between" alignItems="center">
                    <Box>
                      <Text fontSize="12px" fontWeight={600} color={kotakColors.gold} margin="0 0 4px 0">
                        {penalty.date}
                      </Text>
                      <Text fontSize="14px" color={kotakColors.textMuted} margin="0">
                        {penalty.violation}
                      </Text>
                    </Box>
                    <Text fontSize="16px" fontWeight={700} color={kotakColors.gold} margin="0">
                      Rs {penalty.amount}
                    </Text>
                  </FlexBox>
                </Box>
              ))}
            </FlexBox>

            {/* Service Issues */}
            <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0 0 16px 0">
              Service Disruptions
            </Text>
            <FlexBox flexDirection="column" style={{ gap: '10px' }}>
              {manappuramServiceIssues.map((item, index) => (
                <Box
                  key={index}
                  style={{
                    background: kotakColors.darkCard,
                    borderRadius: '10px',
                    padding: '14px 18px',
                    borderLeft: `3px solid ${kotakColors.warning}`,
                  }}
                >
                  <FlexBox alignItems="center" style={{ gap: '12px' }}>
                    <Text fontSize="20px" margin="0">⚠️</Text>
                    <Box>
                      <Text fontSize="15px" fontWeight={600} color={kotakColors.white} margin="0 0 4px 0">
                        {item.issue}
                      </Text>
                      <Text fontSize="13px" color={kotakColors.textMuted} margin="0">
                        {item.description}
                      </Text>
                    </Box>
                  </FlexBox>
                </Box>
              ))}
            </FlexBox>
          </MotionBox>
        </FlexBox>

        {/* Bottom Summary */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.8 }}
          style={{ marginTop: '20px' }}
        >
          <Box
            style={{
              background: `linear-gradient(90deg, ${kotakColors.danger}15, transparent)`,
              borderRadius: '12px',
              padding: '16px 24px',
              border: `1px solid ${kotakColors.danger}30`,
            }}
          >
            <FlexBox alignItems="center" style={{ gap: '16px' }}>
              <Text fontSize="32px" margin="0">📉</Text>
              <Text fontSize="17px" color={kotakColors.white} margin="0">
                <Text fontSize="17px" fontWeight={700} color={kotakColors.danger} style={{ display: 'inline' }}>
                  Total Fraud Impact: Rs 69+ Crore
                </Text>
                {' '}— Multiple fraud cases, RBI penalties, and service disruptions are eroding Manappuram's customer trust.
              </Text>
            </FlexBox>
          </Box>
        </MotionBox>
      </FlexBox>

      <Notes>
        Manappuram has faced multiple fraud cases: Rs 43L in UP, Rs 20Cr tech fraud by AGM in Thrissur,
        Rs 5.5Cr Bhopal betting scam, Rs 70L Odisha fake gold. Add RBI penalties for NPA misclassification
        and ownership verification failures. Their Asirvad subsidiary was also barred from lending.
      </Notes>
    </Slide>
  );
};

export default ManappuramIssuesSlide;
