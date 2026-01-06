import React from 'react';
import { Box, Text, FlexBox } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../theme/kotakTheme';

const MotionBox = motion(Box);

interface ProcessStep {
  step?: number;
  title: string;
  description?: string;
  time?: string;
  icon?: string;
  day?: string;
}

interface ProcessFlowProps {
  steps: ProcessStep[];
  variant?: 'horizontal' | 'vertical';
  color?: 'primary' | 'success' | 'gold';
  showConnectors?: boolean;
}

const colorMap = {
  primary: kotakColors.primary,
  success: kotakColors.success,
  gold: kotakColors.gold,
};

export const ProcessFlow: React.FC<ProcessFlowProps> = ({
  steps,
  variant = 'horizontal',
  color = 'primary',
  showConnectors = true,
}) => {
  const accentColor = colorMap[color];

  if (variant === 'vertical') {
    return (
      <FlexBox flexDirection="column" style={{ gap: '0' }}>
        {steps.map((step, index) => (
          <MotionBox
            key={index}
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.4, delay: index * 0.1 }}
          >
            <FlexBox alignItems="flex-start" style={{ gap: '16px' }}>
              {/* Timeline */}
              <FlexBox flexDirection="column" alignItems="center" style={{ width: '40px' }}>
                <Box
                  style={{
                    width: '32px',
                    height: '32px',
                    borderRadius: '50%',
                    background: accentColor,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  <Text fontSize="14px" fontWeight={700} color={kotakColors.white} margin="0">
                    {step.step || index + 1}
                  </Text>
                </Box>
                {showConnectors && index < steps.length - 1 && (
                  <Box
                    style={{
                      width: '2px',
                      height: '40px',
                      background: `${accentColor}40`,
                    }}
                  />
                )}
              </FlexBox>

              {/* Content */}
              <Box style={{ flex: 1, paddingBottom: '16px' }}>
                {step.day && (
                  <Text fontSize="12px" color={accentColor} fontWeight={600} margin="0 0 4px 0">
                    {step.day}
                  </Text>
                )}
                <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0">
                  {step.title}
                </Text>
                {step.description && (
                  <Text fontSize="14px" color={kotakColors.textMuted} margin="4px 0 0 0">
                    {step.description}
                  </Text>
                )}
              </Box>
            </FlexBox>
          </MotionBox>
        ))}
      </FlexBox>
    );
  }

  // Horizontal variant
  return (
    <FlexBox alignItems="flex-start" style={{ gap: '8px' }}>
      {steps.map((step, index) => (
        <React.Fragment key={index}>
          <MotionBox
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.4, delay: index * 0.1 }}
            style={{
              flex: 1,
              background: kotakColors.darkCard,
              borderRadius: '12px',
              padding: '20px',
              border: `1px solid rgba(255, 255, 255, 0.1)`,
              textAlign: 'center',
              position: 'relative',
            }}
          >
            {/* Step Number Badge */}
            <Box
              style={{
                position: 'absolute',
                top: '-12px',
                left: '50%',
                transform: 'translateX(-50%)',
                width: '24px',
                height: '24px',
                borderRadius: '50%',
                background: accentColor,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Text fontSize="12px" fontWeight={700} color={kotakColors.white} margin="0">
                {step.step || index + 1}
              </Text>
            </Box>

            {step.icon && (
              <Text fontSize="32px" margin="8px 0 12px 0">
                {step.icon}
              </Text>
            )}

            <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0">
              {step.title}
            </Text>

            {step.description && (
              <Text fontSize="14px" color={kotakColors.textMuted} margin="8px 0 0 0" style={{ lineHeight: 1.4 }}>
                {step.description}
              </Text>
            )}

            {step.time && (
              <Box
                style={{
                  marginTop: '12px',
                  background: `${accentColor}20`,
                  borderRadius: '6px',
                  padding: '4px 10px',
                  display: 'inline-block',
                }}
              >
                <Text fontSize="12px" fontWeight={600} color={accentColor} margin="0">
                  {step.time}
                </Text>
              </Box>
            )}
          </MotionBox>

          {/* Connector Arrow */}
          {showConnectors && index < steps.length - 1 && (
            <FlexBox alignItems="center" style={{ padding: '0 4px' }}>
              <Text fontSize="20px" color={kotakColors.textMuted} margin="0">
                →
              </Text>
            </FlexBox>
          )}
        </React.Fragment>
      ))}
    </FlexBox>
  );
};

export default ProcessFlow;
