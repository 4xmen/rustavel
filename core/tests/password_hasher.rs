#[cfg(test)]
mod tests {
    use rustavel_core::facades::password::PasswordDriver;
    use rustavel_core::facades::password::PasswordHasher;

    /// Creates a hasher for each supported password driver.
    fn drivers() -> Vec<(&'static str, PasswordHasher)> {
        vec![
            ("Argon2", PasswordHasher::new(PasswordDriver::Argon2)),
            (
                "bcrypt",
                PasswordHasher::new(PasswordDriver::Bcrypt { cost: 4 }),
            ),
        ]
    }

    #[test]
    fn hash_and_verify_password_for_each_driver() {
        // Verify that each driver hashes and validates passwords correctly.
        for (name, hasher) in drivers() {
            let password = "correct-horse-battery-staple";
            let hash = hasher.hash(password).expect("hashing should succeed");

            assert_ne!(hash, password, "{name}: hash must not expose plaintext");
            assert!(
                hasher.verify(password, &hash).expect("verification should succeed"),
                "{name}: correct password should be accepted"
            );
            assert!(
                !hasher
                    .verify("incorrect-password", &hash)
                    .expect("verification should succeed"),
                "{name}: incorrect password should be rejected"
            );
        }
    }

    #[test]
    fn hashing_same_password_produces_different_hashes() {
        // Random salts should make independently generated hashes different.
        for (name, hasher) in drivers() {
            let password = "same-password";

            let first = hasher.hash(password).expect("first hash should succeed");
            let second = hasher.hash(password).expect("second hash should succeed");

            assert_ne!(first, second, "{name}: hashes should use different salts");

            assert!(
                hasher.verify(password, &first).expect("verification should succeed"),
                "{name}: first hash should remain valid"
            );
            assert!(
                hasher.verify(password, &second).expect("verification should succeed"),
                "{name}: second hash should remain valid"
            );
        }
    }

    #[test]
    fn verify_rejects_malformed_hash() {
        // Invalid encoded hashes must return errors rather than panic.
        for (name, hasher) in drivers() {
            assert!(
                hasher.verify("some-password", "not-a-valid-hash").is_err(),
                "{name}: malformed hash should return an error"
            );
        }
    }
}
