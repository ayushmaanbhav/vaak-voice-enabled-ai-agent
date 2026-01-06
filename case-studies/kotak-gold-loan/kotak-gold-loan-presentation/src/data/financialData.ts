// Financial projections and ROI data

export const threeYearProjections = {
  years: [
    {
      year: 1,
      customers: 50000,
      aum: 1500,
      revenue: 162,
      nii: 87,
      ppop: 36,
      pat: 23,
      investment: 53,
    },
    {
      year: 2,
      customers: 125000,
      aum: 4000,
      revenue: 430,
      nii: 230,
      ppop: 145,
      pat: 99,
      investment: 45,
    },
    {
      year: 3,
      customers: 250000,
      aum: 8500,
      revenue: 905,
      nii: 480,
      ppop: 351,
      pat: 241,
      investment: 50,
    },
  ],
  totals: {
    customers: 425000,
    revenue: 1497,
    ppop: 532,
    pat: 363,
    investment: 148,
  },
};

export const plAssumptions = {
  yield: { value: 10, label: 'Interest Yield', note: 'Kotak gold loan rate' },
  fundingCost: { value: 5, label: 'Cost of Funds', note: 'Kotak CASA advantage' },
  nimSpread: { value: 5, label: 'NIM Spread', note: 'Bank advantage over NBFCs' },
  opexRatio: { value: 1.5, label: 'Operating Cost', note: 'Conservative (industry: 1.0-1.2%)' },
  creditCost: { value: 0.35, label: 'Credit Cost', note: 'Conservative (gold loans typically <0.1%)' },
  taxRate: { value: 25, label: 'Tax Rate', note: 'Corporate tax rate' },
};

export const roiAnalysis = {
  paybackPeriod: '~20 months',
  threeYearROI: '2.45x',
  irr: '~75%',
  breakEvenAUM: 'Rs 900 Cr',
  breakEvenTimeline: '7-9 months',
  riskProfile: 'LOW',
  riskNote: 'Gold loans have near-zero NPA historically',
  // For slide display
  totalInvestment: 'Rs 148 Cr',
  totalPAT: 'Rs 363 Cr',
  roi: '2.45x',
};

export const sensitivityAnalysis = [
  {
    scenario: 'Conservative (-20%)',
    aumAchieved: 6800,
    threeYearPAT: 290,
    roi: '1.96x',
    color: 'warning',
  },
  {
    scenario: 'Base Case',
    aumAchieved: 8500,
    threeYearPAT: 363,
    roi: '2.45x',
    color: 'success',
    highlight: true,
  },
  {
    scenario: 'Optimistic (+20%)',
    aumAchieved: 10200,
    threeYearPAT: 436,
    roi: '2.95x',
    color: 'gold',
  },
];

export const investmentContext = {
  ourAsk: {
    amount: 53,
    unit: 'Cr',
    period: 'Year 1',
  },
  benchmarks: [
    {
      comparison: 'Annual ICT Spend',
      kotakValue: '1,650 Cr',
      ourPercent: '3.2%',
      perspective: 'Minimal portion of IT budget',
    },
    {
      comparison: 'Quarterly PAT',
      kotakValue: '3,305 Cr',
      ourPercent: '1.6%',
      perspective: 'Fraction of one quarter\'s profit',
    },
    {
      comparison: 'Sonata Acquisition',
      kotakValue: '537 Cr',
      ourPercent: '10x smaller',
      perspective: 'Much lower risk investment',
    },
    {
      comparison: 'Digital Investment (2 yrs)',
      kotakValue: '1,000+ Cr',
      ourPercent: '5.3%',
      perspective: 'Aligned with digital strategy',
    },
  ],
};

export const investmentBreakdown = {
  year1: {
    total: 53,
    categories: [
      { category: 'AI/Technology', amount: 14.5, priority: 'Mixed', color: '#3B82F6' },
      { category: 'Marketing & Acquisition', amount: 15, priority: 'Critical', color: '#ED1C24' },
      { category: 'Compliance & Security', amount: 10, priority: 'Mandatory', color: '#10B981' },
      { category: 'Training & HR', amount: 5, priority: 'Critical', color: '#F59E0B' },
      { category: 'Doorstep Pilot', amount: 5, priority: 'Experimental', color: '#8B5CF6' },
      { category: 'Contingency (7%)', amount: 3.5, priority: '-', color: '#64748B' },
    ],
  },
};

export const crisisPreventionValue = {
  impactCategories: [
    { category: 'Portfolio Erosion (50%)', cost: '750 Cr', note: 'at Rs 1,500 Cr AUM' },
    { category: 'Emergency Capital', cost: '300+ Cr', note: 'Regulatory requirement' },
    { category: 'Remediation Costs', cost: '100+ Cr', note: 'Process fixes, audits' },
    { category: 'Customer Re-acquisition', cost: '50+ Cr', note: 'Rebuild trust' },
    { category: 'Reputational Damage', cost: 'Incalculable', note: 'Brand impact' },
  ],
  totalCrisisCost: '1,200+ Cr',
  complianceInvestment: '10 Cr',
  roiInRiskPrevention: '120x',
};

export const growthGuardrails = {
  yellowFlags: [
    { metric: 'Gold Loan NPA', threshold: '>0.5%' },
    { metric: 'LTV Breach Accounts', threshold: '>2% of portfolio' },
    { metric: 'Audit Findings (Material)', threshold: '3-5 per quarter' },
    { metric: 'Customer Complaints', threshold: '>0.3% of accounts' },
    { metric: 'Gold Return SLA Breach', threshold: '>5% of releases' },
  ],
  redFlags: [
    { metric: 'Gold Loan NPA', threshold: '>1.0%', action: 'PAUSE GROWTH' },
    { metric: 'LTV Breach Accounts', threshold: '>5%', action: 'PAUSE GROWTH' },
    { metric: 'Audit Findings (Material)', threshold: '>5 per quarter', action: 'PAUSE GROWTH' },
    { metric: 'Customer Complaints', threshold: '>0.5%', action: 'PAUSE GROWTH' },
    { metric: 'Gold Return SLA Breach', threshold: '>10%', action: 'PAUSE GROWTH' },
  ],
  phases: [
    { phase: 1, aumTarget: '750 Cr', prerequisites: 'BIS certified, LTV system live' },
    { phase: 2, aumTarget: '1,500 Cr', prerequisites: 'Zero major audit findings, NPS > 50' },
    { phase: 3, aumTarget: '4,000 Cr', prerequisites: 'External compliance certification' },
    { phase: 4, aumTarget: '8,500 Cr', prerequisites: '<0.5% NPA, model compliance' },
  ],
};

export const profitabilityRatios = {
  metrics: [
    { ratio: 'NIM on AUM', y1: '5.8%', y2: '5.75%', y3: '5.65%', industry: '10-12%' },
    { ratio: 'ROA', y1: '1.5%', y2: '2.5%', y3: '2.8%', industry: '4.0%' },
    { ratio: 'Cost-to-Income', y1: '58%', y2: '44%', y3: '37%', industry: '~45%' },
    { ratio: 'Net Profit Margin', y1: '14%', y2: '23%', y3: '27%', industry: '26%' },
  ],
  note: 'Lower NIM than NBFCs due to competitive rates (10% vs 18-24%), offset by lower funding cost',
};

export const muthootBenchmark = {
  company: 'Muthoot Finance (Market Leader)',
  metrics: [
    { metric: 'Gold Loan AUM', value: '1.13 lakh Cr', note: 'Market leader' },
    { metric: 'NIM', value: '10.3-12.1%', note: 'High margins' },
    { metric: 'ROE', value: '17-22%', note: 'Excellent returns' },
    { metric: 'ROA', value: '4.0%', note: 'Strong' },
    { metric: 'Net Profit Margin', value: '26.4%', note: 'Highly profitable' },
    { metric: 'GNPA', value: '0.0%', note: 'Near-zero NPAs' },
    { metric: 'AUM Growth', value: '40% YoY', note: 'Strong demand' },
  ],
  insight: 'Gold loans are highly profitable with minimal credit risk when managed well.',
};
