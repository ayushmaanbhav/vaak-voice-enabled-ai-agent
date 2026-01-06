import React from 'react';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Tooltip,
  Legend,
} from 'recharts';
import { motion } from 'framer-motion';
import { Box, Text, FlexBox } from 'spectacle';
import { kotakColors } from '../../theme/kotakTheme';

const MotionBox = motion(Box);

interface StackedData {
  category: string;
  [key: string]: string | number;
}

interface StackConfig {
  key: string;
  name: string;
  color: string;
}

interface StackedBarChartProps {
  data: StackedData[];
  stacks: StackConfig[];
  title?: string;
  height?: number;
  valuePrefix?: string;
  valueSuffix?: string;
  layout?: 'horizontal' | 'vertical';
}

export const StackedBarChart: React.FC<StackedBarChartProps> = ({
  data,
  stacks,
  title,
  height = 300,
  valuePrefix = '',
  valueSuffix = '',
  layout = 'horizontal',
}) => {
  const isVertical = layout === 'vertical';

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
          <BarChart
            data={data}
            layout={isVertical ? 'vertical' : 'horizontal'}
            margin={{ top: 10, right: 30, left: isVertical ? 80 : 10, bottom: 10 }}
          >
            <XAxis
              type={isVertical ? 'number' : 'category'}
              dataKey={isVertical ? undefined : 'category'}
              axisLine={false}
              tickLine={false}
              tick={{ fill: kotakColors.textMuted, fontSize: 14 }}
              tickFormatter={isVertical ? (v) => `${valuePrefix}${v}${valueSuffix}` : undefined}
            />
            <YAxis
              type={isVertical ? 'category' : 'number'}
              dataKey={isVertical ? 'category' : undefined}
              axisLine={false}
              tickLine={false}
              tick={{ fill: kotakColors.textMuted, fontSize: 14 }}
              tickFormatter={!isVertical ? (v) => `${valuePrefix}${v}${valueSuffix}` : undefined}
              width={isVertical ? 70 : 60}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: kotakColors.darkCard,
                border: `1px solid ${kotakColors.primary}`,
                borderRadius: '8px',
                color: kotakColors.white,
              }}
              formatter={(value: number, name: string) => [
                `${valuePrefix}${value}${valueSuffix}`,
                name,
              ]}
            />
            <Legend
              formatter={(value: string) => (
                <span style={{ color: kotakColors.white, fontSize: '14px' }}>{value}</span>
              )}
            />
            {stacks.map((stack) => (
              <Bar
                key={stack.key}
                dataKey={stack.key}
                name={stack.name}
                stackId="a"
                fill={stack.color}
                radius={[4, 4, 4, 4]}
              />
            ))}
          </BarChart>
        </ResponsiveContainer>
      </Box>
    </MotionBox>
  );
};

// Investment breakdown specific chart
interface InvestmentCategory {
  category: string;
  amount: number;
  priority: string;
  color: string;
}

interface InvestmentBreakdownChartProps {
  data: InvestmentCategory[];
  total: number;
  height?: number;
}

export const InvestmentBreakdownChart: React.FC<InvestmentBreakdownChartProps> = ({
  data,
  total,
  height: _height = 350,
}) => {
  void _height; // Suppress unused variable warning
  return (
    <MotionBox
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.5 }}
    >
      <FlexBox flexDirection="column" style={{ gap: '16px' }}>
        {/* Total */}
        <FlexBox justifyContent="space-between" alignItems="center" style={{ marginBottom: '8px' }}>
          <Text fontSize="18px" color={kotakColors.white} margin="0">
            Total Investment (Year 1)
          </Text>
          <Text fontSize="28px" fontWeight={700} color={kotakColors.primary} margin="0">
            Rs {total} Cr
          </Text>
        </FlexBox>

        {/* Bars */}
        {data.map((item, index) => {
          const percentage = (item.amount / total) * 100;
          return (
            <MotionBox
              key={index}
              initial={{ opacity: 0, width: 0 }}
              animate={{ opacity: 1, width: '100%' }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
            >
              <FlexBox justifyContent="space-between" alignItems="center" style={{ marginBottom: '4px' }}>
                <FlexBox alignItems="center" style={{ gap: '8px' }}>
                  <Box
                    style={{
                      width: '12px',
                      height: '12px',
                      borderRadius: '2px',
                      background: item.color,
                    }}
                  />
                  <Text fontSize="15px" color={kotakColors.white} margin="0">
                    {item.category}
                  </Text>
                </FlexBox>
                <Text fontSize="15px" fontWeight={600} color={item.color} margin="0">
                  Rs {item.amount} Cr
                </Text>
              </FlexBox>
              <Box
                style={{
                  width: '100%',
                  height: '24px',
                  background: kotakColors.darkCard,
                  borderRadius: '4px',
                  overflow: 'hidden',
                }}
              >
                <MotionBox
                  initial={{ width: 0 }}
                  animate={{ width: `${percentage}%` }}
                  transition={{ duration: 0.8, delay: index * 0.1 }}
                  style={{
                    height: '100%',
                    background: item.color,
                    borderRadius: '4px',
                  }}
                />
              </Box>
            </MotionBox>
          );
        })}
      </FlexBox>
    </MotionBox>
  );
};

// Year cards for financial summary
interface YearData {
  year: number;
  customers: number;
  aum: number;
  pat: number;
  investment: number;
}

interface YearCardsProps {
  data: YearData[];
}

export const YearCards: React.FC<YearCardsProps> = ({ data }) => {
  return (
    <FlexBox style={{ gap: '16px', width: '100%' }}>
      {data.map((year, index) => (
        <MotionBox
          key={index}
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4, delay: index * 0.15 }}
          style={{
            flex: 1,
            background: index === data.length - 1
              ? `linear-gradient(135deg, ${kotakColors.primary}15, ${kotakColors.primary}05)`
              : kotakColors.darkCard,
            borderRadius: '16px',
            padding: '24px',
            border: index === data.length - 1
              ? `2px solid ${kotakColors.primary}40`
              : `1px solid rgba(255, 255, 255, 0.1)`,
          }}
        >
          <Text
            fontSize="14px"
            fontWeight={600}
            color={index === data.length - 1 ? kotakColors.primary : kotakColors.gold}
            margin="0 0 16px 0"
          >
            Year {year.year}
          </Text>

          <FlexBox flexDirection="column" style={{ gap: '12px' }}>
            <Box>
              <Text fontSize="12px" color={kotakColors.textMuted} margin="0">Customers</Text>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.white} margin="0">
                {year.customers >= 1000 ? `${year.customers / 1000}K` : year.customers}
              </Text>
            </Box>
            <Box>
              <Text fontSize="12px" color={kotakColors.textMuted} margin="0">AUM</Text>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.gold} margin="0">
                Rs {year.aum.toLocaleString()} Cr
              </Text>
            </Box>
            <Box>
              <Text fontSize="12px" color={kotakColors.textMuted} margin="0">PAT</Text>
              <Text fontSize="24px" fontWeight={700} color={kotakColors.success} margin="0">
                Rs {year.pat} Cr
              </Text>
            </Box>
            <Box>
              <Text fontSize="12px" color={kotakColors.textMuted} margin="0">Investment</Text>
              <Text fontSize="18px" fontWeight={600} color={kotakColors.danger} margin="0">
                Rs {year.investment} Cr
              </Text>
            </Box>
          </FlexBox>
        </MotionBox>
      ))}
    </FlexBox>
  );
};

export default StackedBarChart;
