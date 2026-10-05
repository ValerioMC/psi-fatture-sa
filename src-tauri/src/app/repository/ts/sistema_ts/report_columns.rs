/// Where each field of `report.csv` sits, found by its header name. Expense
/// totals come as one `TOTALE xx` / `TOTALE xx RIMBORSATO` pair per expense type.
pub(super) struct ReportColumns {
    pub(super) vat_number: usize,
    pub(super) issue_date: usize,
    pub(super) document_number: usize,
    pub(super) payment_date: usize,
    pub(super) protocol: usize,
    pub(super) sent_date: usize,
    pub(super) send_kind: usize,
    pub(super) totals: Vec<usize>,
    pub(super) refunds: Vec<usize>,
}

impl ReportColumns {
    pub(super) fn from_header(header_line: &str) -> Result<Self, String> {
        let header: Vec<String> = header_line
            .split(';')
            .map(|c| c.trim().to_uppercase())
            .collect();
        let required = |name: &str| {
            header
                .iter()
                .position(|h| h == name)
                .ok_or(format!("Colonna {name} assente nel report"))
        };
        Ok(Self {
            vat_number: required("PARTITA IVA")?,
            issue_date: required("DATA EMISSIONE")?,
            document_number: required("DOCUMENTO FISCALE NUMERO")?,
            payment_date: required("DATA PAGAMENTO")?,
            protocol: required("PROTOCOLLO")?,
            sent_date: required("DATA INVIO")?,
            send_kind: required("TIPO INVIO")?,
            totals: total_columns(&header, false),
            refunds: total_columns(&header, true),
        })
    }
}

fn total_columns(header: &[String], refunded: bool) -> Vec<usize> {
    header
        .iter()
        .enumerate()
        .filter(|(_, h)| h.starts_with("TOTALE ") && h.ends_with("RIMBORSATO") == refunded)
        .map(|(i, _)| i)
        .collect()
}
