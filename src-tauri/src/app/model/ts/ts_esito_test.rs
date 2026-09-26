use super::*;

#[test]
fn parses_esito_codes() {
    assert_eq!(TsEsito::parse("0"), Ok(TsEsito::Accepted));
    assert_eq!(TsEsito::parse("2"), Ok(TsEsito::AcceptedWithWarnings));
    assert_eq!(TsEsito::parse(" 1 "), Ok(TsEsito::Rejected));
    assert!(TsEsito::parse("9").is_err());
}
