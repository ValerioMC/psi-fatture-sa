use super::*;

#[test]
fn encrypts_to_one_rsa_block_of_the_1024_bit_key() {
    let cipher = SanitelCipher::from_bundled_certificate().unwrap();
    let encoded = cipher.encrypt("3489543096").unwrap();
    assert_eq!(STANDARD.decode(encoded).unwrap().len(), 128);
}

#[test]
fn padding_is_randomised() {
    let cipher = SanitelCipher::from_bundled_certificate().unwrap();
    assert_ne!(
        cipher.encrypt("MTOMRA66A41G224M").unwrap(),
        cipher.encrypt("MTOMRA66A41G224M").unwrap()
    );
}
