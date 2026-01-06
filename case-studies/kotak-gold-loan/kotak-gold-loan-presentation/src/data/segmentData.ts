// Customer segmentation data

export const segmentOverview = {
  totalAddressable: '10+ million',
  year1Target: 50000,
  year3Target: 425000,
  segments: [
    { id: 'P1', name: 'High-Value Switchers', share: 40, color: '#ED1C24' },
    { id: 'P2', name: 'Trust-Seekers', share: 25, color: '#F59E0B' },
    { id: 'P3', name: 'Women (Shakti)', share: 20, color: '#EC4899' },
    { id: 'P4', name: 'Young (811)', share: 15, color: '#3B82F6' },
  ],
};

export const segmentP1 = {
  id: 'P1',
  name: 'High-Value Switchers',
  priority: 'TOP PRIORITY',
  color: '#ED1C24',
  loanRange: 'Rs 5 lakh - Rs 25 lakh',
  currentRate: '14-20% p.a.',
  annualSavings: 'Rs 25,000 - Rs 2,50,000',
  profile: {
    age: '35-55 years',
    occupation: 'MSME owner, professional',
    location: 'Tier 1 & 2 cities',
    kotakRelationship: 'Existing savings/current account',
  },
  acquisition: {
    primaryChannel: 'RM outreach',
    secondaryChannel: 'App pre-approval',
    cac: 'Rs 3,000-4,000',
  },
  conversion: '8-12%',
  whyPriority: [
    'Highest revenue per customer',
    'Existing relationship reduces KYC burden',
    'Rational decision-makers responsive to savings math',
    'Lower credit risk due to established business',
  ],
  savingsExample: {
    loanAmount: 1000000,
    currentRate: 18,
    kotakRate: 10,
    annualSavings: 80000,
  },
};

export const segmentP2 = {
  id: 'P2',
  name: 'Trust-Seekers',
  priority: 'HIGH URGENCY',
  color: '#F59E0B',
  trigger: 'Concerned about industry events',
  loanRange: 'Rs 2 lakh - Rs 15 lakh',
  profile: {
    age: '40-55 years',
    behavior: 'Researching "safest gold loan", bank preference',
    riskProfile: 'Risk-averse, higher gold value',
  },
  acquisition: {
    primaryChannel: 'Digital campaigns',
    secondaryChannel: 'Trust-focused messaging',
    cac: 'Rs 1,500-2,000',
  },
  conversion: '10-15%',
  whyPriority: [
    'Self-motivated to switch',
    'Lower acquisition cost',
    'High loyalty potential',
    'Grateful for "rescue"',
  ],
  motivationDrivers: [
    { driver: 'IIFL crisis news', weight: 'HIGH' },
    { driver: 'Fraud cases in media', weight: 'HIGH' },
    { driver: 'Seeking bank security', weight: 'VERY HIGH' },
    { driver: 'Family pressure for safety', weight: 'MEDIUM' },
  ],
};

export const segmentP3 = {
  id: 'P3',
  name: 'Women Entrepreneurs (Shakti Gold)',
  priority: 'STRATEGIC',
  color: '#EC4899',
  loanRange: 'Rs 1 lakh - Rs 5 lakh',
  useCase: 'Business working capital, inventory',
  profile: {
    age: '25-50 years',
    occupation: 'Women business owners',
    location: 'Semi-urban/rural (70%)',
  },
  riskProfile: {
    defaultRate: '5.2%',
    maleDefaultRate: '6.9%',
    advantage: 'Lower default rates',
  },
  acquisition: {
    primaryChannel: 'BC network',
    secondaryChannel: 'Women SHG meetings, app',
    cac: 'Rs 1,000-1,500',
  },
  conversion: '6-8%',
  whyStrategic: [
    'Growing segment (4 Cr new women borrowers 2019-2024)',
    'Excellent credit quality',
    'Social impact narrative',
    'Leverages BSS/Sonata network (2.7M women)',
  ],
  marketStats: {
    newBorrowers: '4 crore',
    period: '2019-2024',
    totalAvailed: '4.7 trillion',
    leadingStates: ['Tamil Nadu (44%)', 'Andhra Pradesh (41%)', 'Karnataka (34%)'],
  },
};

export const segmentP4 = {
  id: 'P4',
  name: 'Young Professionals (Kotak 811)',
  priority: 'VOLUME PLAY',
  color: '#3B82F6',
  loanRange: 'Rs 50,000 - Rs 3 lakh',
  useCase: 'Emergency, travel, education, gadgets',
  profile: {
    age: '21-35 years',
    behavior: 'Digital-native',
    goldSource: 'Family jewelry',
  },
  acquisition: {
    primaryChannel: 'Kotak 811 app',
    secondaryChannel: 'Social media, influencers',
    cac: 'Rs 500-1,000',
  },
  conversion: '4-6%',
  reach: '20M+ Kotak 811 users',
  whyVolume: [
    'Massive reach through existing app',
    'Digital-native, low service cost',
    'Future high-value customers',
    'Lower ticket but high volume potential',
  ],
};

export const channelMapping = [
  {
    segment: 'P1: High-Value',
    primaryChannel: 'RM Outreach',
    secondaryChannel: 'Branch',
    cacTarget: 'Rs 3,000-4,000',
    conversionTarget: '8-12%',
  },
  {
    segment: 'P2: Trust-Seekers',
    primaryChannel: 'Digital Campaign',
    secondaryChannel: 'Branch',
    cacTarget: 'Rs 1,500-2,000',
    conversionTarget: '10-15%',
  },
  {
    segment: 'P3: Women',
    primaryChannel: 'BC Network + App',
    secondaryChannel: 'Branch',
    cacTarget: 'Rs 1,000-1,500',
    conversionTarget: '6-8%',
  },
  {
    segment: 'P4: Young',
    primaryChannel: 'Kotak 811 App',
    secondaryChannel: 'Social Media',
    cacTarget: 'Rs 500-1,000',
    conversionTarget: '4-6%',
  },
];

export const messagingMatrix = [
  {
    segment: 'P1: MSME Owner',
    primaryFear: 'Business disruption',
    message: '"Working capital that never stops"',
    tone: 'Professional',
  },
  {
    segment: 'P2: Trust-Seeker',
    primaryFear: 'Gold safety',
    message: '"Your gold. Our responsibility. Guaranteed."',
    tone: 'Reassuring',
  },
  {
    segment: 'P3: Women',
    primaryFear: 'Respect, safety',
    message: '"Shakti Gold: Financial independence, dignified service"',
    tone: 'Empowering',
  },
  {
    segment: 'P4: Young',
    primaryFear: 'Speed, convenience',
    message: '"Gold loan in clicks, not queues"',
    tone: 'Modern',
  },
];

// Consolidated segment array for easy iteration
export const customerSegments = [
  {
    id: 'P1',
    name: 'High-Value Switchers',
    color: '#ED1C24',
    icon: '💎',
    profile: 'MSME owners, Rs 5L-25L loans',
    conversionRate: '8-12%',
    loanRange: 'Rs 5-25L',
  },
  {
    id: 'P2',
    name: 'Trust-Seekers',
    color: '#F59E0B',
    icon: '🛡️',
    profile: 'Risk-averse, seeking bank safety',
    conversionRate: '10-15%',
    loanRange: 'Rs 2-15L',
  },
  {
    id: 'P3',
    name: 'Women (Shakti)',
    color: '#EC4899',
    icon: '👩‍💼',
    profile: 'Women entrepreneurs, lower defaults',
    conversionRate: '6-8%',
    loanRange: 'Rs 1-5L',
  },
  {
    id: 'P4',
    name: 'Young (811)',
    color: '#3B82F6',
    icon: '📱',
    profile: 'Digital-native, emergency needs',
    conversionRate: '4-6%',
    loanRange: 'Rs 50K-3L',
  },
];

export const propensityScoring = {
  description: 'AI-powered customer identification',
  signals: [
    { type: 'Transaction Signals', description: 'Payments to gold loan providers', confidence: 'HIGH' },
    { type: 'Demographic Fit', description: 'Age, geography, occupation', confidence: 'MEDIUM' },
    { type: 'Financial Stress', description: 'Account balance patterns', confidence: 'MEDIUM' },
    { type: 'Switching History', description: 'Prior provider switches', confidence: 'VERY HIGH' },
    { type: 'Life Events', description: 'Wedding, medical, education payments', confidence: 'HIGH' },
  ],
  actions: [
    { scoreRange: '90-100', action: 'Immediate outreach', channel: 'RM call + App', timing: 'Within 24 hours' },
    { scoreRange: '75-89', action: 'Priority campaign', channel: 'App push + SMS', timing: 'Within 1 week' },
    { scoreRange: '60-74', action: 'Standard campaign', channel: 'Email + App', timing: 'Monthly campaigns' },
    { scoreRange: 'Below 60', action: 'No active targeting', channel: 'Passive awareness', timing: '-' },
  ],
};
