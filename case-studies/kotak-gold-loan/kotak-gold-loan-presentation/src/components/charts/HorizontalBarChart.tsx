import React from 'react';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Cell,
  LabelList,
  Tooltip,
} from 'recharts';
import { motion } from 'framer-motion';
import { Box, Text } from 'spectacle';
import { kotakColors } from '../../theme/kotakTheme';

const MotionBox = motion(Box);

interface BarData {
  name: string;
  value: number;
  color?: string;
  highlight?: boolean;
  label?: string;
}

interface HorizontalBarChartProps {
  data: BarData[];
  title?: string;
  height?: number;
  showValues?: boolean;
  valueFormatter?: (value: number) => string;
  barSize?: number;
}

export const HorizontalBarChart: React.FC<HorizontalBarChartProps> = ({
  data,
  title,
  height = 300,
  showValues = true,
  valueFormatter = (v) => `${v}%`,
  barSize = 28,
}) => {
  return (
    <MotionBox
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      style={{ width: '100%', height }}
    >
      {title && (
        <Text fontSize="16px" color={kotakColors.textMuted} margin="0 0 16px 0">
          {title}
        </Text>
      )}
      <ResponsiveContainer width="100%" height={title ? height - 40 : height}>
        <BarChart
          data={data}
          layout="vertical"
          margin={{ top: 5, right: 80, left: 100, bottom: 5 }}
        >
          <XAxis
            type="number"
            axisLine={false}
            tickLine={false}
            tick={{ fill: kotakColors.textMuted, fontSize: 14 }}
            tickFormatter={valueFormatter}
          />
          <YAxis
            type="category"
            dataKey="name"
            axisLine={false}
            tickLine={false}
            tick={{ fill: kotakColors.white, fontSize: 15, fontWeight: 500 }}
            width={90}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: kotakColors.darkCard,
              border: `1px solid ${kotakColors.primary}`,
              borderRadius: '8px',
              color: kotakColors.white,
            }}
            formatter={(value: number) => [valueFormatter(value), 'Rate']}
          />
          <Bar
            dataKey="value"
            barSize={barSize}
            radius={[0, 4, 4, 0]}
          >
            {data.map((entry, index) => (
              <Cell
                key={`cell-${index}`}
                fill={entry.highlight ? kotakColors.primary : (entry.color || kotakColors.textMuted)}
              />
            ))}
            {showValues && (
              <LabelList
                dataKey="value"
                position="right"
                formatter={valueFormatter}
                style={{ fill: kotakColors.white, fontSize: 14, fontWeight: 600 }}
              />
            )}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </MotionBox>
  );
};

// Rate comparison specific chart
interface RateData {
  lender: string;
  minRate: number;
  maxRate: number;
  type: 'bank' | 'nbfc';
  highlight?: boolean;
}

interface RateComparisonChartProps {
  data: RateData[];
  height?: number;
}

export const RateComparisonChart: React.FC<RateComparisonChartProps> = ({
  data,
  height = 350,
}) => {
  const chartData = data.map(d => ({
    name: d.lender,
    value: d.maxRate,
    color: d.highlight ? kotakColors.primary : (d.type === 'bank' ? kotakColors.secondary : '#64748B'),
    highlight: d.highlight,
  }));

  return (
    <MotionBox
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.6 }}
      style={{ width: '100%', height }}
    >
      <ResponsiveContainer width="100%" height={height}>
        <BarChart
          data={chartData}
          layout="vertical"
          margin={{ top: 10, right: 60, left: 100, bottom: 10 }}
        >
          <XAxis
            type="number"
            domain={[0, 30]}
            axisLine={false}
            tickLine={false}
            tick={{ fill: kotakColors.textMuted, fontSize: 14 }}
            tickFormatter={(v) => `${v}%`}
            ticks={[0, 8, 16, 30]}
          />
          <YAxis
            type="category"
            dataKey="name"
            axisLine={false}
            tickLine={false}
            tick={{ fill: kotakColors.white, fontSize: 15, fontWeight: 500 }}
            width={95}
          />
          {/* Reference line for Kotak max rate */}
          <Bar
            dataKey="value"
            barSize={24}
            radius={[0, 4, 4, 0]}
          >
            {chartData.map((entry, index) => (
              <Cell
                key={`cell-${index}`}
                fill={entry.color}
              />
            ))}
            <LabelList
              dataKey="value"
              position="right"
              formatter={(v: number) => `${v}%`}
              style={{ fill: kotakColors.white, fontSize: 14, fontWeight: 600 }}
            />
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </MotionBox>
  );
};

export default HorizontalBarChart;
