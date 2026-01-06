import React from 'react';
import { Box, Text, FlexBox } from 'spectacle';
import { motion } from 'framer-motion';
import { kotakColors } from '../../theme/kotakTheme';

const MotionBox = motion(Box);

interface TableColumn {
  key: string;
  header: string;
  width?: string;
  align?: 'left' | 'center' | 'right';
  highlight?: boolean;
}

interface ComparisonTableProps {
  columns: TableColumn[];
  data: Record<string, any>[];
  highlightColumn?: string;
  variant?: 'default' | 'compact' | 'striped';
}

export const ComparisonTable: React.FC<ComparisonTableProps> = ({
  columns,
  data,
  highlightColumn,
  variant = 'default',
}) => {
  const isCompact = variant === 'compact';
  const cellPadding = isCompact ? '12px 16px' : '16px 20px';
  const fontSize = isCompact ? '15px' : '16px';
  const headerFontSize = isCompact ? '14px' : '16px';

  return (
    <MotionBox
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      style={{
        width: '100%',
        borderRadius: '12px',
        overflow: 'hidden',
        border: `1px solid rgba(255, 255, 255, 0.1)`,
      }}
    >
      {/* Header */}
      <FlexBox
        style={{
          background: kotakColors.darkCard,
          borderBottom: `1px solid rgba(255, 255, 255, 0.1)`,
        }}
      >
        {columns.map((col) => (
          <Box
            key={col.key}
            style={{
              flex: col.width ? `0 0 ${col.width}` : 1,
              padding: cellPadding,
              textAlign: col.align || 'left',
              background: col.key === highlightColumn ? `${kotakColors.primary}20` : 'transparent',
            }}
          >
            <Text
              fontSize={headerFontSize}
              fontWeight={700}
              color={col.key === highlightColumn ? kotakColors.primary : kotakColors.white}
              margin="0"
            >
              {col.header}
            </Text>
          </Box>
        ))}
      </FlexBox>

      {/* Rows */}
      {data.map((row, rowIndex) => (
        <MotionBox
          key={rowIndex}
          initial={{ opacity: 0, x: -10 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.3, delay: rowIndex * 0.05 }}
        >
          <FlexBox
            style={{
              background: variant === 'striped' && rowIndex % 2 === 1
                ? 'rgba(255, 255, 255, 0.02)'
                : 'transparent',
              borderBottom: rowIndex < data.length - 1
                ? `1px solid rgba(255, 255, 255, 0.05)`
                : 'none',
            }}
          >
            {columns.map((col) => (
              <Box
                key={col.key}
                style={{
                  flex: col.width ? `0 0 ${col.width}` : 1,
                  padding: cellPadding,
                  textAlign: col.align || 'left',
                  background: col.key === highlightColumn ? `${kotakColors.primary}10` : 'transparent',
                }}
              >
                <Text
                  fontSize={fontSize}
                  fontWeight={col.key === highlightColumn ? 600 : 400}
                  color={col.key === highlightColumn ? kotakColors.primary : kotakColors.textPrimary}
                  margin="0"
                >
                  {row[col.key]}
                </Text>
              </Box>
            ))}
          </FlexBox>
        </MotionBox>
      ))}
    </MotionBox>
  );
};

// Side-by-side comparison variant
interface SideBySideProps {
  leftTitle: string;
  rightTitle: string;
  leftItems: { label: string; value: string }[];
  rightItems: { label: string; value: string }[];
  leftColor?: string;
  rightColor?: string;
}

export const SideBySideComparison: React.FC<SideBySideProps> = ({
  leftTitle,
  rightTitle,
  leftItems,
  rightItems,
  leftColor = kotakColors.danger,
  rightColor = kotakColors.success,
}) => {
  return (
    <FlexBox style={{ gap: '24px', width: '100%' }}>
      {/* Left Side */}
      <MotionBox
        initial={{ opacity: 0, x: -30 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.5 }}
        style={{
          flex: 1,
          background: `linear-gradient(135deg, ${leftColor}15, ${leftColor}05)`,
          borderRadius: '16px',
          padding: '24px',
          border: `2px solid ${leftColor}30`,
        }}
      >
        <FlexBox alignItems="center" style={{ gap: '12px', marginBottom: '20px' }}>
          <Text fontSize="24px" margin="0">❌</Text>
          <Text fontSize="20px" fontWeight={700} color={leftColor} margin="0">
            {leftTitle}
          </Text>
        </FlexBox>
        <FlexBox flexDirection="column" style={{ gap: '12px' }}>
          {leftItems.map((item, index) => (
            <FlexBox key={index} justifyContent="space-between" alignItems="center">
              <Text fontSize="16px" color={kotakColors.textMuted} margin="0">
                {item.label}
              </Text>
              <Text fontSize="18px" fontWeight={600} color={leftColor} margin="0">
                {item.value}
              </Text>
            </FlexBox>
          ))}
        </FlexBox>
      </MotionBox>

      {/* Arrow */}
      <FlexBox alignItems="center">
        <Text fontSize="32px" color={kotakColors.textMuted} margin="0">→</Text>
      </FlexBox>

      {/* Right Side */}
      <MotionBox
        initial={{ opacity: 0, x: 30 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.5, delay: 0.2 }}
        style={{
          flex: 1,
          background: `linear-gradient(135deg, ${rightColor}15, ${rightColor}05)`,
          borderRadius: '16px',
          padding: '24px',
          border: `2px solid ${rightColor}30`,
        }}
      >
        <FlexBox alignItems="center" style={{ gap: '12px', marginBottom: '20px' }}>
          <Text fontSize="24px" margin="0">✅</Text>
          <Text fontSize="20px" fontWeight={700} color={rightColor} margin="0">
            {rightTitle}
          </Text>
        </FlexBox>
        <FlexBox flexDirection="column" style={{ gap: '12px' }}>
          {rightItems.map((item, index) => (
            <FlexBox key={index} justifyContent="space-between" alignItems="center">
              <Text fontSize="16px" color={kotakColors.textMuted} margin="0">
                {item.label}
              </Text>
              <Text fontSize="18px" fontWeight={600} color={rightColor} margin="0">
                {item.value}
              </Text>
            </FlexBox>
          ))}
        </FlexBox>
      </MotionBox>
    </FlexBox>
  );
};

export default ComparisonTable;
