import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { mouthshutReviews } from '../../../data/customerQuotes';

const MotionBox = motion(Box);

export const CustomerPainPointsSlide: React.FC = () => {
  // Select 4 impactful reviews
  const selectedReviews = mouthshutReviews.slice(0, 4);

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
            THE PROBLEM: CUSTOMER PAIN
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 8px 0">
            Real Voices: What Customers Are Saying
          </Heading>
          <Text fontSize="18px" color={kotakColors.textMuted} margin="0 0 32px 0">
            MouthShut reviews reveal deep frustration with NBFC gold loan services
          </Text>
        </MotionBox>

        {/* Review Cards */}
        <FlexBox flex={1} style={{ gap: '20px' }} flexWrap="wrap">
          {selectedReviews.map((review, index) => (
            <MotionBox
              key={index}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.2 + index * 0.1 }}
              style={{
                flex: '1 1 calc(50% - 10px)',
                minWidth: '400px',
                background: kotakColors.darkCard,
                borderRadius: '12px',
                padding: '24px',
                borderLeft: `4px solid ${kotakColors.danger}`,
              }}
            >
              {/* Header */}
              <FlexBox justifyContent="space-between" alignItems="center" style={{ marginBottom: '12px' }}>
                <FlexBox alignItems="center" style={{ gap: '8px' }}>
                  <Box
                    style={{
                      padding: '4px 10px',
                      background: kotakColors.danger + '20',
                      borderRadius: '4px',
                    }}
                  >
                    <Text fontSize="12px" fontWeight={600} color={kotakColors.danger} margin="0">
                      {review.provider}
                    </Text>
                  </Box>
                  <Text fontSize="12px" color={kotakColors.textMuted} margin="0">
                    {review.date}
                  </Text>
                </FlexBox>
                <FlexBox alignItems="center" style={{ gap: '4px' }}>
                  <Text fontSize="14px" margin="0">⭐</Text>
                  <Text fontSize="14px" fontWeight={600} color={kotakColors.gold} margin="0">
                    {review.rating}/{review.maxRating}
                  </Text>
                </FlexBox>
              </FlexBox>

              {/* Title */}
              <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0 0 8px 0">
                {review.title}
              </Text>

              {/* Quote */}
              <Text
                fontSize="15px"
                color={kotakColors.textMuted}
                margin="0 0 12px 0"
                style={{ fontStyle: 'italic', lineHeight: 1.5 }}
              >
                {review.quote}
              </Text>

              {/* Pain Point Tag */}
              <Box
                style={{
                  padding: '4px 10px',
                  background: 'rgba(255,255,255,0.05)',
                  borderRadius: '4px',
                  border: '1px solid rgba(255,255,255,0.1)',
                  display: 'inline-block',
                }}
              >
                <Text fontSize="12px" color={kotakColors.textMuted} margin="0">
                  {review.painPoint}
                </Text>
              </Box>
            </MotionBox>
          ))}
        </FlexBox>

        {/* Bottom Stat */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.7 }}
          style={{ marginTop: '20px' }}
        >
          <FlexBox
            style={{
              background: `linear-gradient(90deg, ${kotakColors.danger}15, transparent)`,
              borderRadius: '12px',
              padding: '16px 24px',
              border: `1px solid ${kotakColors.danger}30`,
            }}
            justifyContent="space-between"
            alignItems="center"
          >
            <FlexBox alignItems="center" style={{ gap: '12px' }}>
              <Text fontSize="28px" margin="0">😤</Text>
              <Box>
                <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 2px 0">
                  Muthoot Finance MouthShut Rating
                </Text>
                <Text fontSize="22px" fontWeight={700} color={kotakColors.danger} margin="0">
                  2.19 / 5 stars
                </Text>
              </Box>
            </FlexBox>
            <Box style={{ textAlign: 'right' }}>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0 0 2px 0">
                Top Customer Complaints
              </Text>
              <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0">
                Penalties • App Issues • Dismissive Staff • Renewal Hassles
              </Text>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        These are real reviews from MouthShut - India's leading consumer review platform.
        Muthoot has a 2.19/5 rating. Common complaints: excessive penalties, app issues,
        dismissive staff, and renewal problems. This is opportunity for us.
      </Notes>
    </Slide>
  );
};

export default CustomerPainPointsSlide;
