import { Deck } from 'spectacle';
import { kotakTheme } from './theme/kotakTheme';

// Section A: Opening
import {
  TitleSlide,
  CustomerStorySlide,
  ThreeQuestionsSlide,
  AnswerPreviewSlide,
} from './components/slides/section-a-opening';

// Section B: Market Opportunity
import {
  MarketSizeSlide,
  GrowthDriversSlide,
  RateGapSlide,
  SavingsCalculatorSlide,
  WhyNowSlide,
} from './components/slides/section-b-market';

// Section C: NBFC Issues
import {
  CustomerPainPointsSlide,
  IndustryTrustCrisisSlide,
  ManappuramIssuesSlide,
  MuthootIIFLIssuesSlide,
} from './components/slides/section-c-nbfc-issues';

// Section D: Switching Psychology
import {
  SwitchingBarriersSlide,
  SwitchingJourneySlide,
  TrustMessagingSlide,
} from './components/slides/section-d-psychology';

// Section E: Customer Segments
import {
  SegmentOverviewSlide,
  P1HighValueSlide,
  P2TrustSeekersSlide,
  P3P4OverviewSlide,
  TargetingMatrixSlide,
} from './components/slides/section-e-segments';

// Section F: Product Solution
import {
  SwitchSaveProductSlide,
  BridgeLoanSlide,
  SwitchAssistSlide,
  IncentiveStructureSlide,
  CampaignConceptsSlide,
} from './components/slides/section-f-product';

// Section G: AI & Technology
import {
  AIStrategyOverviewSlide,
  ComputerVisionSlide,
  PredictiveModelSlide,
  PersonalizedPitchingSlide,
  TechInvestmentSlide,
} from './components/slides/section-g-ai';

// Section H: Doorstep Pilot
import {
  DoorstepOpportunitySlide,
  DoorstepEconomicsSlide,
  DoorstepPilotPlanSlide,
} from './components/slides/section-h-doorstep';

// Section I: Financials
import {
  ThreeYearSummarySlide,
  RevenueProfitabilitySlide,
  ROIAnalysisSlide,
  InvestmentContextSlide,
  RiskGuardrailsSlide,
} from './components/slides/section-i-financials';

// Section J: Closing
import {
  StrategicImperativeSlide,
  TheAskSlide,
  ThankYouSlide,
} from './components/slides/section-j-closing';

function App() {
  return (
    <Deck theme={kotakTheme}>
      {/* Section A: Opening (4 slides) */}
      <TitleSlide />
      <CustomerStorySlide />
      <ThreeQuestionsSlide />
      <AnswerPreviewSlide />

      {/* Section B: Market Opportunity (5 slides) */}
      <MarketSizeSlide />
      <GrowthDriversSlide />
      <RateGapSlide />
      <SavingsCalculatorSlide />
      <WhyNowSlide />

      {/* Section C: NBFC Issues (4 slides) */}
      <CustomerPainPointsSlide />
      <IndustryTrustCrisisSlide />
      <ManappuramIssuesSlide />
      <MuthootIIFLIssuesSlide />

      {/* Section D: Switching Psychology (3 slides) */}
      <SwitchingBarriersSlide />
      <SwitchingJourneySlide />
      <TrustMessagingSlide />

      {/* Section E: Customer Segments (5 slides) */}
      <SegmentOverviewSlide />
      <P1HighValueSlide />
      <P2TrustSeekersSlide />
      <P3P4OverviewSlide />
      <TargetingMatrixSlide />

      {/* Section F: Product Solution (5 slides) */}
      <SwitchSaveProductSlide />
      <BridgeLoanSlide />
      <SwitchAssistSlide />
      <IncentiveStructureSlide />
      <CampaignConceptsSlide />

      {/* Section G: AI & Technology (5 slides) */}
      <AIStrategyOverviewSlide />
      <ComputerVisionSlide />
      <PredictiveModelSlide />
      <PersonalizedPitchingSlide />
      <TechInvestmentSlide />

      {/* Section H: Doorstep Pilot (3 slides) */}
      <DoorstepOpportunitySlide />
      <DoorstepEconomicsSlide />
      <DoorstepPilotPlanSlide />

      {/* Section I: Financials (5 slides) */}
      <ThreeYearSummarySlide />
      <RevenueProfitabilitySlide />
      <ROIAnalysisSlide />
      <InvestmentContextSlide />
      <RiskGuardrailsSlide />

      {/* Section J: Closing (3 slides) */}
      <StrategicImperativeSlide />
      <TheAskSlide />
      <ThankYouSlide />
    </Deck>
  );
}

export default App;
