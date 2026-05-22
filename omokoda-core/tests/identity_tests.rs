#[cfg(test)]
mod identity_tests {
    use omokoda_core::identity::dna::generate_dna_fingerprint;

    #[test]
    fn dna_fingerprint_is_86_chars() {
        let dna = generate_dna_fingerprint("luna", 1714348800, &[0u8; 32]);
        assert_eq!(dna.len(), 86);
    }

    #[test]
    fn dna_fingerprint_is_deterministic() {
        let a = generate_dna_fingerprint("luna", 1714348800, &[0u8; 32]);
        let b = generate_dna_fingerprint("luna", 1714348800, &[0u8; 32]);
        assert_eq!(a, b);
    }

    #[test]
    fn dna_fingerprint_differs_by_name() {
        let a = generate_dna_fingerprint("luna", 1714348800, &[0u8; 32]);
        let b = generate_dna_fingerprint("sol", 1714348800, &[0u8; 32]);
        assert_ne!(a, b);
    }

    #[test]
    fn dna_fingerprint_differs_by_timestamp() {
        let a = generate_dna_fingerprint("luna", 1714348800, &[0u8; 32]);
        let b = generate_dna_fingerprint("luna", 1714348801, &[0u8; 32]);
        assert_ne!(a, b);
    }

    #[test]
    fn dna_fingerprint_is_ascii_printable() {
        let dna = generate_dna_fingerprint("luna", 1714348800, &[0u8; 32]);
        assert!(dna.chars().all(|c| c.is_ascii() && !c.is_ascii_control()));
    }

    #[test]
    fn bipon39_derivation_from_entropy() {
        use omokoda_core::identity::bipon39::Bipon39;
        let entropy = [0u8; 32];
        let mnemonic = Bipon39::entropy_to_mnemonic(&entropy);
        assert!(!mnemonic.is_empty());
        let indices = Bipon39::mnemonic_to_indices(&mnemonic).unwrap();
        assert!(!indices.is_empty());
    }
}
