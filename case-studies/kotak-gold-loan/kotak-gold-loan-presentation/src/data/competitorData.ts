// Competitor issues and fraud data - NEW SECTION

export const manappuramFraudCases = [
  {
    date: 'Feb 2025',
    location: 'Vikasnagar, Gorakhpur, UP',
    amount: '43.34 lakh',
    goldWeight: '786.7 grams',
    type: 'Fake Gold Replacement',
    description: 'Five employees replaced genuine customer gold with fake ornaments',
    accused: '5 employees',
    victims: '3 customers',
    status: 'FIR filed, investigation ongoing',
  },
  {
    date: 'Jul 2024',
    location: 'Thrissur, Kerala',
    amount: '20 crore',
    type: 'Tech Fraud',
    description: 'AGM Tech-lead siphoned funds through fraudulent transactions over 5 years',
    accused: 'Dhanya Mohan (AGM)',
    discovery: 'Global CrowdStrike IT outage prevented data deletion',
    status: 'Arrested',
  },
  {
    date: '2024',
    location: 'Bhopal, MP',
    amount: '5.50 crore',
    type: 'Fake Loan + Betting Scam',
    description: 'Branch Manager and Deputy Manager sanctioned fake gold loan, lost money in online betting',
    accused: 'Branch Manager, Deputy Manager',
    status: 'Both arrested, FIR registered',
  },
  {
    date: 'Jan 2023',
    location: 'Odisha',
    amount: '70.88 lakh',
    type: 'Fake Gold Loan',
    description: 'Branch Manager and Assistant granted loan against fake gold',
    accused: 'Branch Manager, Assistant Branch Manager',
    status: 'Under investigation',
  },
];

export const manappuramRBIPenalties = [
  {
    date: 'Jun 2023',
    amount: '20 lakh',
    violation: 'NPA Classification Failure',
    description: 'Failed to classify gold loan accounts with overdue >90 days as NPAs',
    inspectionRef: 'Financial position as of March 2021',
  },
  {
    date: 'Nov 2020',
    amount: '5 lakh',
    violation: 'Ownership Verification Non-Compliance',
    description: 'Non-compliance with verification of ownership of gold jewellery',
    inspectionRef: 'Financial position as of March 2019',
  },
];

export const manappuramServiceIssues = [
  {
    issue: 'Doorstep App Unavailable',
    description: 'Core differentiating feature "unavailable due to revamping" since late 2024',
    impact: 'Customer convenience significantly reduced',
  },
  {
    issue: 'Asirvad Subsidiary Barred',
    date: 'Oct 2024',
    description: 'RBI barred Asirvad Microfinance from sanctioning new loans',
    violations: ['Pricing policies violations', 'Risk management failures', 'Gold loan portfolio issues', 'Customer service failures'],
    stockImpact: 'Manappuram shares fell 14%',
  },
];

export const muthootIssues = {
  rating: {
    platform: 'MouthShut',
    score: '2.19',
    maxScore: '5',
    totalReviews: '1000+',
  },
  complaints: [
    {
      type: 'Excessive Penalties',
      quote: '"Unjustifiable charge... excessive penalty for a single day"',
      impact: 'HIGH',
    },
    {
      type: 'App Payment Issues',
      quote: '"Part payment feature totally useless"',
      impact: 'HIGH',
    },
    {
      type: 'Dismissive Staff',
      quote: '"Dismissive attitude" when raising complaints',
      impact: 'MEDIUM',
    },
    {
      type: 'Renewal Hassles',
      description: 'Documentation repeated unnecessarily',
      impact: 'MEDIUM',
    },
  ],
};

export const iiflCrisis = {
  rbiAction: {
    date: 'Mar 2024',
    action: 'Barred from gold loan operations',
    duration: 'Until remediation complete',
    trigger: 'Multiple compliance violations',
  },
  violations: [
    { type: 'LTV Breaches', description: 'Systematic loan-to-value ratio violations' },
    { type: 'Cash Disbursement', description: 'Exceeding Rs 20,000 cash limit regularly' },
    { type: 'Improper Valuations', description: 'Gold valuation without customer presence' },
    { type: 'Auction Irregularities', description: 'Non-transparent auction processes' },
    { type: 'Evergreening', description: 'Loan evergreening to hide NPAs' },
  ],
  impact: 'Customer trust crisis, portfolio erosion, regulatory scrutiny on entire industry',
};

export const industryWideIssues = [
  {
    issue: 'Third-party Valuation',
    description: 'Gold valuation conducted without customer presence',
    rbiPosition: 'Must be done in customer presence',
  },
  {
    issue: 'LTV Monitoring Gaps',
    description: 'Weak ongoing loan-to-value monitoring',
    rbiPosition: 'Real-time monitoring required',
  },
  {
    issue: 'Cash Disbursement Violations',
    description: 'Exceeding Rs 20,000 statutory limit',
    rbiPosition: 'Zero cash policy recommended',
  },
  {
    issue: 'Auction Transparency',
    description: 'Opaque auction processes',
    rbiPosition: 'Transparent, documented auctions',
  },
  {
    issue: 'Loan Evergreening',
    description: 'Concealing NPAs through renewals',
    rbiPosition: 'Strict NPA recognition',
  },
];

export const trustCrisisSummary = {
  headline: 'Gold Loan Industry Trust Crisis',
  subheadline: 'NBFCs face regulatory scrutiny, fraud cases, and service failures',
  keyPoints: [
    'Rs 89+ crore in documented fraud cases (Manappuram alone)',
    'Multiple RBI penalties across major NBFCs',
    'IIFL operations completely barred',
    'Customer complaints at all-time high',
    'Doorstep services disrupted',
  ],
  kotakOpportunity: 'Position as "safe, compliant, transparent" bank alternative',
};

export const nbfcVsBank = [
  {
    attribute: 'Regulatory Oversight',
    nbfc: 'RBI (limited)',
    bank: 'RBI (comprehensive)',
    advantage: 'bank',
  },
  {
    attribute: 'Deposit Insurance',
    nbfc: 'None',
    bank: 'DICGC protected',
    advantage: 'bank',
  },
  {
    attribute: 'Interest Rates',
    nbfc: '12-27%',
    bank: '8-14%',
    advantage: 'bank',
  },
  {
    attribute: 'Compliance Record',
    nbfc: 'Multiple violations',
    bank: 'Generally clean',
    advantage: 'bank',
  },
  {
    attribute: 'Customer Protection',
    nbfc: 'Limited recourse',
    bank: 'Banking Ombudsman',
    advantage: 'bank',
  },
  {
    attribute: 'Gold Safety',
    nbfc: 'Fraud cases reported',
    bank: 'Bank vault security',
    advantage: 'bank',
  },
];
