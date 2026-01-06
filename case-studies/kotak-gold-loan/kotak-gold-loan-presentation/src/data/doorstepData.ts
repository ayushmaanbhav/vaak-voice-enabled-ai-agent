// Doorstep service pilot data

export const doorstepOpportunity = {
  title: 'Doorstep Gold Loan: Premium Convenience',
  regulatoryStatus: 'Allowed with restrictions',
  precedent: 'Federal Bank operates via Rupeek partnership',
  requirements: [
    { requirement: 'Gold custody', rbiPosition: 'Cannot be stored in BC custody - must reach bank vault same day' },
    { requirement: 'Appraisal', rbiPosition: 'Should not be delegated entirely to BC' },
    { requirement: 'Transport', rbiPosition: 'Secure, insured, time-bound transfer to branch' },
    { requirement: 'Model', rbiPosition: 'Partner model (Rupeek) or owned field executives' },
  ],
};

export const doorstepEconomics = {
  costPerTransaction: {
    total: 850,
    breakdown: [
      { component: 'Field Executive', cost: 400, icon: '👤' },
      { component: 'Vehicle + Fuel', cost: 200, icon: '🚗' },
      { component: 'XRF Device (amortized)', cost: 100, icon: '🔬' },
      { component: 'Transit Insurance', cost: 50, icon: '🛡️' },
      { component: 'Technology', cost: 100, icon: '📱' },
    ],
  },
  viabilityByTicketSize: [
    { ticketSize: '25,000', costPercent: 3.4, verdict: 'NOT VIABLE', note: 'Branch/digital only', color: 'danger' },
    { ticketSize: '50,000', costPercent: 1.7, verdict: 'MARGINAL', note: 'Promotional campaigns', color: 'warning' },
    { ticketSize: '1,00,000', costPercent: 0.85, verdict: 'VIABLE', note: 'Standard offer', color: 'success' },
    { ticketSize: '2,00,000+', costPercent: 0.4, verdict: 'HIGHLY VIABLE', note: 'Premium service', color: 'gold' },
  ],
  recommendation: 'Doorstep service for Rs 50,000+ ticket sizes in 5 Tier 1 pilot cities',
};

export const pilotCities = [
  {
    city: 'Mumbai',
    branches: 45,
    rationale: 'High density, traffic constraints, premium segment',
    phase: 1,
    targetLoans: 500,
  },
  {
    city: 'Delhi NCR',
    branches: 35,
    rationale: 'Largest metro, safety concerns, working professionals',
    phase: 2,
    targetLoans: 400,
  },
  {
    city: 'Bangalore',
    branches: 28,
    rationale: 'Tech-savvy customers, high average ticket',
    phase: 2,
    targetLoans: 350,
  },
  {
    city: 'Hyderabad',
    branches: 22,
    rationale: 'Strong gold culture, growing market',
    phase: 2,
    targetLoans: 300,
  },
  {
    city: 'Pune',
    branches: 18,
    rationale: 'Affluent suburbs, MSME concentration',
    phase: 3,
    targetLoans: 250,
  },
];

export const pilotPhases = [
  {
    phase: 1,
    period: 'M1-M3',
    city: 'Mumbai',
    targetLoans: 500,
    investment: 'Rs 1.5 Cr',
    focus: 'Process refinement, unit economics validation',
  },
  {
    phase: 2,
    period: 'M4-M6',
    cities: 'Delhi + Bangalore',
    targetLoans: 2000,
    investment: 'Rs 1.5 Cr',
    focus: 'Scale testing, operational efficiency',
  },
  {
    phase: 3,
    period: 'M7-M12',
    cities: 'All 5 cities',
    targetLoans: 10000,
    investment: 'Rs 2 Cr',
    focus: 'Full rollout if Phase 2 successful',
  },
];

export const successCriteria = [
  { metric: 'Customer Satisfaction (NPS)', target: '>70', current: '-' },
  { metric: 'Cost per Acquisition', target: '<Rs 2,500', current: '-' },
  { metric: 'Conversion Rate', target: '>60%', current: '-' },
  { metric: 'Zero Security Incidents', target: '100%', current: '-' },
];

export const doorstepVsBranch = [
  { attribute: 'Time for Customer', doorstep: '30 min at home', branch: '2-4 hours', advantage: 'doorstep' },
  { attribute: 'Security Perception', doorstep: 'Insured transit', branch: 'Self-carry risk', advantage: 'doorstep' },
  { attribute: 'Cost to Bank', doorstep: 'Rs 850/transaction', branch: '~Rs 200/transaction', advantage: 'branch' },
  { attribute: 'Ticket Size Sweet Spot', doorstep: 'Rs 1L+', branch: 'Any', advantage: 'varies' },
  { attribute: 'Customer Segment', doorstep: 'Premium, time-poor', branch: 'All segments', advantage: 'varies' },
];

// Consolidated doorstep pilot for easier slide access
export const doorstepPilot = {
  pilotCities: [
    { city: 'Mumbai', branches: 45, targetCustomers: '1,500', phase: 1 },
    { city: 'Chennai', branches: 32, targetCustomers: '1,200', phase: 2 },
    { city: 'Hyderabad', branches: 28, targetCustomers: '1,000', phase: 2 },
    { city: 'Pune', branches: 22, targetCustomers: '900', phase: 3 },
    { city: 'Bangalore', branches: 30, targetCustomers: '900', phase: 3 },
  ],
  totalTarget: '5,500',
  totalPhases: 3,
  duration: '12 months',
  costBreakdown: [
    { item: 'Field Executive', cost: 'Rs 400', icon: '👤' },
    { item: 'Vehicle + Fuel', cost: 'Rs 200', icon: '🚗' },
    { item: 'XRF Device (amortized)', cost: 'Rs 100', icon: '🔬' },
    { item: 'Transit Insurance', cost: 'Rs 50', icon: '🛡️' },
    { item: 'Technology', cost: 'Rs 100', icon: '📱' },
  ],
  totalCostPerTransaction: 'Rs 850',
  breakEvenTicket: 34000,
};

export const pilotInvestment = {
  total: '5 Cr',
  breakdown: [
    { item: 'Field Executive Team (20 executives x 5 cities)', cost: '1.5 Cr' },
    { item: 'Vehicles & Equipment', cost: '1 Cr' },
    { item: 'Technology Platform', cost: '80 lakh' },
    { item: 'Insurance & Compliance', cost: '70 lakh' },
    { item: 'Training & Operations', cost: '50 lakh' },
    { item: 'Buffer', cost: '50 lakh' },
  ],
};
