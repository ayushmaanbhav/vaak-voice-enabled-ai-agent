---
slide_number: 36
title: "Personalized Pitch Architecture"
section: "Personalized Pitch"
colors:
  primary: "#ED1C24"
  secondary: "#002B5C"
  accent: "#C9A227"
layout: "framework_flow"
visual_elements:
  - "Data flow infographic"
  - "Personalization engine diagram"
  - "Input-output architecture"
---

# Personalized Pitch Architecture

## The Personalization Framework

**Customer Data → Propensity Score → Segment Assignment → Personalized Offer → Optimized Channel → Timed Outreach**

---

## Data Sources Powering Personalization

### Internal Data (Kotak Systems)
- **KYC Records**: Age, gender, occupation, address, income bracket
- **Transaction Patterns**: Payments to gold lenders detected via IMPS/NEFT
- **Product Holdings**: Existing Kotak relationship depth (savings, current, credit card, demat)
- **Digital Footprint**: App usage, calculator interactions, loan inquiries
- **Life Events**: Wedding, medical, education payments flagged in transaction history

### External Data (Bureau & Third-Party)
- **Credit Bureau**: CIBIL score, existing gold loan exposure, risk-based pricing tier
- **Seasonal Triggers**: Festivals (Diwali, Onam, Pongal), harvest cycles (Rabi/Kharif)
- **Geographic Patterns**: City-specific competitor concentration, branch proximity

---

## Personalization Engine Output

### The Winning Formula
**Right Rate + Right Message + Right Channel + Right Timing = Maximum Conversion**

| Component | Personalization Logic | Example |
|-----------|----------------------|---------|
| **Right Rate** | Credit score + LTV + relationship depth | 8.5% for Kotak Premier customer with 750+ CIBIL |
| **Right Message** | Segment-specific pain point + value proposition | "Safety meets savings" for Trust-Seekers |
| **Right Channel** | Digital adoption + age + ticket size | RM outreach for 50L+ loan, App push for <35 age |
| **Right Timing** | Payment cycle + festival calendar + life events | 15 days before EMI due date to competitor |

---

## Architecture Visualization

```
┌─────────────────────────────────────────────────────────────────┐
│                     DATA INGESTION LAYER                         │
├─────────────────────────────────────────────────────────────────┤
│  KYC     Transaction    Product     Bureau     Life     Seasonal │
│  Data    Patterns       Holdings    Data       Events   Triggers │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                   PROPENSITY SCORING ENGINE                      │
├─────────────────────────────────────────────────────────────────┤
│  ML Model: Gold Loan Switch Probability (0-100 score)           │
│  Inputs: Payment to competitor + loan inquiry + credit need     │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SEGMENT ASSIGNMENT                            │
├─────────────────────────────────────────────────────────────────┤
│  P1: High-Value Rational  │  P3: Women Entrepreneurs             │
│  P2: Trust-Seekers        │  P4: Young Professionals             │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    OFFER GENERATION                              │
├─────────────────────────────────────────────────────────────────┤
│  Rate + Fee Waiver + Cashback + Special Feature                 │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    CHANNEL OPTIMIZATION                          │
├─────────────────────────────────────────────────────────────────┤
│  RM / Branch / App / SMS / Email / BC Network                   │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    TIMING ORCHESTRATION                          │
├─────────────────────────────────────────────────────────────────┤
│  Send Date: 15 days before competitor EMI + Festival window     │
└─────────────────────────────────────────────────────────────────┘
```

---

## Key Success Metrics

- **Propensity Model Accuracy**: 75%+ precision in identifying switchers
- **Segment Conversion Lift**: 2-3x higher than generic campaigns
- **Channel Efficiency**: 30% lower CAC via optimized routing
- **Timing Impact**: 40% higher open rates for timed campaigns

---

## Implementation Readiness

**Phase 1 (Month 1-3)**: Rule-based segmentation using KYC + transaction data
**Phase 2 (Month 4-6)**: ML propensity model integrated with CRM
**Phase 3 (Month 7-12)**: Real-time personalization engine with A/B testing
