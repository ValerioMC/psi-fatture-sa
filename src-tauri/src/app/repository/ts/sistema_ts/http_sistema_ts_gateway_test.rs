use super::*;

const FAULT: &str = "<env:Envelope xmlns:env=\"http://schemas.xmlsoap.org/soap/envelope/\"><env:Body><env:Fault><faultcode>env:Client</faultcode><faultstring>{}</faultstring></env:Fault></env:Body></env:Envelope>";

#[test]
fn classifies_authentication_and_other_faults() {
    let auth = FAULT.replace("{}", "Errore generico di autenticazione");
    assert_eq!(
        classify_http(500, auth),
        Err(TsGatewayError::Authentication)
    );

    let internal = FAULT.replace("{}", "Internal Error");
    assert_eq!(
        classify_http(500, internal),
        Err(TsGatewayError::Fault("Internal Error".to_string()))
    );
}

#[test]
fn passes_ok_bodies_and_maps_bare_statuses() {
    assert_eq!(
        classify_http(200, "<ok/>".to_string()),
        Ok("<ok/>".to_string())
    );
    assert_eq!(
        classify_http(401, String::new()),
        Err(TsGatewayError::Authentication)
    );
    assert_eq!(
        classify_http(503, String::new()),
        Err(TsGatewayError::Http(503))
    );
}

#[test]
fn builds_with_the_bundled_certificates() {
    assert!(HttpSistemaTsGateway::new().is_ok());
}
