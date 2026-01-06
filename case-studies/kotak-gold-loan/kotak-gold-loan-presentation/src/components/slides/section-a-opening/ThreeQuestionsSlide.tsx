import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const questions = [
  {
    number: '01',
    question: 'Why are 10+ million customers paying 40-50% more than they need to?',
    color: kotakColors.primary,
  },
  {
    number: '02',
    question: 'How can we convert 50,000 of them in Year 1?',
    color: kotakColors.gold,
  },
  {
    number: '03',
    question: "What's the ROI — and why now?",
    color: kotakColors.success,
  },
];

export const ThreeQuestionsSlide: React.FC = () => {
  return (
    <Slide backgroundColor={kotakColors.dark}>
      <FlexBox flexDirection="column" height="100%" padding="40px 60px">
        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
        >
          <Heading fontSize="48px" fontWeight={700} color={kotakColors.white} margin="0 0 48px 0">
            Three Questions We'll Answer Today
          </Heading>
        </MotionBox>

        {/* Questions */}
        <FlexBox flexDirection="column" flex={1} justifyContent="center" style={{ gap: '24px', maxWidth: '900px' }}>
          {questions.map((q, index) => (
            <MotionBox
              key={index}
              initial={{ opacity: 0, x: -40 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.5, delay: 0.2 + index * 0.2 }}
              style={{
                background: kotakColors.darkCard,
                borderRadius: '16px',
                padding: '28px 32px',
                borderLeft: `5px solid ${q.color}`,
                marginLeft: index * 40,
              }}
            >
              <FlexBox alignItems="center" style={{ gap: '20px' }}>
                <Box
                  style={{
                    width: '48px',
                    height: '48px',
                    borderRadius: '50%',
                    background: `${q.color}20`,
                    border: `2px solid ${q.color}`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  <Text fontSize="18px" fontWeight={700} color={q.color} margin="0">
                    {q.number}
                  </Text>
                </Box>
                <Text fontSize="22px" fontWeight={500} color={kotakColors.white} margin="0">
                  {q.question}
                </Text>
              </FlexBox>
            </MotionBox>
          ))}
        </FlexBox>

        {/* Bottom Teaser */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 1 }}
          style={{
            marginTop: '40px',
            padding: '20px 28px',
            background: 'rgba(255, 255, 255, 0.03)',
            borderRadius: '12px',
            textAlign: 'center',
          }}
        >
          <Text fontSize="18px" color={kotakColors.textMuted} margin="0">
            By the end, you'll see why this is a{' '}
            <Text fontSize="18px" fontWeight={700} color={kotakColors.gold} style={{ display: 'inline' }}>
              Rs 363 crore PAT opportunity
            </Text>{' '}
            over three years — with minimal risk.
          </Text>
        </MotionBox>
      </FlexBox>

      <Notes>
        These are the three questions that guide our entire strategy. First, understanding
        the market dynamics. Second, our acquisition approach. Third, the business case.
        Let's dive in.
      </Notes>
    </Slide>
  );
};

export default ThreeQuestionsSlide;
