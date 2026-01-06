import React from 'react';
import { Slide, Heading, Text, FlexBox, Box, Notes } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../../theme/kotakTheme';

const MotionBox = motion(Box);

const riskIndicators = [
  {
    category: 'Asset Quality',
    metric: 'NPA Rate',
    green: '< 0.5%',
    yellow: '0.5-1.0%',
    red: '> 1.0%',
    action: 'Pause acquisition, review underwriting',
  },
  {
    category: 'Growth Quality',
    metric: 'CAC Efficiency',
    green: '< Rs 2,000',
    yellow: 'Rs 2,000-2,500',
    red: '> Rs 2,500',
    action: 'Optimize channels, reduce marketing',
  },
  {
    category: 'Profitability',
    metric: 'Unit Economics',
    green: '> Rs 5,000 LTV',
    yellow: 'Rs 4,000-5,000',
    red: '< Rs 4,000',
    action: 'Review pricing, reduce servicing cost',
  },
  {
    category: 'Compliance',
    metric: 'Audit Findings',
    green: 'Zero critical',
    yellow: '1-2 minor',
    red: 'Any critical',
    action: 'Immediate remediation, pause scaling',
  },
];

export const RiskGuardrailsSlide: React.FC = () => {
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
            color={kotakColors.warning}
            margin="0 0 8px 0"
            style={{ letterSpacing: '2px', textTransform: 'uppercase' }}
          >
            RISK MANAGEMENT
          </Text>
        </MotionBox>

        {/* Title */}
        <MotionBox
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          <Heading fontSize="44px" fontWeight={700} color={kotakColors.white} margin="0 0 24px 0">
            Risk Guardrails & Triggers
          </Heading>
        </MotionBox>

        {/* Risk Indicator Table */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.2 }}
          style={{ flex: 1 }}
        >
          <Box
            style={{
              background: kotakColors.darkCard,
              borderRadius: '16px',
              overflow: 'hidden',
            }}
          >
            {/* Header */}
            <FlexBox style={{ background: kotakColors.primary, padding: '14px 20px' }}>
              <Box style={{ width: '120px' }}><Text fontSize="12px" fontWeight={600} color={kotakColors.white} margin="0">Category</Text></Box>
              <Box style={{ width: '100px' }}><Text fontSize="12px" fontWeight={600} color={kotakColors.white} margin="0">Metric</Text></Box>
              <Box style={{ flex: 1, textAlign: 'center' }}>
                <FlexBox justifyContent="center" style={{ gap: '24px' }}>
                  <Text fontSize="12px" fontWeight={600} color={kotakColors.white} margin="0">🟢 Green</Text>
                  <Text fontSize="12px" fontWeight={600} color={kotakColors.white} margin="0">🟡 Yellow</Text>
                  <Text fontSize="12px" fontWeight={600} color={kotakColors.white} margin="0">🔴 Red</Text>
                </FlexBox>
              </Box>
              <Box style={{ flex: 1 }}><Text fontSize="12px" fontWeight={600} color={kotakColors.white} margin="0">Action if Red</Text></Box>
            </FlexBox>

            {/* Rows */}
            {riskIndicators.map((row, index) => (
              <MotionBox
                key={index}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.4, delay: 0.3 + index * 0.1 }}
              >
                <FlexBox
                  style={{
                    padding: '16px 20px',
                    borderBottom: index < riskIndicators.length - 1 ? '1px solid rgba(255,255,255,0.05)' : 'none',
                    background: index % 2 === 1 ? 'rgba(255,255,255,0.02)' : 'transparent',
                  }}
                  alignItems="center"
                >
                  <Box style={{ width: '120px' }}>
                    <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0">{row.category}</Text>
                  </Box>
                  <Box style={{ width: '100px' }}>
                    <Text fontSize="13px" color={kotakColors.textMuted} margin="0">{row.metric}</Text>
                  </Box>
                  <Box style={{ flex: 1 }}>
                    <FlexBox justifyContent="center" style={{ gap: '16px' }}>
                      <Box style={{ padding: '6px 12px', background: `${kotakColors.success}20`, borderRadius: '6px', minWidth: '80px', textAlign: 'center' }}>
                        <Text fontSize="12px" fontWeight={600} color={kotakColors.success} margin="0">{row.green}</Text>
                      </Box>
                      <Box style={{ padding: '6px 12px', background: `${kotakColors.warning}20`, borderRadius: '6px', minWidth: '80px', textAlign: 'center' }}>
                        <Text fontSize="12px" fontWeight={600} color={kotakColors.warning} margin="0">{row.yellow}</Text>
                      </Box>
                      <Box style={{ padding: '6px 12px', background: `${kotakColors.danger}20`, borderRadius: '6px', minWidth: '80px', textAlign: 'center' }}>
                        <Text fontSize="12px" fontWeight={600} color={kotakColors.danger} margin="0">{row.red}</Text>
                      </Box>
                    </FlexBox>
                  </Box>
                  <Box style={{ flex: 1 }}>
                    <Text fontSize="13px" color={kotakColors.textMuted} margin="0">{row.action}</Text>
                  </Box>
                </FlexBox>
              </MotionBox>
            ))}
          </Box>
        </MotionBox>

        {/* Gold Loan Risk Profile */}
        <MotionBox
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.7 }}
          style={{ marginTop: '20px' }}
        >
          <FlexBox style={{ gap: '16px' }}>
            <Box
              style={{
                flex: 1,
                background: `${kotakColors.success}10`,
                borderRadius: '12px',
                padding: '16px',
                border: `1px solid ${kotakColors.success}30`,
              }}
            >
              <FlexBox alignItems="center" style={{ gap: '12px' }}>
                <Text fontSize="28px" margin="0">🛡️</Text>
                <Box>
                  <Text fontSize="14px" fontWeight={600} color={kotakColors.success} margin="0 0 4px 0">
                    INHERENTLY LOW RISK
                  </Text>
                  <Text fontSize="13px" color={kotakColors.textMuted} margin="0">
                    Gold loans are secured at 75% LTV. Industry NPA is &lt;1%. Collateral value typically appreciates.
                  </Text>
                </Box>
              </FlexBox>
            </Box>
            <Box
              style={{
                flex: 1,
                background: kotakColors.darkCard,
                borderRadius: '12px',
                padding: '16px',
              }}
            >
              <FlexBox alignItems="center" style={{ gap: '12px' }}>
                <Text fontSize="28px" margin="0">📊</Text>
                <Box>
                  <Text fontSize="14px" fontWeight={600} color={kotakColors.white} margin="0 0 4px 0">
                    QUARTERLY REVIEW
                  </Text>
                  <Text fontSize="13px" color={kotakColors.textMuted} margin="0">
                    Program committee review with pause authority. Escalation to board if any red triggers.
                  </Text>
                </Box>
              </FlexBox>
            </Box>
          </FlexBox>
        </MotionBox>
      </FlexBox>

      <Notes>
        Risk guardrails: NPA under 0.5% green, CAC under Rs 2000 green, LTV over Rs 5000 green.
        Any red trigger equals pause and remediate. Quarterly program committee review.
        Gold loans inherently low risk - secured at 75% LTV, industry NPA under 1%.
      </Notes>
    </Slide>
  );
};

export default RiskGuardrailsSlide;
