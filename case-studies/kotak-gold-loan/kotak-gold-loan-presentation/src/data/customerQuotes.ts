// Customer quotes and pain points from research

export const lakshmiPersona = {
  name: 'Lakshmi',
  title: 'Small Business Owner in Chennai',
  avatar: '👩‍💼',
  business: 'Textile business',
  currentProvider: 'NBFC',
  currentRate: 18,
  loanAmount: 500000,
  annualInterest: 90000,
  potentialSavingsRate: 10,
  potentialSavings: 40000,
  quote: "I pay Rs 90,000 a year in gold loan interest. I didn't know I had options.",
  backstory: 'Has been with the same NBFC for 5 years out of habit and fear of switching',
};

export const mouthshutReviews = [
  {
    provider: 'Muthoot Finance',
    rating: 1,
    maxRating: 5,
    title: 'Excessive Penalties',
    quote: '"Unjustifiable charge... excessive penalty for a single day delay. They treat loyal customers like criminals."',
    date: '2024',
    painPoint: 'Penalties',
    verified: true,
  },
  {
    provider: 'Muthoot Finance',
    rating: 1,
    maxRating: 5,
    title: 'Useless App',
    quote: '"Part payment feature totally useless. App crashes every time I try to pay. Had to visit branch 3 times."',
    date: '2024',
    painPoint: 'Technology',
    verified: true,
  },
  {
    provider: 'Muthoot Finance',
    rating: 2,
    maxRating: 5,
    title: 'Dismissive Staff',
    quote: '"Dismissive attitude when I raised a complaint about wrong interest calculation. No one takes responsibility."',
    date: '2024',
    painPoint: 'Service',
    verified: true,
  },
  {
    provider: 'Manappuram',
    rating: 1,
    maxRating: 5,
    title: 'Hidden Charges',
    quote: '"They never told me about the processing fee, insurance, and storage charges. Final cost was 30% higher than quoted."',
    date: '2024',
    painPoint: 'Transparency',
    verified: true,
  },
  {
    provider: 'IIFL',
    rating: 1,
    maxRating: 5,
    title: 'Gold Access Issues',
    quote: '"Needed my gold for a family wedding. They made me wait 2 weeks and asked for more documents. Felt trapped."',
    date: '2024',
    painPoint: 'Access',
    verified: true,
  },
  {
    provider: 'NBFC General',
    rating: 2,
    maxRating: 5,
    title: 'Auction Fear',
    quote: '"My neighbor\'s gold was auctioned for missing one EMI. Now I live in constant fear of the same happening to me."',
    date: '2024',
    painPoint: 'Security',
    verified: true,
  },
];

export const painCategories = [
  {
    category: 'Interest Rate Burden',
    icon: '💸',
    painPoints: [
      { issue: 'High rates (18-27%)', impact: 'Rs 90,000+ annual interest on Rs 5L loan' },
      { issue: 'Hidden charges', impact: 'Unexpected fees at renewal' },
      { issue: 'Compounding interest', impact: 'Interest on interest accumulation' },
    ],
    kotakSolution: '40-50% savings with transparent pricing',
  },
  {
    category: 'Service & Penalty Issues',
    icon: '😤',
    painPoints: [
      { issue: 'Excessive penalties', impact: 'Harsh treatment for minor delays' },
      { issue: 'App payment issues', impact: 'Forced branch visits' },
      { issue: 'Dismissive staff', impact: 'Complaints ignored' },
    ],
    kotakSolution: 'Grace periods, digital payments, premium service',
  },
  {
    category: 'Trust & Security Concerns',
    icon: '😰',
    painPoints: [
      { issue: 'Gold safety fears', impact: 'Fraud cases in news' },
      { issue: 'Auction anxiety', impact: 'Stories of unfair auctions' },
      { issue: 'Access restrictions', impact: 'Difficulty retrieving gold' },
    ],
    kotakSolution: 'Bank vault security, 30-day notice, guaranteed access',
  },
  {
    category: 'Regulatory Uncertainty',
    icon: '⚠️',
    painPoints: [
      { issue: 'NBFC compliance issues', impact: 'Providers facing RBI action' },
      { issue: 'Service disruptions', impact: 'Doorstep services unavailable' },
      { issue: 'Subsidiary problems', impact: 'Group companies banned' },
    ],
    kotakSolution: 'Zero violations, bank-grade compliance',
  },
];

export const switchingBarriers = [
  {
    barrier: 'Emotional attachment to current lender',
    impactScore: 7,
    description: 'Years of relationship, familiarity with branch staff',
    mitigation: '"Your gold deserves bank protection"',
  },
  {
    barrier: 'Fear of transit during switch',
    impactScore: 9,
    description: 'Anxiety about moving gold from one location to another',
    mitigation: 'Kotak Switch Assist with insured transit',
  },
  {
    barrier: 'Cash flow gap during redemption',
    impactScore: 8,
    description: 'Need money to close existing loan before getting new one',
    mitigation: '7-day bridge loan at zero interest',
  },
  {
    barrier: 'Multiple branch visits required',
    impactScore: 6,
    description: 'Time off work, coordination hassles',
    mitigation: 'Pre-approved offers for Kotak customers',
  },
  {
    barrier: 'Uncertainty about new process',
    impactScore: 5,
    description: 'Unknown procedures, documentation requirements',
    mitigation: 'Video tutorials, dedicated support',
  },
];

export const trustTriggers = [
  {
    trigger: '"I saw news about gold loan problems"',
    psychology: 'Fear of inaccessibility',
    kotakResponse: '"Access your gold when YOU need it"',
  },
  {
    trigger: '"My neighbor\'s gold was auctioned unfairly"',
    psychology: 'Auction anxiety',
    kotakResponse: '"Transparent auctions with 30-day notice"',
  },
  {
    trigger: '"The branch always has long queues"',
    psychology: 'Service frustration',
    kotakResponse: '"Premium service, dedicated counters"',
  },
  {
    trigger: '"I don\'t understand the charges"',
    psychology: 'Confusion, mistrust',
    kotakResponse: '"What you see is what you pay"',
  },
];

export const keyInsight = {
  headline: 'These customers aren\'t loyal — they\'re trapped.',
  subheadline: 'They\'re waiting for someone to offer them a better option.',
  callToAction: 'Kotak can be that option.',
};
