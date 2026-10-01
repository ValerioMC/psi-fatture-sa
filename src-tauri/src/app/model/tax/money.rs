/// Rounds an amount to the cent, half away from zero, as invoices store it.
pub fn round_cents(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
