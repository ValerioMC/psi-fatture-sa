use super::*;

fn total(month: i64, revenue: f64, invoice_count: i64) -> MonthTotal {
    MonthTotal {
        month,
        revenue,
        invoice_count,
    }
}

#[test]
fn monthly_revenue_fills_every_month_in_order() {
    let months = monthly_revenue(&[total(3, 160.0, 2), total(11, 80.0, 1)]);
    assert_eq!(months.len(), 12);
    assert_eq!(months[0].month_name, "Gennaio");
    assert_eq!(months[0].revenue, 0.0);
    assert_eq!(months[2].revenue, 160.0);
    assert_eq!(months[2].invoice_count, 2);
    assert_eq!(months[10].month_name, "Novembre");
    assert_eq!(months[10].invoice_count, 1);
}

#[test]
fn monthly_revenue_without_totals_is_all_zero() {
    let months = monthly_revenue(&[]);
    assert!(months
        .iter()
        .all(|m| m.revenue == 0.0 && m.invoice_count == 0));
}
