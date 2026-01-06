import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);
const MotionHeading = motion(Heading);

export const TitleSlide: React.FC = () => {
  return (
    <Slide backgroundColor={kotakColors.primary}>
      <FlexBox
        flexDirection="column"
        justifyContent="center"
        alignItems="center"
        height="100%"
      >
        {/* Trophy Icon */}
        <MotionBox
          initial={{ scale: 0, rotate: -180 }}
          animate={{ scale: 1, rotate: 0 }}
          transition={{ duration: 0.8, type: 'spring' }}
        >
          <Text fontSize="72px" margin="0 0 24px 0">
            🏆
          </Text>
        </MotionBox>

        {/* Main Title */}
        <MotionHeading
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.3 }}
          fontSize="64px"
          fontWeight={700}
          color={kotakColors.white}
          margin="0 0 16px 0"
        >
          Switch & Save
        </MotionHeading>

        {/* Subtitle */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.5 }}
        >
          <Text
            fontSize="28px"
            fontWeight={500}
            color={kotakColors.goldLight}
            margin="0 0 40px 0"
            textAlign="center"
          >
            Acquiring India's Gold Loan Customers
          </Text>
        </MotionBox>

        {/* Divider */}
        <MotionBox
          initial={{ width: 0 }}
          animate={{ width: '120px' }}
          transition={{ duration: 0.8, delay: 0.7 }}
          style={{
            height: '3px',
            background: kotakColors.white,
            marginBottom: '40px',
          }}
        />

        {/* Description */}
        <MotionBox
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.6, delay: 0.9 }}
        >
          <Text
            fontSize="18px"
            color="rgba(255, 255, 255, 0.8)"
            margin="0 0 8px 0"
            textAlign="center"
          >
            A Product Strategy for C-Suite Leadership
          </Text>
          <Text
            fontSize="16px"
            color="rgba(255, 255, 255, 0.6)"
            margin="0"
            textAlign="center"
          >
            December 2025 | Board Confidential
          </Text>
        </MotionBox>

        {/* Kotak Logo */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 1.1 }}
          style={{ marginTop: '48px' }}
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
        Welcome everyone. Today we're presenting a strategic opportunity to acquire
        gold loan customers from NBFCs - a Rs 363 crore PAT opportunity over 3 years.
        This is a balance transfer product targeting customers who are overpaying at
        competitors like Muthoot, Manappuram, and IIFL.
      </Notes>
    </Slide>
  );
};

export default TitleSlide;
