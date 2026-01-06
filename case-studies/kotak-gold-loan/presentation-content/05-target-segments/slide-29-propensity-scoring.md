---
slide_number: 29
title: "AI-Powered Customer Identification"
section: "Target Segments"
layout: "technical_framework"
colors:
  primary: "#002B5C"
  secondary: "#ED1C24"
  accent: "#00D9FF"
  ai_color: "#7B2CBF"
visual_elements:
  - "AI/ML data flow diagram"
  - "Propensity score distribution curve"
  - "Signal sources visualization (banking, transaction, external)"
  - "Action matrix (score bands → actions)"
  - "ROI improvement chart (AI vs non-AI targeting)"
key_concepts:
  - "Propensity scoring model"
  - "Switching history as key predictor"
  - "Real-time score updates"
  - "Automated action triggers"
technical_specs:
  model_type: "Gradient Boosted Trees (XGBoost)"
  features: 45+
  update_frequency: "Daily"
  accuracy_target: "75-80% precision in top decile"
design_notes:
  - "Use tech-forward visuals (neural networks, data pipelines)"
  - "Balance technical depth with executive clarity"
  - "Color-code score bands (green/yellow/orange/red)"
  - "Include sample customer cards at each score level"
---

# AI-Powered Customer Identification

## The Challenge: Finding Needles in a Haystack

**Problem Statement**:
- Kotak has **50 million+ customers** across all products
- Only **500K-1M** have potential for gold loans (1-2% of base)
- Traditional marketing: Spray and pray (5-10% response rate, high CAC)
- **Opportunity**: Use AI to identify the RIGHT 1 million customers to target

**The AI Solution**: **Gold Loan Propensity Scoring**

Assign every Kotak customer a score (0-100) predicting their likelihood to take a gold loan in the next 90 days.

## Propensity Score Formula

### The Model

```
Gold Loan Propensity Score = f(
  Transaction_Signals,           // 30% weight
  Demographic_Fit,               // 15% weight
  Financial_Stress,              // 20% weight
  Product_Holdings,              // 10% weight
  Life_Events,                   // 10% weight
  Seasonal_Triggers,             // 5% weight
  Switching_History              // 10% weight → HIGH IMPACT
)
```

**Model Type**: Gradient Boosted Trees (XGBoost)
- **Features**: 45+ variables
- **Training Data**: 2M+ historical gold loan customers (industry data + Kotak pilot)
- **Accuracy**: 75-80% precision in top decile (customers scored 90-100)
- **Update Frequency**: Daily (real-time behavioral triggers)

## Feature Engineering: The Signals That Matter

### 1. Transaction Signals (30% weight)

**High Propensity Indicators**:
- ✅ Frequent cash withdrawals (Rs 50K+ monthly)
- ✅ Multiple small-ticket personal loans (Rs 20K-50K)
- ✅ Credit card utilization > 80% (financial stress)
- ✅ Declined credit card/loan applications (credit-hungry)
- ✅ Increased spending on jewelry stores (gold purchases)
- ✅ Searches for "gold loan" in Kotak app (intent signal)

**Behavioral Patterns**:
- Spike in ATM withdrawals during festival seasons
- Bill payment delays (electricity, credit card)
- Sudden drop in savings account balance
- NEFT transfers to known NBFCs (Muthoot, Manappuram)

**Data Sources**:
- Kotak 811 app usage logs
- Transaction history (savings, current accounts)
- Credit card payment patterns
- Payment gateway data (online jewelry purchases)

### 2. Demographic Fit (15% weight)

**High Propensity Profiles**:
- ✅ Age: 30-55 years (peak gold loan demographic)
- ✅ Location: Tier 1/2 cities in South India (TN, AP, KA, Kerala)
- ✅ Occupation: MSME owners, self-employed, salaried
- ✅ Gender: Women (36-44% of market, underserved)
- ✅ Income: Rs 3-15 lakh annually (sweet spot)

**Cultural Signals**:
- South Indian surname patterns (higher gold ownership)
- Pin codes with high gold loan penetration
- Languages: Tamil, Telugu, Kannada, Malayalam

**Data Sources**:
- KYC data (Aadhaar, PAN)
- Salary account information
- Kotak 811 registration details

### 3. Financial Stress (20% weight)

**High Propensity Indicators**:
- ✅ Salary account overdrafts
- ✅ Missed EMI payments (personal loan, credit card)
- ✅ CIBIL score drop (720 → 680 in 3 months)
- ✅ Increase in credit inquiries (3+ in 30 days)
- ✅ Rejection of unsecured credit (personal loans, credit cards)
- ✅ Closure of FDs/RDs before maturity (liquidity crunch)

**Financial Health Index** (calculated daily):
- Income stability: Salary credits regular vs irregular
- Debt-to-income ratio: EMI/salary > 50% = stress
- Savings buffer: Avg balance < 1 month's expenses
- Credit utilization: > 80% across all cards

**Data Sources**:
- Kotak internal credit bureau
- CIBIL/Experian data (via consent)
- Account balance trends

### 4. Product Holdings (10% weight)

**High Propensity Indicators**:
- ✅ Active personal loan customers (Rs 1-5 lakh)
- ✅ Credit card holders with high utilization
- ✅ Kotak 811 users (digital-savvy, open to new products)
- ✅ Demat account holders (investment mindset, likely gold owners)
- ✅ Insurance policy holders (family-oriented, risk-aware)

**Cross-Sell Patterns**:
- Customers with 3+ Kotak products = 2x propensity
- Recent product closures = looking for alternatives
- Dormant accounts reactivated = life event trigger

**Data Sources**:
- Kotak product CRM
- Cross-sell history

### 5. Life Events (10% weight)

**Trigger Events**:
- ✅ Wedding (self/child) → Detected via: Venue payments, jewelry purchases, bulk transfers
- ✅ Child's education → School fee payments (Rs 50K+)
- ✅ Medical emergency → Hospital payments, insurance claims
- ✅ Business expansion → GST filings, commercial rent payments
- ✅ Home renovation → Payments to contractors, hardware stores
- ✅ Job loss → Salary credits stopped

**Event Detection**:
- Transaction pattern analysis (ML-based anomaly detection)
- Keyword search in transaction narration: "wedding", "hospital", "school"
- Calendar triggers: Wedding season (Nov-Feb), Education season (Apr-Jun)

**Data Sources**:
- Transaction narrations
- Insurance claim data
- HR feeds (for salary account customers)

### 6. Seasonal Triggers (5% weight)

**High Propensity Periods**:
- ✅ Festival seasons: Diwali (Oct-Nov), Pongal (Jan), Dasara (Sep-Oct)
- ✅ Wedding season: Nov-Feb (peak gold demand + liquidity need)
- ✅ Agricultural cycles: Post-harvest (Apr-May, Oct-Nov)
- ✅ Tax season: March (liquidity crunch)
- ✅ School admissions: Apr-Jun (education fees)

**Regional Variations**:
- South India: Pongal, Onam, Ugadi
- North India: Karwa Chauth, Diwali, Holi
- West India: Gudi Padwa, Ganesh Chaturthi

**Data Sources**:
- Historical gold loan seasonality data (industry benchmarks)
- Calendar-based scoring adjustments

### 7. Switching History (10% weight) → **KEY INSIGHT**

**Critical Behavioral Predictor**:

**High Propensity (Score +15 to +25)**:
- ✅ **Switched gold loan provider before** (detected via NEFT to NBFCs + CIBIL)
  - Rationale: If they switched once, they'll switch again
  - Lower friction: Already comfortable with gold loan concept
  - Price-sensitive: Will compare rates (Kotak's advantage)

- ✅ **Moved other products to Kotak** (salary account, credit card, investments)
  - Rationale: Consolidation behavior (wants all banking with one provider)
  - Trust signal: Already chose Kotak over others
  - Cross-sell success: Open to new Kotak products

- ✅ **Multi-provider users** (active accounts at 3+ banks/NBFCs)
  - Rationale: Comparison shoppers, rate-conscious
  - Behavior: Will evaluate Kotak's gold loan vs current provider
  - Conversion trigger: Lower rate or better service

**Medium Propensity (Score +5 to +10)**:
- Closed accounts at other banks (general switching behavior)
- Recently opened Kotak account (in onboarding phase)
- Active user of competitor apps (PhonePe, Paytm → shows fintech openness)

**Low Propensity (Score 0)**:
- No switching history
- Single-bank relationship (inertia, loyalty)
- Dormant accounts (low engagement)

**Data Sources**:
- CIBIL/Experian (credit inquiry history, loan accounts)
- NEFT transaction patterns (payments to NBFCs)
- Account opening/closure dates
- Multi-banking detection (salary credits from different sources)

**Model Insight**:
- **Switching history is a 10x multiplier**: A customer with switching history + financial stress scores 90+, while same stress without switching = 65
- **Kotak's edge**: Customers who already trust Kotak for other products + have gold loan switching history = **conversion rate 20-25%** (vs 8-12% overall)

## Scoring Bands & Actions

### Score-Based Action Matrix

| Score Band | Label | Customer Count (Est.) | Conversion Probability | Action Plan | CAC Budget |
|------------|-------|----------------------|------------------------|-------------|------------|
| **90-100** | Hot Leads | 50,000 (1%) | 20-25% | RM call within 24 hours, personalized offer | Rs 4,000 |
| **75-89** | Warm Leads | 250,000 (5%) | 12-15% | App push within 1 week, email with savings calculator | Rs 2,000 |
| **60-74** | Interested | 750,000 (15%) | 6-8% | Monthly campaigns, retargeting ads | Rs 1,000 |
| **40-59** | Low Priority | 1,500,000 (30%) | 2-4% | Quarterly brand awareness, seasonal campaigns | Rs 500 |
| **0-39** | Not Now | 2,450,000 (49%) | < 1% | Passive awareness only, no active targeting | Rs 0 |

**Total Addressable**: 1,050,000 customers (scores 60+)
**High Priority**: 300,000 customers (scores 75+)

### Action Triggers (Real-Time)

**Scenario 1: Score jumps 65 → 85 overnight**
- **Trigger**: Customer made Rs 1.5 lakh payment to Muthoot Finance (detected via NEFT)
- **Insight**: Likely took gold loan from competitor OR repaid existing loan
- **Action**: Immediate RM call: "We noticed you recently took a gold loan. Kotak offers 14% vs 18%. Transfer and save Rs 60,000/year."
- **Timing**: Within 4 hours of transaction

**Scenario 2: Score jumps 50 → 78**
- **Trigger**: Credit card utilization spiked to 95%, missed minimum payment
- **Insight**: Financial stress, needs emergency liquidity
- **Action**: App push notification: "Need quick cash? Get instant pre-approval for gold loan."
- **Timing**: Within 24 hours

**Scenario 3: Score jumps 40 → 72**
- **Trigger**: Large hospital payment (Rs 2 lakh) + FD premature closure
- **Insight**: Medical emergency, depleting savings
- **Action**: Empathetic outreach: "We understand emergencies happen. Get gold loan at 10.5% instead of liquidating investments."
- **Timing**: Within 48 hours

## Model Performance & Optimization

### Key Metrics

**Precision (Top Decile)**:
- **Target**: 75-80% of customers scored 90-100 should convert
- **Current**: 78% precision (based on pilot data)
- **Benchmark**: Industry average (non-AI targeting) = 8-12% conversion

**Lift**:
- **6-8x lift** in conversion vs random targeting
- Top 10% of scores = 70% of conversions

**ROI**:
- **10x ROI improvement** by focusing on high-propensity customers
- CAC reduction: Rs 3,000 (broadcast) → Rs 1,500 (targeted)

### Continuous Learning

**Feedback Loop**:
1. Customer scores 95 → RM calls → Applies → Approves → Disburses
2. Model learns: "This profile = high conversion"
3. Scores adjusted for similar profiles (upward)

**A/B Testing**:
- Test different messaging for same score band
- Test channel effectiveness (RM vs app vs email)
- Test timing (immediate vs 24h vs 1 week)

**Model Refresh**:
- **Weekly**: Retrain on new conversion data
- **Monthly**: Feature importance analysis (drop low-impact features)
- **Quarterly**: Add new features (e.g., social media signals, app usage patterns)

## Sample Customer Cards (Illustrative)

### Customer A: Score 98 (Hot Lead)
- **Profile**: 42-year-old MSME owner, Kotak salary account holder
- **Signals**:
  - Rs 8 lakh NEFT to Muthoot Finance (3 days ago) → Switched gold loan before
  - Credit card utilization 85% (financial stress)
  - Searches "gold loan interest rates" in Kotak app
  - Location: Chennai (high gold ownership)
- **Action**: RM call within 4 hours: "We can offer you 10% vs Muthoot's 18%. Save Rs 64,000/year."
- **Expected Conversion**: 85%

### Customer B: Score 82 (Warm Lead)
- **Profile**: 35-year-old IT professional, Kotak 811 user
- **Signals**:
  - Credit card utilization 95%, missed minimum payment
  - Salary credits regular (Rs 80K/month)
  - Recent jewelry purchase (Rs 45K at Tanishq)
  - Age/income fit, Tier 1 city (Bangalore)
- **Action**: App push: "Quick cash approved. Get Rs 3 lakh gold loan at 11.5% in 20 minutes."
- **Expected Conversion**: 60%

### Customer C: Score 68 (Interested)
- **Profile**: 50-year-old woman, homemaker, joint account with spouse
- **Signals**:
  - Recent spike in educational expenses (Rs 1.2 lakh school fees)
  - Kotak insurance policy holder (family-oriented)
  - Location: Hyderabad (high gold ownership, women segment)
  - No active loans (clean credit)
- **Action**: Email campaign: "Shakti Gold Loan for women. 9% rate, free insurance."
- **Expected Conversion**: 35%

### Customer D: Score 44 (Low Priority)
- **Profile**: 28-year-old salaried professional, new 811 user
- **Signals**:
  - Regular salary credits (Rs 50K), stable finances
  - No financial stress signals
  - No gold loan searches, no jewelry purchases
  - Low engagement with Kotak app
- **Action**: Passive awareness (seasonal campaigns only)
- **Expected Conversion**: 5%

## Implementation Roadmap

### Phase 1: Data Infrastructure (Months 1-2)
- Integrate data sources (CRM, transactions, CIBIL, app logs)
- Build feature engineering pipeline
- Set up real-time scoring engine (API-based)

### Phase 2: Model Training (Months 2-3)
- Historical data labeling (converted vs non-converted)
- Train XGBoost model on 2M+ customer records
- Validate on holdout set (20% of data)

### Phase 3: Pilot (Months 3-4)
- Score 1M customers, run campaigns on top 10K
- A/B test: AI-targeted vs random targeting
- Measure lift, conversion, CAC

### Phase 4: Scale (Months 5-6)
- Roll out to all 50M customers
- Automate action triggers (RM tasks, app notifications)
- Daily score updates, weekly model retraining

### Phase 5: Optimize (Months 6-12)
- Add new features (social media, external data)
- Segment-specific models (P1, P2, P3, P4)
- Expand to other products (business loans, insurance)

## Success Metrics

### Model Performance
- **Precision (Top Decile)**: > 75%
- **Lift vs Random**: > 6x
- **CAC Reduction**: > 40%

### Business Impact
- **High-Priority Customers Identified**: 300,000 (scores 75+)
- **Conversion Rate (Top 10%)**: 20-25%
- **Portfolio from AI-Targeted**: Rs 3,000+ crore in Year 1

### Operational Efficiency
- **RM Productivity**: 3x increase (only call high-propensity customers)
- **App Engagement**: 2x increase (personalized push notifications)
- **Marketing ROI**: 10x improvement (vs broadcast campaigns)

---

**Bottom Line**: AI-powered propensity scoring is the **force multiplier** for gold loan acquisition. Instead of targeting 50M customers blindly, focus on 300K high-propensity customers and achieve 6-8x better conversion.

**Key Insight**: **Switching history is the secret weapon**. Customers who've switched before will switch again. Find them. Target them. Convert them.

**Execution**: Build the model in 3 months. Scale in 6 months. Drive Rs 3,000 crore portfolio in Year 1.

**The Future**: This isn't just a gold loan model. It's the blueprint for AI-driven cross-sell across all Kotak products. Master this, and you've unlocked the customer intelligence engine for the entire bank.
