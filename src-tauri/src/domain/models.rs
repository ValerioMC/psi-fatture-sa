/// Domain models for PSI Fatture SA.
///
/// All structs represent domain entities as Value Objects or Entities.
/// Serde is used for serialization to/from the Tauri command layer.
use serde::{Deserialize, Serialize};

// ─── Professional Config ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfessionalConfig {
    pub id: i64,
    pub title: String,
    pub first_name: String,
    pub last_name: String,
    pub vat_number: String,
    pub fiscal_code: String,
    pub tax_regime: TaxRegime,
    pub albo_number: String,
    pub albo_region: String,
    pub address: String,
    pub city: String,
    pub province: String,
    pub zip_code: String,
    pub country: String,
    pub phone: String,
    pub pec_email: String,
    pub iban: String,
    pub coefficient: f64,
    pub is_psicoanalista: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertConfigInput {
    pub title: String,
    pub first_name: String,
    pub last_name: String,
    pub vat_number: String,
    pub fiscal_code: String,
    pub tax_regime: TaxRegime,
    pub albo_number: String,
    pub albo_region: String,
    pub address: String,
    pub city: String,
    pub province: String,
    pub zip_code: String,
    pub country: String,
    pub phone: String,
    pub pec_email: String,
    pub iban: String,
    pub coefficient: f64,
    pub is_psicoanalista: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaxRegime {
    Forfettario,
    Ordinario,
}

impl TaxRegime {
    pub fn as_str(&self) -> &str {
        match self {
            TaxRegime::Forfettario => "forfettario",
            TaxRegime::Ordinario => "ordinario",
        }
    }
}

impl From<String> for TaxRegime {
    fn from(s: String) -> Self {
        match s.as_str() {
            "ordinario" => TaxRegime::Ordinario,
            _ => TaxRegime::Forfettario,
        }
    }
}

// ─── Client ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: i64,
    pub client_type: ClientType,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub fiscal_code: String,
    pub vat_number: Option<String>,
    pub address: String,
    pub city: String,
    pub province: String,
    pub zip_code: String,
    pub email: Option<String>,
    pub phone: String,
    pub notes: Option<String>,
    pub sts_authorization: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientInput {
    pub client_type: ClientType,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub fiscal_code: String,
    pub vat_number: Option<String>,
    pub address: String,
    pub city: String,
    pub province: String,
    pub zip_code: String,
    pub email: Option<String>,
    pub phone: String,
    pub notes: Option<String>,
    pub sts_authorization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClientInput {
    pub id: i64,
    pub client_type: ClientType,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub fiscal_code: String,
    pub vat_number: Option<String>,
    pub address: String,
    pub city: String,
    pub province: String,
    pub zip_code: String,
    pub email: Option<String>,
    pub phone: String,
    pub notes: Option<String>,
    pub sts_authorization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ClientType {
    PersonaFisica,
    Azienda,
}

impl From<String> for ClientType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "azienda" => ClientType::Azienda,
            _ => ClientType::PersonaFisica,
        }
    }
}

impl ClientType {
    pub fn as_str(&self) -> &str {
        match self {
            ClientType::PersonaFisica => "persona_fisica",
            ClientType::Azienda => "azienda",
        }
    }
}

// ─── Service ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub default_price: f64,
    pub vat_rate: f64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceInput {
    pub name: String,
    pub description: String,
    pub default_price: f64,
    pub vat_rate: f64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServiceInput {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub default_price: f64,
    pub vat_rate: f64,
    pub is_active: bool,
}

// ─── Invoice ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: i64,
    pub client_id: i64,
    pub client_name: String,
    pub invoice_number: String,
    pub year: i64,
    pub issue_date: String,
    pub due_date: Option<String>,
    pub status: InvoiceStatus,
    pub payment_method: PaymentMethod,
    pub notes: String,
    pub apply_enpap: bool,
    pub contributo_enpap: f64,
    pub ritenuta_acconto: f64,
    pub marca_da_bollo: bool,
    pub total_net: f64,
    pub total_tax: f64,
    pub total_gross: f64,
    pub total_due: f64,
    pub paid_date: Option<String>,
    pub lines: Vec<InvoiceLine>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLine {
    pub id: Option<i64>,
    pub invoice_id: Option<i64>,
    pub service_id: Option<i64>,
    pub description: String,
    pub quantity: i64,
    pub unit_price: f64,
    pub vat_rate: f64,
    pub line_total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceInput {
    pub client_id: i64,
    pub issue_date: String,
    pub due_date: Option<String>,
    pub status: InvoiceStatus,
    pub payment_method: PaymentMethod,
    pub notes: String,
    pub apply_enpap: bool,
    pub lines: Vec<InvoiceLineInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInvoiceInput {
    pub id: i64,
    pub client_id: i64,
    pub issue_date: String,
    pub due_date: Option<String>,
    pub status: InvoiceStatus,
    pub payment_method: PaymentMethod,
    pub notes: String,
    pub apply_enpap: bool,
    pub paid_date: Option<String>,
    pub lines: Vec<InvoiceLineInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineInput {
    pub service_id: Option<i64>,
    pub description: String,
    pub quantity: i64,
    pub unit_price: f64,
    pub vat_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InvoiceStatus {
    Draft,
    Issued,
    Paid,
    Overdue,
    Cancelled,
}

impl InvoiceStatus {
    pub fn as_str(&self) -> &str {
        match self {
            InvoiceStatus::Draft => "draft",
            InvoiceStatus::Issued => "issued",
            InvoiceStatus::Paid => "paid",
            InvoiceStatus::Overdue => "overdue",
            InvoiceStatus::Cancelled => "cancelled",
        }
    }
}

impl From<String> for InvoiceStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "issued" => InvoiceStatus::Issued,
            "paid" => InvoiceStatus::Paid,
            "overdue" => InvoiceStatus::Overdue,
            "cancelled" => InvoiceStatus::Cancelled,
            _ => InvoiceStatus::Draft,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PaymentMethod {
    Bonifico,
    Contanti,
    Pos,
    Altro,
}

impl PaymentMethod {
    pub fn as_str(&self) -> &str {
        match self {
            PaymentMethod::Bonifico => "bonifico",
            PaymentMethod::Contanti => "contanti",
            PaymentMethod::Pos => "pos",
            PaymentMethod::Altro => "altro",
        }
    }
}

impl From<String> for PaymentMethod {
    fn from(s: String) -> Self {
        match s.as_str() {
            "contanti" => PaymentMethod::Contanti,
            "pos" => PaymentMethod::Pos,
            "altro" => PaymentMethod::Altro,
            _ => PaymentMethod::Bonifico,
        }
    }
}

// ─── Appointment ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appointment {
    pub id: i64,
    pub client_id: i64,
    pub client_name: String,
    pub service_id: Option<i64>,
    pub service_name: Option<String>,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
    pub status: AppointmentStatus,
    pub notes: String,
    pub recurrence_group_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAppointmentInput {
    pub client_id: i64,
    pub service_id: Option<i64>,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
    pub status: AppointmentStatus,
    pub notes: String,
    pub recurrence_group_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAppointmentInput {
    pub id: i64,
    pub client_id: i64,
    pub service_id: Option<i64>,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
    pub status: AppointmentStatus,
    pub notes: String,
    pub recurrence_group_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecurringAppointmentsInput {
    pub client_id: i64,
    pub service_id: Option<i64>,
    pub dates: Vec<String>,
    pub start_time: String,
    pub end_time: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AppointmentStatus {
    Scheduled,
    Completed,
    Cancelled,
}

impl AppointmentStatus {
    pub fn as_str(&self) -> &str {
        match self {
            AppointmentStatus::Scheduled => "scheduled",
            AppointmentStatus::Completed => "completed",
            AppointmentStatus::Cancelled => "cancelled",
        }
    }
}

impl From<String> for AppointmentStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "completed" => AppointmentStatus::Completed,
            "cancelled" => AppointmentStatus::Cancelled,
            _ => AppointmentStatus::Scheduled,
        }
    }
}

// ─── Dashboard ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub year: i64,
    pub total_revenue: f64,
    pub paid_revenue: f64,
    pub unpaid_revenue: f64,
    pub total_invoices: i64,
    pub paid_invoices: i64,
    pub draft_invoices: i64,
    pub monthly_revenue: Vec<MonthlyRevenue>,
    pub recent_invoices: Vec<Invoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyRevenue {
    pub month: i64,
    pub month_name: String,
    pub revenue: f64,
    pub invoice_count: i64,
}

// ─── Filters ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceFilters {
    pub year: Option<i64>,
    pub status: Option<String>,
    pub client_id: Option<i64>,
    pub search: Option<String>,
}
