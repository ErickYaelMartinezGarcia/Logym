    use super::*;
    #[test]
    fn persistent_faults_recreate_after_three_attempts_and_success_resets_counter() {
        let mut failures = FallosConsecutivos::default();
        assert!(!failures.registrar(true));
        assert!(!failures.registrar(true));
        assert!(failures.registrar(true));
        assert!(!failures.registrar(true));
        assert!(!failures.registrar(false));
        assert!(!failures.registrar(true));
        assert!(!failures.registrar(true));
        assert!(failures.registrar(true));
    }
