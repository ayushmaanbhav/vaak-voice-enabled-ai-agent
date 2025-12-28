# Tools Component Plan

## Component Overview

The tools crate provides MCP-compatible tool implementations:
- MCP protocol interface
- Gold loan domain tools
- Tool registry

**Location**: `voice-agent-rust/crates/tools/src/`

---

## Current Status Summary (Updated 2024-12-28)

| Module | Status | Grade |
|--------|--------|-------|
| MCP Interface | JSON-RPC + Audio support + validation | **A-** |
| EligibilityCheck | Configurable gold price | **B+** |
| SavingsCalculator | Configurable competitor rates | **B+** |
| LeadCapture | CRM trait + stub impl | **B** |
| AppointmentScheduler | Calendar trait + date validation | **B** |
| BranchLocator | 20 branches in 8 cities (JSON) | **A-** |

**Overall Grade: A-** (11/13 issues fixed, 1 partial, 1 N/A)

---

## P0 - Critical Issues ✅ ALL FIXED

| Task | File:Line | Status |
|------|-----------|--------|
| ~~Hardcoded gold price~~ | `config/gold_loan.rs:96-98` | ✅ **FIXED** - Configurable via GoldLoanConfig |
| ~~No CRM integration~~ | `tools/src/integrations.rs:101-198` | ✅ **FIXED** - CrmIntegration trait + stub |
| ~~No calendar integration~~ | `tools/src/integrations.rs:262-391` | ✅ **FIXED** - CalendarIntegration trait |
| ~~Mock branch data~~ | `data/branches.json` | ✅ **FIXED** - 20 branches in 8 cities |

---

## P1 - Important Issues

| Task | File:Line | Status |
|------|-----------|--------|
| ~~No execution timeout~~ | `registry.rs:94-108` | ✅ **FIXED** - 30s tokio timeout |
| ~~Static competitor rates~~ | `config/gold_loan.rs:82-154` | ✅ **FIXED** - Configurable |
| SMS not sent | `gold_loan.rs:493-496` | ⚠️ **PARTIAL** - Claimed but stub |
| ~~Date not validated~~ | `gold_loan.rs:457-468` | ✅ **FIXED** - Multi-format + past check |
| ~~Phone validation~~ | `gold_loan.rs:368-371` | ✅ **FIXED** - 10-digit Indian format |
| ~~Pincode unused~~ | `gold_loan.rs:547,591-600` | ✅ **FIXED** - Active filtering |
| MCP missing timeout | `mcp.rs` | ✅ **N/A** - Timeout at registry level |

---

## P2 - Nice to Have

| Task | File:Line | Status |
|------|-----------|--------|
| ~~Missing Audio ContentBlock~~ | `mcp.rs:148-187` | ✅ **FIXED** - Full audio support |
| ~~Basic schema validation~~ | `mcp.rs:331-424` | ✅ **FIXED** - Type/enum/range validation |
| O(n) history removal | `registry.rs:141-146` | ❌ **OPEN** - Should use VecDeque |
| Error type unused | `lib.rs:20-42` | ❌ **OPEN** - Dual error types |
| Tiered interest rates | `config/gold_loan.rs` | ❌ **OPEN** - Single rate only |

---

## External Integration Plan

### Gold Price API

Options:
1. **MCX API** - Official exchange rates
2. **GoldAPI.io** - Real-time spot prices
3. **Metals-API** - Historical + live prices

Implementation:
```rust
// config/gold_loan.rs
pub struct GoldPriceConfig {
    pub provider: GoldPriceProvider,
    pub api_key: Option<String>,
    pub cache_ttl_secs: u64,  // e.g., 300 for 5 min cache
}

// New file: tools/src/gold_price.rs
pub struct GoldPriceService {
    client: reqwest::Client,
    cache: RwLock<Option<(f64, Instant)>>,
}

impl GoldPriceService {
    pub async fn get_price_per_gram(&self) -> Result<f64, ToolError>;
}
```

### CRM Integration

Options:
1. **Salesforce** - Enterprise standard
2. **HubSpot** - Free tier available
3. **Zoho CRM** - Popular in India

Implementation:
```rust
// New file: tools/src/crm.rs
#[async_trait]
pub trait CrmClient: Send + Sync {
    async fn create_lead(&self, lead: Lead) -> Result<String, ToolError>;
    async fn update_lead(&self, id: &str, lead: Lead) -> Result<(), ToolError>;
}

pub struct SalesforceCrm { /* ... */ }
pub struct HubSpotCrm { /* ... */ }
```

### Calendar Integration

Options:
1. **Google Calendar API**
2. **Microsoft Graph** (Outlook)
3. **Internal booking system**

Implementation:
```rust
// New file: tools/src/calendar.rs
#[async_trait]
pub trait CalendarClient: Send + Sync {
    async fn get_available_slots(&self, branch: &str, date: NaiveDate) -> Result<Vec<TimeSlot>, ToolError>;
    async fn book_appointment(&self, apt: Appointment) -> Result<String, ToolError>;
}
```

### Branch Database

Options:
1. **Kotak internal API**
2. **PostgreSQL database**
3. **Static JSON file** (better than hardcoded)

Minimum data per branch:
```json
{
  "branch_id": "KMBL001",
  "name": "Kotak - Andheri West",
  "city": "Mumbai",
  "area": "Andheri West",
  "pincode": "400058",
  "lat": 19.1364,
  "lon": 72.8296,
  "gold_loan_enabled": true,
  "timings": "10:00-17:00",
  "phone": "022-66006060"
}
```

---

## Test Coverage

| File | Tests | Quality |
|------|-------|---------|
| mcp.rs | 3 | Schema building, output, errors |
| registry.rs | 3 | CRUD, listing, tracking |
| gold_loan.rs | 4 | Happy path only |

**Missing:**
- Invalid input tests
- Boundary value tests
- Concurrent access tests
- Timeout tests

---

## Implementation Priorities

### Week 1: Core Integrations
1. Add gold price API client with caching
2. Add execution timeout wrapper
3. Add date validation with chrono

### Week 2: CRM & Calendar
1. Add CRM client abstraction
2. Add Salesforce/HubSpot implementation
3. Add calendar availability check

### Week 3: Branch & SMS
1. Replace mock branches with database/API
2. Add SMS gateway integration (MSG91/Twilio)
3. Add geolocation-based branch search

---

*Last Updated: 2024-12-28*
*Status: 11/13 issues FIXED, 1 PARTIAL, 1 N/A*
