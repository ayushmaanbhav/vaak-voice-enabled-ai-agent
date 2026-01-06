import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';
import { marketFundamentals } from '../../../data/marketData';

const MotionBox = motion(Box);

const metrics = [
  {
    value: marketFundamentals.totalMarketSize,
    label: 'Total Market Size',
    sublabel: 'Gold loan industry AUM',
    icon: '📊',
    color: kotakColors.primary,
  },
  {
    value: marketFundamentals.cagr,
    label: 'CAGR Growth',
    sublabel: '5-year projected growth',
    icon: '📈',
    color: kotakColors.success,
  },
  {
    value: marketFundamentals.householdGold,
    label: 'Household Gold',
    sublabel: 'Total gold held by Indian households',
    icon: '🏠',
    color: kotakColors.gold,
  },
  {
    value: marketFundamentals.goldMonetized,
    label: 'Gold Monetized',
    sublabel: 'Only this much is pledged for loans',
    icon: '💎',
    color: kotakColors.secondary,
  },
];

export const MarketSizeSlide: React.FC = () => {
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
            MARKET OPPORTUNITY
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 48px 0">
            A Rs 7.1 Lakh Crore Opportunity
          </Heading>
        </MotionBox>

        {/* Metric Cards */}
        <FlexBox flex={1} alignItems="center" justifyContent="center" style={{ gap: '24px' }}>
          {metrics.map((item, index) => (
            <MotionBox
              key={index}
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.2 + index * 0.1 }}
              style={{
                flex: 1,
                maxWidth: '240px',
                background: kotakColors.darkCard,
                borderRadius: '16px',
                padding: '32px 24px',
                textAlign: 'center',
                borderTop: `4px solid ${item.color}`,
              }}
            >
              <Text fontSize="40px" margin="0 0 12px 0">{item.icon}</Text>
              <Text fontSize="36px" fontWeight={700} color={item.color} margin="0">
                {item.value}
              </Text>
              <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="12px 0 4px 0">
                {item.label}
              </Text>
              <Text fontSize="14px" color={kotakColors.textMuted} margin="0">
                {item.sublabel}
              </Text>
            </MotionBox>
          ))}
        </FlexBox>

        {/* Bottom Insight */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.7 }}
          style={{
            marginTop: '24px',
            padding: '16px 24px',
            background: 'rgba(245, 158, 11, 0.1)',
            borderRadius: '12px',
            borderLeft: `4px solid ${kotakColors.gold}`,
          }}
        >
          <Text fontSize="18px" color={kotakColors.white} margin="0">
            <Text fontSize="18px" fontWeight={700} color={kotakColors.gold} style={{ display: 'inline' }}>
              94.4% untapped potential
            </Text>
            {' '}— Most household gold remains unmonetized, representing massive growth opportunity.
          </Text>
        </MotionBox>
      </FlexBox>

      <Notes>
        The Indian gold loan market is Rs 7.1 lakh crore in AUM with 14.85% CAGR growth.
        Indian households hold Rs 126 lakh crore worth of gold, but only 5.6% is monetized.
        This represents a massive untapped opportunity for organized lenders.
      </Notes>
    </Slide>
  );
};

export default MarketSizeSlide;
