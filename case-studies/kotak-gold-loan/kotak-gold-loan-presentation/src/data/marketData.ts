// Market data extracted from the strategy document

export const marketFundamentals = {
  marketSize: {
    current: '7.1',
    unit: 'Lakh Crore',
    year: '2024-25',
    projected: '14.19',
    projectedYear: '2028',
  },
  cagr: '14.85%',
  householdGold: 'Rs 126L Cr',
  monetized: '5.6%',
  organizedShare: '37',
  unorganizedShare: '63',
  bankGrowthYoY: '71.3',
  // Convenience properties for slides
  totalMarketSize: 'Rs 7.1L Cr',
  goldMonetized: '5.6%',
};

export const marketMetrics = [
  {
    icon: '💰',
    value: '7.1L Cr',
    label: 'Market Size',
    sublabel: '2024-25',
    color: 'primary',
  },
  {
    icon: '📈',
    value: '14.85%',
    label: 'CAGR',
    sublabel: 'Projected growth',
    color: 'success',
  },
  {
    icon: '🏠',
    value: '126L Cr',
    label: 'Household Gold',
    sublabel: 'Untapped wealth',
    color: 'gold',
  },
  {
    icon: '🎯',
    value: '5.6%',
    label: 'Monetized',
    sublabel: 'Only this much!',
    color: 'danger',
  },
];

export const growthDrivers = [
  {
    metric: 'Organized Sector',
    value: 37,
    remaining: 63,
    insight: '63% still unorganized - massive opportunity',
  },
  {
    metric: 'Gold Monetized',
    value: 5.6,
    remaining: 94.4,
    insight: '94.4% of household gold untapped',
  },
  {
    metric: 'Bank Gold Loan Growth',
    value: 71.3,
    unit: '% YoY',
    insight: 'Banks winning market share from NBFCs',
  },
];

export const interestRateComparison: Array<{
  lender: string;
  minRate: number;
  maxRate: number;
  type: 'bank' | 'nbfc';
  color: string;
  highlight?: boolean;
}> = [
  { lender: 'SBI', minRate: 8.5, maxRate: 11.25, type: 'bank', color: '#003874' },
  { lender: 'Bank of Baroda', minRate: 7.75, maxRate: 10.5, type: 'bank', color: '#003874' },
  { lender: 'Kotak', minRate: 9, maxRate: 12, type: 'bank', color: '#ED1C24', highlight: true },
  { lender: 'HDFC Bank', minRate: 9.3, maxRate: 14, type: 'bank', color: '#003874' },
  { lender: 'Muthoot', minRate: 11.9, maxRate: 26, type: 'nbfc', color: '#64748B' },
  { lender: 'Manappuram', minRate: 12, maxRate: 26, type: 'nbfc', color: '#64748B' },
  { lender: 'IIFL', minRate: 9.24, maxRate: 24, type: 'nbfc', color: '#64748B' },
];

export const savingsCalculator = [
  {
    loanAmount: '2 lakh',
    nbfcCost: '36,000',
    kotakCost: '20,000',
    savings: '16,000',
    savingsPercent: 44,
  },
  {
    loanAmount: '5 lakh',
    nbfcCost: '90,000',
    kotakCost: '50,000',
    savings: '40,000',
    savingsPercent: 44,
  },
  {
    loanAmount: '10 lakh',
    nbfcCost: '1,80,000',
    kotakCost: '1,00,000',
    savings: '80,000',
    savingsPercent: 44,
  },
];

export const whyNowTimeline = [
  {
    date: 'Sep 2024',
    event: 'RBI Issues 11-Point Circular',
    description: 'Industry-wide deficiencies identified, compliance deadline set',
    icon: '📋',
    type: 'regulatory',
  },
  {
    date: 'Mar 2024',
    event: 'IIFL Gold Loan Operations Barred',
    description: 'RBI action creates customer anxiety, trust crisis begins',
    icon: '🚫',
    type: 'crisis',
  },
  {
    date: 'Jun 2025',
    event: 'New RBI Directions Announced',
    description: 'Stricter LTV, bullet loan caps, customer protection mandates',
    icon: '⚡',
    type: 'regulatory',
  },
];

export const competitorMarketShare = [
  { name: 'Muthoot Finance', share: 38, aum: '1.09L Cr', branches: '7,500+' },
  { name: 'IIFL Finance', share: 13, aum: '27,000 Cr', branches: 'Pan-India' },
  { name: 'Manappuram', share: 12, aum: '20,800 Cr', branches: '5,000+' },
  { name: 'SBI', share: 15, aum: '1.72L Cr', branches: 'Pan-India' },
  { name: 'Other Banks', share: 12, aum: 'Growing', branches: '-' },
  { name: 'Unorganized', share: 10, aum: '-', branches: '-' },
];

export const kotakPosition = {
  customerBase: '53 million',
  digitalUsers: '20M+ Kotak 811',
  interestRate: '9-12% p.a.',
  npaRate: '0.22%',
  complianceRecord: 'Zero gold loan violations',
  advantage: '40-50% savings for customers',
};
