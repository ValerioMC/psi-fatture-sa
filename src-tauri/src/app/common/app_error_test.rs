use super::*;

#[test]
fn serializes_as_its_message() {
    let error = AppError::Conflict("Numero già utilizzato".to_string());

    assert_eq!(
        serde_json::to_string(&error).unwrap(),
        "\"Numero già utilizzato\""
    );
}

#[test]
fn database_errors_keep_the_driver_message() {
    let error = AppError::from(DbErr::Custom("disco pieno".to_string()));

    assert_eq!(
        error.to_string(),
        "Errore del database: Custom Error: disco pieno"
    );
}
