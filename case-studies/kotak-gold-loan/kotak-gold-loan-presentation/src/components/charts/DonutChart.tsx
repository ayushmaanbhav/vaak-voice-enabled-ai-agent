import React from 'react';
import {
  PieChart,
  Pie,
  Cell,
  ResponsiveContainer,
  Tooltip,
  Legend,
} from 'recharts';
import { motion } from 'framer-motion';
import { Box, Text, FlexBox } from 'spectacle';
import { kotakColors } from '../../theme/kotakTheme';

const MotionBox = motion(Box);

interface DonutData {
  name: string;
  value: number;
  color: string;
  description?: string;
}

interface DonutChartProps {
  data: DonutData[];
  title?: string;
  height?: number;
  innerRadius?: number;
  outerRadius?: number;
  showLegend?: boolean;
  centerLabel?: string;
  centerValue?: string;
}

export const DonutChart: React.FC<DonutChartProps> = ({
  data,
  title,
  height = 300,
  innerRadius = 60,
  outerRadius = 100,
  showLegend = true,
  centerLabel,
  centerValue,
}) => {
  const renderCustomLabel = ({
    cx,
    cy,
    midAngle,
    innerRadius,
    outerRadius,
    percent,
  }: any) => {
    const RADIAN = Math.PI / 180;
    const radius = innerRadius + (outerRadius - innerRadius) * 0.5;
    const x = cx + radius * Math.cos(-midAngle * RADIAN);
    const y = cy + radius * Math.sin(-midAngle * RADIAN);

    if (percent < 0.08) return null; // Don't show label for small segments

    return (
      <text
        x={x}
        y={y}
        fill={kotakColors.white}
        textAnchor="middle"
        dominantBaseline="central"
        fontSize={14}
        fontWeight={600}
      >
        {`${(percent * 100).toFixed(0)}%`}
      </text>
    );
  };

  return (
    <MotionBox
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.6, type: 'spring' }}
      style={{ width: '100%', height, position: 'relative' }}
    >
      {title && (
        <Text fontSize="16px" color={kotakColors.textMuted} margin="0 0 8px 0" textAlign="center">
          {title}
        </Text>
      )}
      <ResponsiveContainer width="100%" height={title ? height - 30 : height}>
        <PieChart>
          <Pie
            data={data}
            cx="50%"
            cy="50%"
            labelLine={false}
            label={renderCustomLabel}
            outerRadius={outerRadius}
            innerRadius={innerRadius}
            dataKey="value"
            strokeWidth={2}
            stroke={kotakColors.dark}
          >
            {data.map((entry, index) => (
              <Cell key={`cell-${index}`} fill={entry.color} />
            ))}
          </Pie>
          <Tooltip
            contentStyle={{
              backgroundColor: kotakColors.darkCard,
              border: `1px solid ${kotakColors.primary}`,
              borderRadius: '8px',
              color: kotakColors.white,
            }}
            formatter={(value: number, name: string) => [`${value}%`, name]}
          />
          {showLegend && (
            <Legend
              verticalAlign="bottom"
              height={40}
              formatter={(value: string) => (
                <span style={{ color: kotakColors.white, fontSize: '14px' }}>{value}</span>
              )}
            />
          )}
        </PieChart>
      </ResponsiveContainer>

      {/* Center Label */}
      {(centerLabel || centerValue) && (
        <Box
          style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            textAlign: 'center',
          }}
        >
          {centerValue && (
            <Text fontSize="24px" fontWeight={700} color={kotakColors.white} margin="0">
              {centerValue}
            </Text>
          )}
          {centerLabel && (
            <Text fontSize="12px" color={kotakColors.textMuted} margin="4px 0 0 0">
              {centerLabel}
            </Text>
          )}
        </Box>
      )}
    </MotionBox>
  );
};

// Segment donut with legend on side
interface SegmentDonutProps {
  data: DonutData[];
  height?: number;
}

export const SegmentDonut: React.FC<SegmentDonutProps> = ({
  data,
  height: _height = 350,
}) => {
  void _height; // Suppress unused variable warning
  return (
    <MotionBox
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.5 }}
    >
      <FlexBox alignItems="center" style={{ gap: '40px' }}>
        {/* Chart */}
        <Box style={{ width: '200px', height: '200px', position: 'relative' }}>
          <ResponsiveContainer width="100%" height={200}>
            <PieChart>
              <Pie
                data={data}
                cx="50%"
                cy="50%"
                outerRadius={90}
                innerRadius={55}
                dataKey="value"
                strokeWidth={2}
                stroke={kotakColors.dark}
              >
                {data.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.color} />
                ))}
              </Pie>
            </PieChart>
          </ResponsiveContainer>
        </Box>

        {/* Legend */}
        <FlexBox flexDirection="column" style={{ gap: '12px' }}>
          {data.map((item, index) => (
            <MotionBox
              key={index}
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.3, delay: index * 0.1 }}
            >
              <FlexBox alignItems="center" style={{ gap: '12px' }}>
                <Box
                  style={{
                    width: '16px',
                    height: '16px',
                    borderRadius: '4px',
                    background: item.color,
                  }}
                />
                <Box>
                  <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0">
                    {item.name}
                  </Text>
                  <Text fontSize="14px" color={kotakColors.textMuted} margin="0">
                    {item.value}% | {item.description}
                  </Text>
                </Box>
              </FlexBox>
            </MotionBox>
          ))}
        </FlexBox>
      </FlexBox>
    </MotionBox>
  );
};

export default DonutChart;
