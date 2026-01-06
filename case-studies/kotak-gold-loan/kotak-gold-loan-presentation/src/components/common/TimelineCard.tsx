import React from 'react';
import { Box, Text, FlexBox } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../theme/kotakTheme';

const MotionBox = motion(Box);

interface TimelineEvent {
  date: string;
  event: string;
  description?: string;
  icon?: string;
  type?: 'regulatory' | 'crisis' | 'opportunity' | 'default';
}

interface TimelineCardProps {
  events: TimelineEvent[];
  variant?: 'horizontal' | 'vertical';
}

const typeColors = {
  regulatory: kotakColors.secondary,
  crisis: kotakColors.danger,
  opportunity: kotakColors.success,
  default: kotakColors.gold,
};

export const TimelineCard: React.FC<TimelineCardProps> = ({
  events,
  variant = 'horizontal',
}) => {
  if (variant === 'vertical') {
    return (
      <FlexBox flexDirection="column" style={{ gap: '0' }}>
        {events.map((event, index) => {
          const color = typeColors[event.type || 'default'];
          return (
            <MotionBox
              key={index}
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.4, delay: index * 0.15 }}
            >
              <FlexBox alignItems="flex-start" style={{ gap: '20px' }}>
                {/* Timeline dot and line */}
                <FlexBox flexDirection="column" alignItems="center" style={{ width: '20px' }}>
                  <Box
                    style={{
                      width: '16px',
                      height: '16px',
                      borderRadius: '50%',
                      background: color,
                      border: `3px solid ${kotakColors.dark}`,
                      boxShadow: `0 0 0 2px ${color}`,
                    }}
                  />
                  {index < events.length - 1 && (
                    <Box
                      style={{
                        width: '2px',
                        flex: 1,
                        minHeight: '60px',
                        background: `linear-gradient(180deg, ${color}, ${kotakColors.darkCard})`,
                      }}
                    />
                  )}
                </FlexBox>

                {/* Content */}
                <Box style={{ flex: 1, paddingBottom: '24px' }}>
                  <Text fontSize="14px" fontWeight={600} color={color} margin="0 0 4px 0">
                    {event.date}
                  </Text>
                  <FlexBox alignItems="center" style={{ gap: '8px', marginBottom: '8px' }}>
                    {event.icon && (
                      <Text fontSize="20px" margin="0">{event.icon}</Text>
                    )}
                    <Text fontSize="18px" fontWeight={600} color={kotakColors.white} margin="0">
                      {event.event}
                    </Text>
                  </FlexBox>
                  {event.description && (
                    <Text fontSize="16px" color={kotakColors.textMuted} margin="0" style={{ lineHeight: 1.5 }}>
                      {event.description}
                    </Text>
                  )}
                </Box>
              </FlexBox>
            </MotionBox>
          );
        })}
      </FlexBox>
    );
  }

  // Horizontal variant
  return (
    <FlexBox style={{ gap: '16px', width: '100%' }}>
      {events.map((event, index) => {
        const color = typeColors[event.type || 'default'];
        return (
          <MotionBox
            key={index}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.4, delay: index * 0.15 }}
            style={{
              flex: 1,
              background: kotakColors.darkCard,
              borderRadius: '16px',
              padding: '24px',
              borderTop: `4px solid ${color}`,
              position: 'relative',
            }}
          >
            {/* Date Badge */}
            <Box
              style={{
                background: `${color}20`,
                borderRadius: '6px',
                padding: '4px 12px',
                display: 'inline-block',
                marginBottom: '12px',
              }}
            >
              <Text fontSize="14px" fontWeight={600} color={color} margin="0">
                {event.date}
              </Text>
            </Box>

            {event.icon && (
              <Text fontSize="28px" margin="0 0 12px 0">{event.icon}</Text>
            )}

            <Text fontSize="18px" fontWeight={600} color={kotakColors.white} margin="0 0 8px 0">
              {event.event}
            </Text>

            {event.description && (
              <Text fontSize="15px" color={kotakColors.textMuted} margin="0" style={{ lineHeight: 1.5 }}>
                {event.description}
              </Text>
            )}

            {/* Connector */}
            {index < events.length - 1 && (
              <Box
                style={{
                  position: 'absolute',
                  right: '-16px',
                  top: '50%',
                  transform: 'translateY(-50%)',
                  color: kotakColors.textMuted,
                  fontSize: '20px',
                  zIndex: 1,
                }}
              >
                →
              </Box>
            )}
          </MotionBox>
        );
      })}
    </FlexBox>
  );
};

export default TimelineCard;
