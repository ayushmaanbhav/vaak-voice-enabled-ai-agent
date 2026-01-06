import React from 'react';
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Tooltip,
  ReferenceLine,
} from 'recharts';
import { motion } from 'framer-motion';
import { Box, Text } from 'spectacle';
import { kotakColors } from '../../theme/kotakTheme';

const MotionBox = motion(Box);

interface GrowthData {
  period: string;
  value: number;
  label?: string;
}

interface AreaGrowthChartProps {
  data: GrowthData[];
  title?: string;
  height?: number;
  color?: string;
  valuePrefix?: string;
  valueSuffix?: string;
  target?: number;
  targetLabel?: string;
}

export const AreaGrowthChart: React.FC<AreaGrowthChartProps> = ({
  data,
  title,
  height = 250,
  color = kotakColors.primary,
  valuePrefix = '',
  valueSuffix = '',
  target,
  targetLabel,
}) => {
  return (
    <MotionBox
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      style={{ width: '100%' }}
    >
      {title && (
        <Text fontSize="16px" fontWeight={600} color={kotakColors.white} margin="0 0 16px 0">
          {title}
        </Text>
      )}
      <Box style={{ width: '100%', height }}>
        <ResponsiveContainer width="100%" height={height}>
          <AreaChart
            data={data}
            margin={{ top: 10, right: 30, left: 10, bottom: 10 }}
          >
            <defs>
              <linearGradient id="colorGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor={color} stopOpacity={0.4} />
                <stop offset="95%" stopColor={color} stopOpacity={0} />
              </linearGradient>
            </defs>
            <XAxis
              dataKey="period"
              axisLine={false}
              tickLine={false}
              tick={{ fill: kotakColors.textMuted, fontSize: 14 }}
            />
            <YAxis
              axisLine={false}
              tickLine={false}
              tick={{ fill: kotakColors.textMuted, fontSize: 14 }}
              tickFormatter={(v) => `${valuePrefix}${v}${valueSuffix}`}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: kotakColors.darkCard,
                border: `1px solid ${color}`,
                borderRadius: '8px',
                color: kotakColors.white,
              }}
              formatter={(value: number) => [`${valuePrefix}${value}${valueSuffix}`, 'Value']}
            />
            {target && (
              <ReferenceLine
                y={target}
                stroke={kotakColors.gold}
                strokeDasharray="5 5"
                label={{
                  value: targetLabel || `Target: ${valuePrefix}${target}${valueSuffix}`,
                  fill: kotakColors.gold,
                  fontSize: 12,
                  position: 'right',
                }}
              />
            )}
            <Area
              type="monotone"
              dataKey="value"
              stroke={color}
              strokeWidth={3}
              fill="url(#colorGradient)"
              dot={{
                fill: color,
                strokeWidth: 2,
                r: 5,
              }}
              activeDot={{
                r: 8,
                fill: color,
                stroke: kotakColors.white,
                strokeWidth: 2,
              }}
            />
          </AreaChart>
        </ResponsiveContainer>
      </Box>
    </MotionBox>
  );
};

// Multi-line growth chart for year comparisons
interface MultiLineData {
  period: string;
  [key: string]: string | number;
}

interface MultiLineChartProps {
  data: MultiLineData[];
  lines: { key: string; name: string; color: string }[];
  height?: number;
  valuePrefix?: string;
  valueSuffix?: string;
}

export const MultiLineGrowthChart: React.FC<MultiLineChartProps> = ({
  data,
  lines,
  height = 250,
  valuePrefix = '',
  valueSuffix = '',
}) => {
  return (
    <MotionBox
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.5 }}
      style={{ width: '100%', height }}
    >
      <ResponsiveContainer width="100%" height={height}>
        <AreaChart
          data={data}
          margin={{ top: 10, right: 30, left: 10, bottom: 10 }}
        >
          <defs>
            {lines.map((line, index) => (
              <linearGradient key={index} id={`gradient-${line.key}`} x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor={line.color} stopOpacity={0.3} />
                <stop offset="95%" stopColor={line.color} stopOpacity={0} />
              </linearGradient>
            ))}
          </defs>
          <XAxis
            dataKey="period"
            axisLine={false}
            tickLine={false}
            tick={{ fill: kotakColors.textMuted, fontSize: 14 }}
          />
          <YAxis
            axisLine={false}
            tickLine={false}
            tick={{ fill: kotakColors.textMuted, fontSize: 14 }}
            tickFormatter={(v) => `${valuePrefix}${v}${valueSuffix}`}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: kotakColors.darkCard,
              border: `1px solid ${kotakColors.primary}`,
              borderRadius: '8px',
              color: kotakColors.white,
            }}
          />
          {lines.map((line) => (
            <Area
              key={line.key}
              type="monotone"
              dataKey={line.key}
              name={line.name}
              stroke={line.color}
              strokeWidth={2}
              fill={`url(#gradient-${line.key})`}
              dot={{ fill: line.color, strokeWidth: 2, r: 4 }}
            />
          ))}
        </AreaChart>
      </ResponsiveContainer>
    </MotionBox>
  );
};

export default AreaGrowthChart;
