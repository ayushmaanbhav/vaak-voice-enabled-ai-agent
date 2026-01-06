// Product and campaign data

export const switchAndSaveProduct = {
  name: 'Switch & Save',
  tagline: 'Bank-Grade Security. India\'s Smartest Gold Loan.',
  target: 'Existing Kotak customers with competitor gold loans',
  minimumAmount: 'Rs 3 lakh',
  features: [
    {
      icon: '💰',
      title: 'Low Interest Rate',
      description: '9.5-10.5% p.a. - 40-50% below NBFC rates',
      highlight: true,
    },
    {
      icon: '🎯',
      title: 'Zero Processing Fee',
      description: 'No upfront costs for balance transfers',
      highlight: true,
    },
    {
      icon: '🌉',
      title: '7-Day Bridge Loan',
      description: 'Zero-interest loan to close existing loan',
      highlight: true,
    },
    {
      icon: '🔒',
      title: '24-Month Rate Lock',
      description: 'Guaranteed rate for 2 years',
      highlight: false,
    },
    {
      icon: '🛡️',
      title: 'Transit Insurance',
      description: 'Full coverage during gold movement',
      highlight: false,
    },
    {
      icon: '💵',
      title: 'Cashback Offer',
      description: 'Rs 1,000-3,000 for loans above Rs 5L',
      highlight: false,
    },
  ],
};

export const bridgeLoanMechanism = {
  title: 'The Bridge Loan: Eliminating Switching Friction',
  innovation: 'Key Innovation',
  withoutBridge: {
    title: 'Without Bridge Loan',
    color: 'danger',
    totalTime: '10-14 days',
    anxietyLevel: 'HIGH',
    successRate: '40-50%',
    steps: [
      { day: 'Day 1', action: 'Customer decides to switch', icon: '🤔' },
      { day: 'Day 2-3', action: 'Visits current lender for closure amount', icon: '🏢' },
      { day: 'Day 4-7', action: 'Arranges funds to close loan', icon: '💸' },
      { day: 'Day 8', action: 'Redeems gold from current lender', icon: '🥇' },
      { day: 'Day 9', action: 'Transits gold (personal risk)', icon: '😰' },
      { day: 'Day 10+', action: 'Visits new lender, waits for disbursement', icon: '⏳' },
    ],
  },
  withBridge: {
    title: 'With Kotak Switch Assist',
    color: 'success',
    totalTime: '3 days',
    anxietyLevel: 'LOW',
    successRate: '75-85%',
    steps: [
      { day: 'Day 0', action: 'Customer shows interest (app/branch)', icon: '📱' },
      { day: 'Day 0', action: 'Pre-approval issued (existing customer)', icon: '✅' },
      { day: 'Day 1', action: 'Bridge loan sanctioned (ZERO interest)', icon: '🌉' },
      { day: 'Day 1', action: 'Funds transferred to close existing loan', icon: '💰' },
      { day: 'Day 2-3', action: 'Customer redeems gold with Kotak-insured transit', icon: '🛡️' },
      { day: 'Day 3', action: 'Gold pledged at Kotak, full loan replaces bridge', icon: '🎉' },
    ],
  },
  keyInsight: 'The bridge loan is low-cost to us but transformational for conversion.',
};

export const switchAssistService = {
  title: 'Switch Assist: End-to-End Switching Service',
  components: [
    {
      step: 1,
      title: 'Pre-Approval',
      description: 'Instant offer based on Kotak relationship',
      costToKotak: 'Nil (system cost)',
      icon: '✅',
      time: 'Instant',
    },
    {
      step: 2,
      title: 'Bridge Financing',
      description: '7-day zero-interest loan to close existing loan',
      costToKotak: 'Interest float',
      icon: '🌉',
      time: 'Same day',
    },
    {
      step: 3,
      title: 'Transit Insurance',
      description: 'Coverage during gold movement',
      costToKotak: 'Rs 100-200 per switch',
      icon: '🛡️',
      time: 'Included',
    },
    {
      step: 4,
      title: 'Dedicated Support',
      description: 'Single point of contact for switching process',
      costToKotak: 'Staff time',
      icon: '🤝',
      time: 'Throughout',
    },
  ],
  customerQuote: '"I was worried about moving my gold. Kotak handled everything."',
  totalCost: 'Each component costs us very little but addresses specific friction points.',
};

export const incentiveStructure = {
  recommended: [
    {
      incentive: 'Processing Fee Waiver',
      frictionAddressed: 'Switching cost',
      customerValue: 'Rs 3,000-15,000',
      kotakCost: 'Direct cost',
      effectiveness: 'HIGH',
    },
    {
      incentive: '7-Day Interest-Free Bridge',
      frictionAddressed: 'Cash flow gap',
      customerValue: 'Rs 2,000-10,000',
      kotakCost: 'Float cost (~Rs 500)',
      effectiveness: 'VERY HIGH',
    },
    {
      incentive: 'Transit Insurance',
      frictionAddressed: 'Safety anxiety',
      customerValue: 'Peace of mind',
      kotakCost: 'Rs 100-200',
      effectiveness: 'HIGH',
    },
    {
      incentive: 'Rate Lock (24 months)',
      frictionAddressed: 'Future uncertainty',
      customerValue: 'Predictability',
      kotakCost: 'Margin risk',
      effectiveness: 'MEDIUM',
    },
  ],
  avoid: [
    {
      incentive: 'Generic cashback (no conditions)',
      reason: 'Attracts deal-seekers, not switchers',
    },
    {
      incentive: 'Gift cards',
      reason: 'No connection to gold loan value proposition',
    },
    {
      incentive: 'Lucky draws',
      reason: 'Dilutes serious positioning',
    },
    {
      incentive: 'High referral bonuses',
      reason: 'Encourages fraud',
    },
  ],
};

export const campaignConcepts = [
  {
    name: 'The Smart Switch',
    target: 'P1: High-Value Rational',
    segment: 'P1: High-Value',
    coreMessage: 'Calculation showing savings',
    cta: '"Calculate your savings. Make the smart switch."',
    tagline: 'Calculate your savings. Make the smart switch.',
    channels: ['Digital', 'Print (business dailies)', 'RM outreach'],
    tone: 'Rational, data-driven',
    color: '#ED1C24',
    icon: '📊',
  },
  {
    name: 'Gold Deserves a Bank',
    target: 'P2: Trust-Seekers',
    segment: 'P2: Trust-Seekers',
    coreMessage: 'Safety and security',
    cta: '"Your gold deserves bank-level protection"',
    tagline: 'Your gold deserves bank-level protection',
    channels: ['Digital video', 'Regional TV'],
    tone: 'Reassuring, trustworthy',
    color: '#003874',
    icon: '🛡️',
  },
  {
    name: 'Shakti Gold',
    target: 'P3: Women Entrepreneurs',
    segment: 'P3: Women',
    coreMessage: 'Empowerment and dignity',
    cta: '"Financial independence starts here"',
    tagline: 'Financial independence starts here',
    channels: ['BC network', 'Women\'s groups', 'Social media'],
    tone: 'Empowering, respectful',
    color: '#EC4899',
    icon: '💪',
  },
  {
    name: 'Gold in Clicks',
    target: 'P4: Young Professionals',
    segment: 'P4: Digital',
    coreMessage: 'Digital-first convenience',
    cta: '"Turn idle gold into instant opportunity"',
    tagline: 'Turn idle gold into instant opportunity',
    channels: ['Kotak 811 app', 'Instagram', 'YouTube'],
    tone: 'Modern, quick',
    color: '#3B82F6',
    icon: '📱',
  },
];

export const implicitMessaging = [
  {
    claim: '"Your gold deserves vault-level protection"',
    surfaceMeaning: 'We have great vaults',
    impliedContrast: 'Others may not',
  },
  {
    claim: '"Zero regulatory concerns. Ever."',
    surfaceMeaning: 'We\'re compliant',
    impliedContrast: 'Others have had issues',
  },
  {
    claim: '"Access your gold when YOU need it"',
    surfaceMeaning: 'Convenient access',
    impliedContrast: 'Others may restrict',
  },
  {
    claim: '"Transparent valuation - watch it happen"',
    surfaceMeaning: 'Open process',
    impliedContrast: 'Others may hide',
  },
  {
    claim: '"Already meeting 2026 standards today"',
    surfaceMeaning: 'We\'re ahead',
    impliedContrast: 'Others are scrambling',
  },
  {
    claim: '"Bank-backed, not just promises"',
    surfaceMeaning: 'Regulatory protection',
    impliedContrast: 'NBFCs aren\'t banks',
  },
];
