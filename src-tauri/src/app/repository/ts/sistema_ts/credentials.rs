/// The login and PINCODE of the professional, in clear; encrypted when written.
pub struct Credentials<'a> {
    pub username: &'a str,
    pub pincode: &'a str,
}
