// Primitive dispatcher — validates and routes birth/think/act to internal handlers.
// Only these three primitives are the public surface; all other operations are secondary.
// This module is the formal enforcement point for the 3-primitive surface invariant.
use crate::identity::user::UserIdentity;
use crate::Primitive;

/// Validates a Primitive before it enters the interpreter loop.
pub struct PrimitiveDispatcher;

/// Dispatch error with context about which primitive failed and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchError {
    pub primitive: &'static str,
    pub reason: String,
}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dispatch error [{}]: {}", self.primitive, self.reason)
    }
}

impl PrimitiveDispatcher {
    pub fn new() -> Self {
        Self
    }

    /// Validate a Primitive against its constraints before routing.
    /// Returns Ok(()) if the primitive is well-formed and permitted for this identity.
    pub fn validate(
        &self,
        primitive: &Primitive,
        identity: &UserIdentity,
    ) -> Result<(), DispatchError> {
        match primitive {
            Primitive::Birth { name, .. } => {
                if name.is_empty() {
                    return Err(DispatchError {
                        primitive: "birth",
                        reason: "agent name must not be empty".to_string(),
                    });
                }
                if name.len() > 64 {
                    return Err(DispatchError {
                        primitive: "birth",
                        reason: "agent name exceeds 64-character limit".to_string(),
                    });
                }
                Ok(())
            }
            Primitive::Think { prompt, private } => {
                if prompt.is_empty() {
                    return Err(DispatchError {
                        primitive: "think",
                        reason: "intent prompt must not be empty".to_string(),
                    });
                }
                // Incognito sessions implicitly enable private mode.
                if identity.is_incognito() && !private {
                    // Allow but note: the interpreter will enforce local-only regardless.
                }
                Ok(())
            }
            Primitive::Act { tool, .. } => {
                if tool.is_empty() {
                    return Err(DispatchError {
                        primitive: "act",
                        reason: "tool name must not be empty".to_string(),
                    });
                }
                Ok(())
            }
        }
    }

    /// Primitive name as a &str — used for rhythm and receipt tracking.
    pub fn primitive_name(primitive: &Primitive) -> &'static str {
        match primitive {
            Primitive::Birth { .. } => "birth",
            Primitive::Think { .. } => "think",
            Primitive::Act { .. } => "act",
        }
    }

    /// Returns true if the primitive should be subject to rhythm cooldown checks.
    /// Birth is exempt (agents can only be born once per session).
    pub fn is_rhythm_gated(primitive: &Primitive) -> bool {
        matches!(primitive, Primitive::Think { .. } | Primitive::Act { .. })
    }
}

impl Default for PrimitiveDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::user::{PrivacyMode, UserIdentity};

    fn public_identity() -> UserIdentity {
        let mut u = UserIdentity::from_seed("test-meta");
        u.apply_privacy(PrivacyMode::Public);
        u
    }

    fn incognito_identity() -> UserIdentity {
        let mut u = UserIdentity::from_seed("test-incog");
        u.apply_privacy(PrivacyMode::Incognito);
        u
    }

    #[test]
    fn birth_validates_name() {
        let d = PrimitiveDispatcher::new();
        let id = public_identity();

        assert!(d
            .validate(
                &Primitive::Birth {
                    name: "oracle".to_string(),
                    metadata: vec![]
                },
                &id
            )
            .is_ok());
        assert!(d
            .validate(
                &Primitive::Birth {
                    name: "".to_string(),
                    metadata: vec![]
                },
                &id
            )
            .is_err());
        assert!(d
            .validate(
                &Primitive::Birth {
                    name: "x".repeat(65),
                    metadata: vec![]
                },
                &id
            )
            .is_err());
    }

    #[test]
    fn think_validates_prompt() {
        let d = PrimitiveDispatcher::new();
        let id = public_identity();

        assert!(d
            .validate(
                &Primitive::Think {
                    prompt: "hello".to_string(),
                    private: false
                },
                &id
            )
            .is_ok());
        assert!(d
            .validate(
                &Primitive::Think {
                    prompt: "".to_string(),
                    private: false
                },
                &id
            )
            .is_err());
    }

    #[test]
    fn think_incognito_is_valid() {
        let d = PrimitiveDispatcher::new();
        let id = incognito_identity();
        // Incognito think is valid — interpreter enforces local-only
        assert!(d
            .validate(
                &Primitive::Think {
                    prompt: "anything".to_string(),
                    private: false
                },
                &id
            )
            .is_ok());
    }

    #[test]
    fn act_validates_tool_name() {
        let d = PrimitiveDispatcher::new();
        let id = public_identity();

        assert!(d
            .validate(
                &Primitive::Act {
                    tool: "read".to_string(),
                    params: "{}".to_string(),
                    sandbox: false
                },
                &id
            )
            .is_ok());
        assert!(d
            .validate(
                &Primitive::Act {
                    tool: "".to_string(),
                    params: "{}".to_string(),
                    sandbox: false
                },
                &id
            )
            .is_err());
    }

    #[test]
    fn primitive_names_are_correct() {
        assert_eq!(
            PrimitiveDispatcher::primitive_name(&Primitive::Birth {
                name: "x".to_string(),
                metadata: vec![]
            }),
            "birth"
        );
        assert_eq!(
            PrimitiveDispatcher::primitive_name(&Primitive::Think {
                prompt: "x".to_string(),
                private: false
            }),
            "think"
        );
        assert_eq!(
            PrimitiveDispatcher::primitive_name(&Primitive::Act {
                tool: "x".to_string(),
                params: "{}".to_string(),
                sandbox: false
            }),
            "act"
        );
    }

    #[test]
    fn rhythm_gate_excludes_birth() {
        assert!(!PrimitiveDispatcher::is_rhythm_gated(&Primitive::Birth {
            name: "x".to_string(),
            metadata: vec![]
        }));
        assert!(PrimitiveDispatcher::is_rhythm_gated(&Primitive::Think {
            prompt: "x".to_string(),
            private: false
        }));
        assert!(PrimitiveDispatcher::is_rhythm_gated(&Primitive::Act {
            tool: "x".to_string(),
            params: "{}".to_string(),
            sandbox: false
        }));
    }
}
