
use argon2::{
    Argon2,
    password_hash::{
        PasswordHasher as _,
        PasswordVerifier as _,
        phc::PasswordHash,
    },
};

/// Selects the algorithm and work factor used for password hashing.
pub enum PasswordDriver {
    /// Argon2id with its default parameters.
    Argon2,

    /// bcrypt with the specified logarithmic work factor.
    Bcrypt { cost: u32 },
}

/// Hashes and verifies passwords using the selected algorithm.
pub struct PasswordHasher {
    driver: PasswordDriver,
}

impl PasswordHasher {
    /// Creates a password hasher using the selected driver.
    pub fn new(driver: PasswordDriver) -> Self {
        Self { driver }
    }

    /// Hashes a password and returns its encoded hash.
    ///
    /// The selected algorithm generates a random salt and stores the
    /// algorithm-specific parameters in the resulting hash string.
    pub fn hash(&self, password: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Dispatch hashing to the selected driver.
        match self.driver {
            PasswordDriver::Argon2 => {
                Ok(Argon2::default()
                    .hash_password(password.as_bytes())?
                    .to_string())
            }
            PasswordDriver::Bcrypt { cost } => {
                Ok(bcrypt::non_truncating_hash(password, cost)?)
            }
        }
    }

    /// Verifies a password against a hash produced by the selected driver.
    ///
    /// Returns false when the password does not match.
    pub fn verify(
        &self,
        password: &str,
        hash: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Dispatch verification to the selected driver.
        match self.driver {
            PasswordDriver::Argon2 => {
                let parsed = PasswordHash::new(hash)?;

                match Argon2::default().verify_password(password.as_bytes(), &parsed) {
                    Ok(()) => Ok(true),
                    Err(argon2::password_hash::Error::PasswordInvalid) => Ok(false),
                    Err(error) => Err(Box::new(error)),
                }
            }
            PasswordDriver::Bcrypt { .. } => {
                Ok(bcrypt::non_truncating_verify(password, hash)?)
            }
        }
    }
}
