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
}


