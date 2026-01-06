import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { HorizontalBarChart } from '../../charts/HorizontalBarChart';

const MotionBox = motion(Box);

const marketShareData = [
  { name: 'Unorganized (Local Lenders)', value: 63, fill: kotakColors.textMuted },
  { name: 'NBFCs (Muthoot, Manappuram)', value: 22, fill: kotakColors.gold },
  { name: 'Banks', value: 15, fill: kotakColors.primary },
];

const drivers = [
  { icon: '🏦', text: 'Banks gaining share from NBFCs due to lower rates' },
  { icon: '📱', text: 'Digital adoption accelerating post-COVID' },
  { icon: '⚠️', text: 'NBFC regulatory issues creating switching momentum' },
];

export const GrowthDriversSlide: React.FC = () => {
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
            MARKET DYNAMICS
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 32px 0">
            Banks Are Winning the Gold Loan War
          </Heading>
        </MotionBox>

        {/* Main Content */}
        <FlexBox flex={1} style={{ gap: '48px' }}>
          {/* Left: Market Share Chart */}
          <MotionBox
            initial={{ opacity: 0, x: -30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            style={{ flex: 1.2 }}
          >
            <Box
              style={{
                background: kotakColors.darkCard,
                borderRadius: '16px',
                padding: '24px',
                height: '100%',
              }}
            >
              <Text fontSize="18px" fontWeight={600} color={kotakColors.white} margin="0 0 16px 0">
                Current Market Share Distribution
              </Text>
              <HorizontalBarChart
                data={marketShareData}
                height={200}
                showValues
                valueFormatter={(v) => `${v}%`}
              />
              <FlexBox justifyContent="center" style={{ gap: '24px', marginTop: '16px' }}>
                {marketShareData.map((item, i) => (
                  <FlexBox key={i} alignItems="center" style={{ gap: '8px' }}>
                    <Box style={{ width: '12px', height: '12px', borderRadius: '2px', background: item.fill }} />
                    <Text fontSize="13px" color={kotakColors.textMuted} margin="0">{item.name}</Text>
                  </FlexBox>
                ))}
              </FlexBox>
            </Box>
          </MotionBox>

          {/* Right: Key Drivers */}
          <MotionBox
            initial={{ opacity: 0, x: 30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            style={{ flex: 1 }}
          >
            <Text fontSize="18px" fontWeight={600} color={kotakColors.white} margin="0 0 20px 0">
              Key Growth Drivers
            </Text>
            <FlexBox flexDirection="column" style={{ gap: '16px' }}>
              {drivers.map((driver, index) => (
                <MotionBox
                  key={index}
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.4, delay: 0.4 + index * 0.1 }}
                  style={{
                    background: kotakColors.darkCard,
                    borderRadius: '12px',
                    padding: '20px',
                    borderLeft: `3px solid ${kotakColors.primary}`,
                  }}
                >
                  <FlexBox alignItems="center" style={{ gap: '16px' }}>
                    <Text fontSize="32px" margin="0">{driver.icon}</Text>
                    <Text fontSize="17px" color={kotakColors.white} margin="0" style={{ lineHeight: 1.4 }}>
                      {driver.text}
                    </Text>
                  </FlexBox>
                </MotionBox>
              ))}
            </FlexBox>

            {/* Opportunity Callout */}
            <MotionBox
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.7 }}
              style={{
                marginTop: '24px',
                padding: '20px',
                background: `linear-gradient(135deg, ${kotakColors.primary}20, ${kotakColors.primary}05)`,
                borderRadius: '12px',
                border: `1px solid ${kotakColors.primary}40`,
              }}
            >
              <Text fontSize="16px" fontWeight={600} color={kotakColors.primary} margin="0 0 4px 0">
                THE OPPORTUNITY
              </Text>
              <Text fontSize="18px" color={kotakColors.white} margin="0">
                63% unorganized market + NBFC distrust = massive acquisition potential
              </Text>
            </MotionBox>
          </MotionBox>
        </FlexBox>
      </FlexBox>

      <Notes>
        The gold loan market is 63% unorganized - local moneylenders charging 36-60% rates.
        Banks are gaining share from NBFCs due to significantly lower rates and better trust.
        Recent NBFC issues like IIFL ban and fraud cases are accelerating this shift.
      </Notes>
    </Slide>
  );
};

export default GrowthDriversSlide;
