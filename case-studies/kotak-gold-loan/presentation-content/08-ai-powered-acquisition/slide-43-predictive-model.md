---
slide_number: 43
section: "AI-Powered Acquisition"
title: "Finding Gold Loan Customers in 53 Million"
subtitle: "Predictive Acquisition Model for Competitor Gold Loan Identification"
layout: "solution-deep-dive"
colors:
  primary: "#ED1C24"
  secondary: "#002B5C"
  accent: "#C9A227"
visual_elements:
  - "Data signal flow diagram (transaction patterns → AI model → predictions)"
  - "Sample model output JSON card"
  - "Performance lift comparison table"
  - "Privacy compliance badge (DPDP Act 2023)"
key_metrics:
  overall_conversion_lift: "16x"
  cac_reduction: "75%"
  campaign_response_lift: "4x"
---

# Finding Gold Loan Customers in 53 Million

## The Acquisition Challenge

**The Question**: Which of Kotak's 53 million customers currently have active gold loans with competitors?

### Why This Matters

**Current Approach (Spray & Pray)**
- Mass marketing to all customers
- 0.12% conversion rate (600 conversions per 500K outreach)
- Rs 2,000 CAC (customer acquisition cost)
- 98% wasted effort and budget

**The Opportunity**
- Estimated 8-10 lakh Kotak customers have competitor gold loans
- These customers are PROVEN gold loan users
- Lower risk, higher intent, easier to convert
- Need: Identify them WITHOUT violating privacy

## The Solution: Transaction Pattern AI

### Detection Signals (High → Low Confidence)

#### HIGH Confidence Signals

**1. Recurring Payments to Known Gold Lenders**
```
Example Transaction Pattern:
- Rs 4,500 to "Muthoot Finance" (monthly, 12 months)
- Rs 8,200 to "Manappuram Finance" (monthly, 8 months)
- Rs 15,000 to "IIFL Gold Loan" (quarterly, 4 payments)

Analysis: Fixed recurring amount → likely interest payment
         Known gold lender → HIGH confidence gold loan
         Duration 8-12 months → typical gold loan tenure
```

**2. Payment Description Keywords**
```
UPI/NEFT descriptions containing:
- "GL" (gold loan abbreviation)
- "Interest payment"
- "Gold loan EMI"
- Merchant names: "Muthoot", "Manappuram", "IIFL", "Federal"

Analysis: Explicit mention → VERY HIGH confidence
```

#### MEDIUM Confidence Signals

**3. UPI Merchant Category Codes (MCC)**
```
Payments to MCC 6012 (Financial Institutions - Gold Loans)
Payments to MCC 6051 (Non-bank Financial Institutions)

Analysis: Right category, but could be other loan types
         Combined with amount/frequency → MEDIUM confidence
```

**4. Amount Pattern Analysis**
```
Typical gold loan interest patterns:
- Rs 2,000 - 5,000/month → Rs 1-2 lakh loan @ 12-15% p.a.
- Rs 5,000 - 15,000/month → Rs 3-6 lakh loan @ 12-15% p.a.
- Rs 15,000 - 50,000/month → Rs 8-20 lakh loan @ 12-15% p.a.

Analysis: Amount + regularity suggests loan interest
         But could be rent, subscription, etc.
         MEDIUM confidence alone, HIGH when combined with other signals
```

### Model Architecture

**Input Features (30+ variables):**
- Transaction frequency (daily/weekly/monthly)
- Transaction amounts (mean, median, variance)
- Merchant names (NLP entity extraction)
- Payment descriptions (keyword matching)
- MCC codes (category classification)
- Account balance patterns (spike before payment → loan disbursement?)
- Customer demographics (age, location, income bracket)
- Existing Kotak products (savings, FD → gold loan affinity)

**Model Type:** Gradient Boosted Trees (XGBoost)
- Training data: 2M anonymized transaction samples
- Labels: Known gold loan customers (from marketing surveys + consented data)
- Accuracy: 84% precision, 78% recall
- False positive rate: 16% (acceptable for marketing use case)

**Output:** Propensity score (0-100) + recommended action

## Model Output Example

### Sample Prediction Card

```json
{
  "customer_id": "KOTAK789012",
  "propensity_score": 87,
  "confidence_level": "HIGH",

  "competitor_analysis": {
    "competitor_likely": true,
    "probable_lender": "Muthoot Finance",
    "estimated_outstanding": "Rs 3,50,000",
    "estimated_interest_rate": "13.5% p.a.",
    "monthly_payment": "Rs 4,200",
    "loan_tenure_months": 9,
    "renewal_likelihood": "62%"
  },

  "recommended_offer": {
    "interest_rate": "9.5% p.a.",
    "savings_message": "Save Rs 1,200/month on interest",
    "processing_fee": "Zero (waived)",
    "special_incentive": "Rs 3,000 cashback on transfer",
    "tenure": "12 months",
    "ltv": "75%"
  },

  "engagement_strategy": {
    "best_channel": "RM_call",
    "secondary_channel": "WhatsApp",
    "best_time": "Month-end (3 days before payment due)",
    "message_tone": "Savings-focused (not feature-focused)",
    "urgency_trigger": "Renewal approaching in 3 months"
  },

  "risk_assessment": {
    "credit_score": 742,
    "kotak_relationship_years": 6,
    "avg_monthly_balance": "Rs 45,000",
    "default_risk": "LOW"
  }
}
```

## Performance Lift: Untargeted vs AI-Targeted

| Metric | Untargeted Campaign | AI-Targeted Campaign | Lift |
|--------|-------------------|---------------------|------|
| **Campaign response rate** | 2% | 8% | **4x** |
| **Lead to conversion rate** | 15% | 35% | **2.3x** |
| **Overall conversion** | 0.12% | 1.96% | **16x** |
| **Customer acquisition cost** | Rs 2,000 | Rs 500 | **-75%** |
| **Average loan size** | Rs 1.8 lakh | Rs 3.2 lakh | **+78%** |

### ROI Calculation (Year 1)

**Investment:** Rs 2.5 Cr (model development + infrastructure)

**Campaign Scenario:**
- Target audience: 5 lakh high-propensity customers (score >70)
- Campaign response: 8% = 40,000 leads
- Lead to conversion: 35% = 14,000 disbursements
- Average loan: Rs 3.2 lakh
- Total disbursement: Rs 448 Cr
- Revenue (NIM 3.5%): Rs 15.7 Cr (Year 1)

**Compared to Untargeted:**
- Same budget, 5 lakh outreach → 600 conversions
- Total disbursement: Rs 108 Cr
- Revenue: Rs 3.8 Cr

**Incremental Revenue:** Rs 11.9 Cr
**ROI:** 476% in Year 1

## Privacy & Compliance

### DPDP Act 2023 Compliance

**What We DON'T Do:**
- ❌ Store raw transaction descriptions
- ❌ Share data with third parties
- ❌ Use data for purposes other than Kotak product offers
- ❌ Process data without legal basis

**What We DO:**
- ✅ Aggregate pattern analysis (not individual transaction reading)
- ✅ Customer consent for marketing (existing ToS + opt-out option)
- ✅ Anonymized model training data
- ✅ Right to opt-out of targeted marketing
- ✅ Data retention limits (propensity scores, not raw transactions)

**Legal Basis:**
- Legitimate interest: Offering better products to existing customers
- Contractual necessity: Banking relationship permits product recommendations
- Consent: ToS includes marketing analytics clause + opt-out mechanism

### Transparency

**Customer Communication:**
> "Based on your financial activity, we think you might benefit from Kotak's Gold Loan at 9.5% p.a. - lower than many competitors. This is a personalized offer. Click here to opt-out of such recommendations."

**Internal Governance:**
- Data access limited to authorized marketing team
- Model audit every 6 months (bias, accuracy, compliance)
- Privacy impact assessment (PIA) conducted
- RBI guidelines on customer data analytics followed

## Implementation Roadmap

### Phase 1: Pilot (Months 1-3) - Rs 1.5 Cr
- Build model on 6-month transaction history
- Test on 50K customers in Mumbai
- Manual review of high-propensity leads (score >80)
- Success criteria: 10%+ response rate, 25%+ conversion

### Phase 2: Scale (Months 4-8) - Rs 0.75 Cr
- Expand to 5 lakh customers (pan-India)
- Automate campaign triggering (no manual review)
- A/B test messaging variants
- Success criteria: 8%+ response, 1.5%+ overall conversion

### Phase 3: Optimize (Months 9-12) - Rs 0.25 Cr
- Real-time propensity scoring (daily model refresh)
- Integration with CRM and branch RM dashboards
- Lookalike modeling (find similar customers)
- Success criteria: 2%+ overall conversion, Rs 500 CAC

---

**Visual Design Notes:**
- Left: Data signal icons flowing into central AI brain
- Center: Model output JSON displayed as card/dashboard
- Right: Performance lift comparison (bar chart with arrows showing improvement)
- Bottom: Privacy compliance badge and DPDP Act 2023 logo
- Color coding: Blue for data inputs, Red for AI processing, Gold for outputs
