//! Gold Loan Utility Functions
//!
//! Financial calculation utilities for gold loan tools.

/// P0 FIX: Calculate EMI using the standard amortization formula
///
/// EMI = P × r × (1 + r)^n / [(1 + r)^n - 1]
///
/// Where:
/// - P = Principal loan amount
/// - r = Monthly interest rate (annual_rate / 12 / 100)
/// - n = Number of months (tenure)
///
/// Note: Gold loans often use simple interest schemes where only interest
/// is paid monthly, but this function provides true EMI for accurate comparison.
pub fn calculate_emi(principal: f64, annual_rate_percent: f64, tenure_months: i64) -> f64 {
    if tenure_months <= 0 || principal <= 0.0 {
        return 0.0;
    }

    let monthly_rate = annual_rate_percent / 100.0 / 12.0;

    // Handle edge case of 0% interest
    if monthly_rate <= 0.0 {
        return principal / tenure_months as f64;
    }

    let n = tenure_months as f64;
    let one_plus_r_n = (1.0 + monthly_rate).powf(n);

    // EMI formula: P * r * (1+r)^n / [(1+r)^n - 1]
    principal * monthly_rate * one_plus_r_n / (one_plus_r_n - 1.0)
}

/// Calculate total interest paid over the loan tenure
pub fn calculate_total_interest(
    principal: f64,
    annual_rate_percent: f64,
    tenure_months: i64,
) -> f64 {
    let emi = calculate_emi(principal, annual_rate_percent, tenure_months);
    let total_paid = emi * tenure_months as f64;
    total_paid - principal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_emi() {
        // 1 lakh at 12% for 12 months
        let emi = calculate_emi(100_000.0, 12.0, 12);
        // Expected EMI around 8884.87
        assert!((emi - 8884.87).abs() < 1.0);
    }

    #[test]
    fn test_calculate_emi_zero_principal() {
        assert_eq!(calculate_emi(0.0, 12.0, 12), 0.0);
    }

    #[test]
    fn test_calculate_emi_zero_tenure() {
        assert_eq!(calculate_emi(100_000.0, 12.0, 0), 0.0);
    }

    #[test]
    fn test_calculate_emi_zero_rate() {
        // 1 lakh at 0% for 12 months = 8333.33 per month
        let emi = calculate_emi(100_000.0, 0.0, 12);
        assert!((emi - 8333.33).abs() < 1.0);
    }
}
