//! The Seven Planes of Nature — sim-to-real verification fidelity, as an
//! axis independent from Tier (fractal.rs).
//!
//! Tier answers "how much power/governance does this agent hold." Plane
//! answers a different question: "how proven is THIS specific claim or
//! action, on the ladder from cheap/reversible simulation up to
//! permanently-settled reality?" A single agent can submit actions at many
//! different planes over time; Plane is a property of an action/receipt,
//! not a fixed property of the agent.
//!
//! Same 7 names and the same order as the locked Orisha<->Tier table
//! (docs/256---65536.md's "7 Ascension Domains") -- Physical is the
//! densest/most concrete plane (an unverified claim, pure potential) and
//! Logoic is the most abstract/settled (an outcome anchored immutably on
//! chain). Reusing the same order keeps one mental model instead of two
//! competing "which one is tier 1" schemes.
//!
//! Honesty over completeness: only Physical (trivial base case), Buddhic
//! (real -- wired to the same ethics/Gender scoring that already runs live)
//! and Logoic (real -- checks for an actual on-chain receipt hash) have a
//! real verification check right now. Astral (VeilSim), Mental (MuJoCo
//! physics-accurate sim), Atmic (hardware-in-the-loop), and Monadic
//! (multi-agent consensus) all have real *systems* elsewhere in this
//! ecosystem (VeilSim, the proven cross-arch MuJoCo determinism result,
//! ScarabSwarm) but no live Rust-callable hook into this crate yet --
//! `verify` for those returns `Unverified::NotYetWired` rather than a fake
//! pass. Wiring each one is separate, real integration work.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Plane {
    /// An unverified claim -- pure potential, no proof of any kind yet.
    /// Trivially the starting plane for every new action.
    Physical = 0,
    /// Cheap, reversible, low-fidelity simulation (VeilSim). Fast iteration,
    /// no determinism guarantee.
    Astral = 1,
    /// Physics-accurate simulation with a proven determinism guarantee
    /// (MuJoCo, fixed timestep + RK4, cross-arch hash-identical). The real
    /// "does this actually work in physics" proof.
    Mental = 2,
    /// Ethical/consent review -- does this action honor the agent's own
    /// Gender-principle (creative/receptive balance) and Justice hooks?
    Buddhic = 3,
    /// Hardware-in-the-loop: real actuators, controlled environment,
    /// supervised execution (e.g. a real drone flight under human RPIC
    /// oversight per FAA Part 107).
    Atmic = 4,
    /// Multi-agent consensus before wider rollout -- do independent agents
    /// agree this outcome is sound?
    Monadic = 5,
    /// Settled: the outcome is anchored immutably (on-chain, e.g.
    /// Ṣàngó/Move) and can no longer be revised.
    Logoic = 6,
}

impl Plane {
    pub const ALL: [Plane; 7] = [
        Plane::Physical,
        Plane::Astral,
        Plane::Mental,
        Plane::Buddhic,
        Plane::Atmic,
        Plane::Monadic,
        Plane::Logoic,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Plane::Physical => "Physical",
            Plane::Astral => "Astral",
            Plane::Mental => "Mental",
            Plane::Buddhic => "Buddhic",
            Plane::Atmic => "Atmic",
            Plane::Monadic => "Monadic",
            Plane::Logoic => "Logoic",
        }
    }

    /// The Tier (1-7) that shares this plane in the locked Orisha table.
    /// Purely informational -- Plane does not require an agent to *be* that
    /// Tier to submit an action claiming this plane of verification.
    pub fn corresponding_tier(&self) -> u8 {
        *self as u8 + 1
    }

    pub fn next(&self) -> Option<Plane> {
        Self::ALL.get(*self as usize + 1).copied()
    }
}

/// Evidence a caller supplies when asking whether an action has reached a
/// given plane. Intentionally minimal -- only the fields the currently-real
/// checks (Buddhic, Logoic) actually consume; the rest are for the
/// not-yet-wired planes, kept here so the shape is ready when those land.
#[derive(Debug, Clone, Default)]
pub struct PlaneEvidence {
    /// Gender-principle score (0.0-1.0) from this action's HermeticState /
    /// Justice evaluation -- the real signal Buddhic checks.
    pub gender_score: Option<f64>,
    /// A real on-chain receipt/transaction hash, if this outcome has been
    /// settled. Logoic checks only that this is present and non-empty --
    /// it does not itself verify the hash resolves on-chain (the caller is
    /// expected to have already confirmed that before calling).
    pub onchain_receipt_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaneVerification {
    /// The plane's real check passed.
    Verified,
    /// The plane's real check ran and failed -- not a missing integration,
    /// an actual negative result.
    Failed { reason: String },
    /// No real verification exists yet for this plane in this crate. Not a
    /// pass -- callers must not treat this as "verified."
    NotYetWired,
}

/// Evaluate whether `evidence` satisfies `plane`'s real requirement.
/// Physical always passes (it's the zero-proof base case). Buddhic and
/// Logoic run real checks. Everything else honestly returns `NotYetWired`.
pub fn verify_plane(plane: Plane, evidence: &PlaneEvidence) -> PlaneVerification {
    match plane {
        Plane::Physical => PlaneVerification::Verified,
        Plane::Buddhic => match evidence.gender_score {
            Some(score) if score >= 0.5 => PlaneVerification::Verified,
            Some(score) => PlaneVerification::Failed {
                reason: format!(
                    "Gender-principle score {score:.3} below the Buddhic threshold (0.5) -- \
                     creative/receptive balance not demonstrated"
                ),
            },
            None => PlaneVerification::Failed {
                reason: "no gender_score supplied -- Buddhic requires a real ethics evaluation"
                    .to_string(),
            },
        },
        Plane::Logoic => match &evidence.onchain_receipt_hash {
            Some(hash) if !hash.trim().is_empty() => PlaneVerification::Verified,
            _ => PlaneVerification::Failed {
                reason: "no on-chain receipt hash supplied -- Logoic requires a settled outcome"
                    .to_string(),
            },
        },
        Plane::Astral | Plane::Mental | Plane::Atmic | Plane::Monadic => {
            PlaneVerification::NotYetWired
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_planes_in_locked_order() {
        assert_eq!(Plane::ALL.len(), 7);
        assert_eq!(Plane::ALL[0], Plane::Physical);
        assert_eq!(Plane::ALL[6], Plane::Logoic);
    }

    #[test]
    fn corresponding_tier_matches_locked_table() {
        assert_eq!(Plane::Physical.corresponding_tier(), 1);
        assert_eq!(Plane::Astral.corresponding_tier(), 2);
        assert_eq!(Plane::Mental.corresponding_tier(), 3);
        assert_eq!(Plane::Buddhic.corresponding_tier(), 4);
        assert_eq!(Plane::Atmic.corresponding_tier(), 5);
        assert_eq!(Plane::Monadic.corresponding_tier(), 6);
        assert_eq!(Plane::Logoic.corresponding_tier(), 7);
    }

    #[test]
    fn next_ascends_in_order_and_stops_at_logoic() {
        assert_eq!(Plane::Physical.next(), Some(Plane::Astral));
        assert_eq!(Plane::Monadic.next(), Some(Plane::Logoic));
        assert_eq!(Plane::Logoic.next(), None);
    }

    #[test]
    fn physical_always_verifies() {
        let evidence = PlaneEvidence::default();
        assert_eq!(
            verify_plane(Plane::Physical, &evidence),
            PlaneVerification::Verified
        );
    }

    #[test]
    fn buddhic_requires_real_gender_score() {
        let no_evidence = PlaneEvidence::default();
        assert!(matches!(
            verify_plane(Plane::Buddhic, &no_evidence),
            PlaneVerification::Failed { .. }
        ));

        let low = PlaneEvidence {
            gender_score: Some(0.2),
            ..Default::default()
        };
        assert!(matches!(
            verify_plane(Plane::Buddhic, &low),
            PlaneVerification::Failed { .. }
        ));

        let high = PlaneEvidence {
            gender_score: Some(0.9),
            ..Default::default()
        };
        assert_eq!(
            verify_plane(Plane::Buddhic, &high),
            PlaneVerification::Verified
        );
    }

    #[test]
    fn logoic_requires_real_onchain_hash() {
        let no_evidence = PlaneEvidence::default();
        assert!(matches!(
            verify_plane(Plane::Logoic, &no_evidence),
            PlaneVerification::Failed { .. }
        ));

        let empty = PlaneEvidence {
            onchain_receipt_hash: Some("   ".to_string()),
            ..Default::default()
        };
        assert!(matches!(
            verify_plane(Plane::Logoic, &empty),
            PlaneVerification::Failed { .. }
        ));

        let real = PlaneEvidence {
            onchain_receipt_hash: Some("0xabc123".to_string()),
            ..Default::default()
        };
        assert_eq!(
            verify_plane(Plane::Logoic, &real),
            PlaneVerification::Verified
        );
    }

    #[test]
    fn unwired_planes_never_silently_pass() {
        let evidence = PlaneEvidence {
            gender_score: Some(1.0),
            onchain_receipt_hash: Some("0xabc123".to_string()),
        };
        for plane in [Plane::Astral, Plane::Mental, Plane::Atmic, Plane::Monadic] {
            assert_eq!(
                verify_plane(plane, &evidence),
                PlaneVerification::NotYetWired,
                "{:?} must not silently verify just because unrelated evidence was supplied",
                plane
            );
        }
    }
}
