import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { campaignConcepts } from '../../../data/productData';

const MotionBox = motion(Box);

export const CampaignConceptsSlide: React.FC = () => {
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
            MARKETING CAMPAIGNS
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Four Campaign Concepts
          </Heading>
        </MotionBox>

        {/* Campaign Grid */}
        <FlexBox flex={1} style={{ gap: '16px' }} flexWrap="wrap">
          {campaignConcepts.map((campaign, index) => (
            <MotionBox
              key={index}
              initial={{ opacity: 0, y: 20, scale: 0.95 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              transition={{ duration: 0.4, delay: 0.2 + index * 0.1 }}
              style={{
                flex: '1 1 calc(50% - 8px)',
                minWidth: '400px',
                background: kotakColors.darkCard,
                borderRadius: '16px',
                padding: '24px',
                borderLeft: `4px solid ${campaign.color}`,
              }}
            >
              <FlexBox alignItems="flex-start" style={{ gap: '16px' }}>
                <Box
                  style={{
                    width: '56px',
                    height: '56px',
                    borderRadius: '14px',
                    background: `${campaign.color}20`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    flexShrink: 0,
                  }}
                >
                  <Text fontSize="28px" margin="0">{campaign.icon}</Text>
                </Box>
                <Box style={{ flex: 1 }}>
                  <Text fontSize="18px" fontWeight={700} color={kotakColors.white} margin="0 0 4px 0">
                    {campaign.name}
                  </Text>
                  <Box
                    style={{
                      display: 'inline-block',
                      padding: '4px 10px',
                      background: `${campaign.color}20`,
                      borderRadius: '4px',
                      marginBottom: '12px',
                    }}
                  >
                    <Text fontSize="12px" fontWeight={600} color={campaign.color} margin="0">
                      {campaign.segment}
                    </Text>
                  </Box>

                  {/* Tagline */}
                  <Box
                    style={{
                      padding: '12px 16px',
                      background: 'rgba(255,255,255,0.03)',
                      borderRadius: '8px',
                      marginBottom: '12px',
                    }}
                  >
                    <Text fontSize="15px" fontWeight={600} color={campaign.color} margin="0" style={{ fontStyle: 'italic' }}>
                      "{campaign.tagline}"
                    </Text>
                  </Box>

                  {/* Channels */}
                  <FlexBox style={{ gap: '6px' }} flexWrap="wrap">
                    {campaign.channels.map((channel, i) => (
                      <Box
                        key={i}
                        style={{
                          padding: '4px 8px',
                          background: 'rgba(255,255,255,0.05)',
                          borderRadius: '4px',
                        }}
                      >
                        <Text fontSize="11px" color={kotakColors.textMuted} margin="0">{channel}</Text>
                      </Box>
                    ))}
                  </FlexBox>
                </Box>
              </FlexBox>
            </MotionBox>
          ))}
        </FlexBox>

        {/* Campaign Timeline */}
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
              padding: '16px 24px',
            }}
            justifyContent="space-around"
            alignItems="center"
          >
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">Q1 Launch</Text>
              <Text fontSize="16px" fontWeight={700} color={kotakColors.primary} margin="0">Smart Switch + Gold Deserves Bank</Text>
            </Box>
            <Text fontSize="20px" color={kotakColors.textMuted} margin="0">→</Text>
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">Q2 Expansion</Text>
              <Text fontSize="16px" fontWeight={700} color="#EC4899" margin="0">+ Shakti Gold Loan</Text>
            </Box>
            <Text fontSize="20px" color={kotakColors.textMuted} margin="0">→</Text>
            <Box style={{ textAlign: 'center' }}>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 4px 0">Q3 Digital Push</Text>
              <Text fontSize="16px" fontWeight={700} color="#8B5CF6" margin="0">+ Gold in 3 Clicks</Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Four campaign concepts targeting different segments:
        1. Smart Switch - savings focus for P1
        2. Gold Deserves Bank - trust focus for P2
        3. Shakti Gold Loan - women empowerment for P3
        4. Gold in 3 Clicks - digital convenience for P4
        Phased rollout: Q1 core, Q2 women, Q3 digital.
      </Notes>
    </Slide>
  );
};

export default CampaignConceptsSlide;
