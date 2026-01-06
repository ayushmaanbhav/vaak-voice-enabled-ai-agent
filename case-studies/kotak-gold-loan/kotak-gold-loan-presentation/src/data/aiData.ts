// AI and Technology data

export const aiOverview = {
  title: 'AI-Powered Gold Loan Acquisition',
  totalInvestment: '14.5',
  unit: 'Cr',
  solutions: [
    {
      name: 'Computer Vision Appraisal',
      investment: '5 Cr',
      priority: 'CRITICAL',
      roi: '80% conversion lift',
      timeline: '12 months',
      icon: '📸',
    },
    {
      name: 'Predictive Acquisition Model',
      investment: '2.5 Cr',
      priority: 'CRITICAL',
      roi: '16x campaign conversion',
      timeline: '6 months',
      icon: '🎯',
    },
    {
      name: 'Personalized Pitching Engine',
      investment: '1.5 Cr (Phase 1)',
      priority: 'EXPERIMENTAL',
      roi: '7x offer acceptance',
      timeline: '9 months',
      icon: '🎨',
    },
  ],
};

export const computerVisionSolution = {
  title: 'Computer Vision: Appraisal in Customer\'s Palm',
  problemSolved: 'Customer uncertainty about gold value creates hesitation; multiple branch visits waste time',
  impactLabel: 'HIGH IMPACT',
  journey: [
    {
      step: 1,
      title: 'App Capture',
      description: 'Customer photographs gold at home',
      time: '2 min',
      icon: '📱',
    },
    {
      step: 2,
      title: 'AI Analysis',
      description: 'Multi-angle assessment, hallmark detection',
      time: '30 sec',
      icon: '🤖',
    },
    {
      step: 3,
      title: 'Instant Estimate',
      description: 'Indicative value within ±15% accuracy',
      time: 'Instant',
      icon: '💰',
    },
    {
      step: 4,
      title: 'Branch Visit',
      description: 'Quick XRF verification, 60-min total',
      time: '63 min',
      icon: '🏦',
    },
  ],
  technicalSpecs: {
    appBased: [
      { component: 'Item Classification', technology: 'YOLO v8 / EfficientNet', accuracy: '97%+' },
      { component: 'Hallmark Detection', technology: 'OCR + BIS database', accuracy: '92%+' },
      { component: 'Purity Estimation', technology: 'CNN color/luster analysis', accuracy: '75-80%' },
      { component: 'Weight Estimation', technology: 'Photogrammetry', accuracy: '±15%' },
    ],
    branchBased: [
      { component: 'Purity Testing', technology: 'XRF (X-Ray Fluorescence)', accuracy: '99%+ (error <0.5%)' },
      { component: 'Testing Time', technology: 'Handheld XRF analyzers', accuracy: '40-60 seconds' },
    ],
  },
  impact: {
    metrics: [
      { metric: 'Branch conversion', current: '25%', withCV: '45%', improvement: '+80%' },
      { metric: 'Time to sanction', current: '4-6 hours', withCV: '90 minutes', improvement: '-65%' },
      { metric: 'Customer drop-off', current: '40%', withCV: '15%', improvement: '-62%' },
      { metric: 'Cost per qualified lead', current: 'Rs 500', withCV: 'Rs 200', improvement: '-60%' },
    ],
  },
  investment: {
    total: '5 Cr',
    breakdown: [
      { item: 'Data Collection (200K images)', cost: '40 lakh' },
      { item: 'Cloud Infrastructure', cost: '50 lakh' },
      { item: 'Model Development', cost: '1.2 Cr' },
      { item: 'App Integration', cost: '80 lakh' },
      { item: 'Security & Compliance', cost: '60 lakh' },
      { item: 'Pilot & Testing', cost: '70 lakh' },
      { item: 'Full Rollout', cost: '50 lakh' },
      { item: 'Buffer (15%)', cost: '65 lakh' },
    ],
  },
  compliance: 'App appraisal is PRELIMINARY ONLY. Final valuation at branch with customer present, using BIS-certified XRF.',
};

export const predictiveModel = {
  title: 'Predictive Acquisition: Finding Hidden Gold Loan Customers',
  problemSolved: 'Identifying which of Kotak\'s 53 million customers have active gold loans with other providers',
  signalCategories: [
    {
      category: 'Transaction Signals',
      icon: '💳',
      signals: ['NBFC EMI payments', 'Gold lender UPI', 'Interest payments'],
      weight: 35,
      color: '#ED1C24',
    },
    {
      category: 'Account Behavior',
      icon: '📊',
      signals: ['Recurring outflows', 'Month-end patterns', 'Fixed amounts'],
      weight: 25,
      color: '#F59E0B',
    },
    {
      category: 'Demographic Fit',
      icon: '👤',
      signals: ['Age 35-55', 'Self-employed', 'Gold belt geography'],
      weight: 20,
      color: '#3B82F6',
    },
    {
      category: 'Credit Profile',
      icon: '📈',
      signals: ['Active loans', 'Credit inquiries', 'Bureau tradelines'],
      weight: 15,
      color: '#10B981',
    },
    {
      category: 'App Behavior',
      icon: '📱',
      signals: ['Gold loan page views', 'Rate calculator usage', 'Branch locator'],
      weight: 5,
      color: '#8B5CF6',
    },
  ],
  signals: [
    { type: 'Recurring payments to gold lenders', pattern: 'Monthly/quarterly fixed amounts', confidence: 'HIGH' },
    { type: 'Payment descriptions', pattern: 'Keywords: "GL", "interest", provider names', confidence: 'HIGH' },
    { type: 'UPI patterns', pattern: 'Payments to gold loan merchant codes', confidence: 'MEDIUM' },
    { type: 'Amount patterns', pattern: 'Rs 2K-50K recurring (interest range)', confidence: 'MEDIUM' },
  ],
  sampleOutput: {
    customerId: 'KOTAK789012',
    propensityScore: 87,
    competitorLikely: true,
    estimatedOutstanding: 'Rs 3,50,000',
    recommendedOffer: {
      rate: '9.5%',
      processingFee: 'Zero',
      special: 'Rs 3,000 cashback',
    },
    bestChannel: 'RM call',
    bestTime: 'Month-end (payment due cycle)',
  },
  performance: [
    { metric: 'Campaign response', untargeted: '2%', aiTargeted: '8%', lift: '4x' },
    { metric: 'Lead to conversion', untargeted: '15%', aiTargeted: '35%', lift: '2.3x' },
    { metric: 'Overall conversion', untargeted: '0.12%', aiTargeted: '1.96%', lift: '16x' },
    { metric: 'CAC', untargeted: 'Rs 2,000', aiTargeted: 'Rs 500', lift: '75% lower' },
  ],
  investment: {
    total: '2.5 Cr',
    breakdown: [
      { item: 'Data Engineering', cost: '60 lakh' },
      { item: 'ML Model Development', cost: '55 lakh' },
      { item: 'CRM Integration', cost: '45 lakh' },
      { item: 'Privacy & Compliance (DPDP)', cost: '35 lakh' },
      { item: 'Testing & Validation', cost: '25 lakh' },
      { item: 'Buffer (15%)', cost: '30 lakh' },
    ],
  },
};

export const personalizedPitching = {
  title: 'Personalized Pitching: Right Offer, Right Customer',
  parameters: [
    { param: 'Age Group', source: 'KYC', impact: 'Communication tone' },
    { param: 'Gender', source: 'KYC', impact: 'Shakti Gold for women' },
    { param: 'Occupation', source: 'KYC, transactions', impact: 'MSME vs salaried messaging' },
    { param: 'Credit Score', source: 'Bureau', impact: 'Risk-based pricing' },
  ],
  offerMatrix: [
    { segment: 'MSME Owner', rate: '9.5%', processingFee: 'Zero', special: 'Top-up facility', theme: 'Fuel your business growth' },
    { segment: 'Trust-Seeker', rate: '9.25%', processingFee: 'Zero', special: 'Rs 3,000 cashback', theme: 'Safety meets savings' },
    { segment: 'Women (Shakti)', rate: '9.0%', processingFee: 'Zero', special: 'Free insurance', theme: 'Your gold, your power' },
    { segment: 'Young Professional', rate: '10.5%', processingFee: '0.5%', special: 'Rs 500 instant cashback', theme: 'Gold loan in clicks' },
  ],
  phases: [
    { phase: 1, scope: 'Rule-based segmentation (4 segments)', investment: '1.5 Cr', criteria: '5% conversion lift' },
    { phase: 2, scope: 'ML-based dynamic personalization', investment: '1 Cr', criteria: 'Additional 3% lift' },
    { phase: 3, scope: 'Real-time behavioral triggers', investment: '1 Cr', criteria: '10% total lift' },
  ],
};

export const additionalAI = [
  {
    name: 'LTV Monitoring',
    investment: '1.5 Cr',
    priority: 'MANDATORY',
    description: 'Real-time LTV calculation, breach alerts',
    reason: 'RBI September 2024 circular compliance',
  },
  {
    name: 'Fraud Detection',
    investment: '2 Cr',
    priority: 'CRITICAL',
    description: 'Fake gold detection, employee behavior anomaly',
    reason: 'Loss prevention',
  },
  {
    name: 'Multilingual Chatbot',
    investment: '2 Cr',
    priority: 'EFFICIENCY',
    description: '24x7 support in 6 languages',
    reason: 'Cost reduction',
  },
];

// Consolidated AI solutions array for easy iteration
export const aiSolutions = [
  {
    name: 'Computer Vision Appraisal',
    color: '#ED1C24',
    icon: '📸',
    priority: 'CRITICAL',
    description: 'App-based gold assessment with AI-powered purity estimation. Reduces branch visits and speeds up disbursement.',
    benefits: [
      '+80% branch conversion',
      '90-min total process (vs 4-6 hours)',
      '60% lower cost per qualified lead',
    ],
    roi: '+80%',
    investment: 'Rs 5 Cr',
  },
  {
    name: 'Predictive Acquisition',
    color: '#3B82F6',
    icon: '🎯',
    priority: 'CRITICAL',
    description: 'ML model identifying existing customers with gold loans at competitors through transaction pattern analysis.',
    benefits: [
      '16x campaign conversion lift',
      '75% lower acquisition cost',
      'Propensity scoring for 53M customers',
    ],
    roi: '16x',
    investment: 'Rs 2.5 Cr',
  },
  {
    name: 'Personalized Pitching',
    color: '#F59E0B',
    icon: '🎨',
    priority: 'EXPERIMENTAL',
    description: 'Segment-specific offers and messaging based on customer demographics, behavior, and financial profile.',
    benefits: [
      '7x offer acceptance rate',
      'Segment-specific rate offers',
      'Dynamic personalization',
    ],
    roi: '7x',
    investment: 'Rs 1.5 Cr',
  },
];

export const techInvestmentSummary = {
  total: '14.5 Cr',
  breakdown: [
    { category: 'Critical', items: ['CV Appraisal', 'Predictive Model', 'Fraud Detection'], total: '9.5 Cr' },
    { category: 'Mandatory', items: ['LTV Monitoring'], total: '1.5 Cr' },
    { category: 'Experimental', items: ['Personalization (Ph1)'], total: '1.5 Cr' },
    { category: 'Efficiency', items: ['Chatbot'], total: '2 Cr' },
  ],
};
