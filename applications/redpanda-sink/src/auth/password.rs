use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(password: &str) -> Result<String, String> {
    hash(password, DEFAULT_COST).map_err(|e| format!("Failed to hash password: {}", e))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    verify(password, hash).map_err(|e| format!("Failed to verify password: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "test-password-123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }

    #[test]
    fn test_hash_different_passwords() {
        let password1 = "password1";
        let password2 = "password2";

        let hash1 = hash_password(password1).unwrap();
        let hash2 = hash_password(password2).unwrap();

        // Hashes should be different
        assert_ne!(hash1, hash2);

        // Each should verify with its own password
        assert!(verify_password(password1, &hash1).unwrap());
        assert!(verify_password(password2, &hash2).unwrap());

        // But not with the other password
        assert!(!verify_password(password1, &hash2).unwrap());
        assert!(!verify_password(password2, &hash1).unwrap());
    }

    #[test]
    fn test_hash_empty_password() {
        let password = "";
        let hash = hash_password(password).unwrap();

        // Empty password should hash successfully
        assert!(!hash.is_empty());
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("not-empty", &hash).unwrap());
    }

    #[test]
    fn test_hash_special_characters() {
        let passwords = vec![
            "!@#$%^&*()",
            "password with spaces",
            "pässwörd",
            "密码123",
            "very-long-password-with-many-characters-that-should-still-work-correctly",
        ];

        for password in passwords {
            let hash = hash_password(password).unwrap();
            assert!(verify_password(password, &hash).unwrap());
            assert!(!verify_password("wrong", &hash).unwrap());
        }
    }

    #[test]
    fn test_verify_invalid_hash() {
        let password = "test-password";

        // Test with invalid hash format
        assert!(verify_password(password, "not-a-valid-hash").is_err());
        assert!(verify_password(password, "").is_err());
        assert!(verify_password(password, "$2b$12$invalid").is_err());
    }

    #[test]
    fn test_hash_same_password_different_hashes() {
        let password = "same-password";

        // Hashing the same password multiple times should produce different hashes
        // (due to salt), but all should verify correctly
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();
        let hash3 = hash_password(password).unwrap();

        // Hashes should be different (due to random salt)
        assert_ne!(hash1, hash2);
        assert_ne!(hash2, hash3);
        assert_ne!(hash1, hash3);

        // But all should verify with the same password
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
        assert!(verify_password(password, &hash3).unwrap());
    }

    #[test]
    fn test_hash_case_sensitive() {
        let password1 = "Password123";
        let password2 = "password123";

        let hash1 = hash_password(password1).unwrap();
        let hash2 = hash_password(password2).unwrap();

        // Case-sensitive passwords should produce different hashes
        assert_ne!(hash1, hash2);

        // Each should only verify with its exact password
        assert!(verify_password(password1, &hash1).unwrap());
        assert!(verify_password(password2, &hash2).unwrap());
        assert!(!verify_password(password1, &hash2).unwrap());
        assert!(!verify_password(password2, &hash1).unwrap());
    }

    #[test]
    fn test_hash_unicode_passwords() {
        let passwords = vec!["café", "🚀password", "пароль", "パスワード"];

        for password in passwords {
            let hash = hash_password(password).unwrap();
            assert!(verify_password(password, &hash).unwrap());
        }
    }
}
