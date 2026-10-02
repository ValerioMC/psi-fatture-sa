use super::*;

#[test]
fn parses_esito_codes() {
    assert_eq!(TsEsito::parse("0").ok(), Some(TsEsito::Accepted));
    assert_eq!(
        TsEsito::parse("2").ok(),
        Some(TsEsito::AcceptedWithWarnings)
    );
    assert_eq!(TsEsito::parse(" 1 ").ok(), Some(TsEsito::Rejected));
    assert!(TsEsito::parse("9").is_err());
}
