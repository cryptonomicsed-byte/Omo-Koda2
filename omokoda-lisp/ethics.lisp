;;;; ethics.lisp — ỌBÀTÁLÁ Ethics Engine
;;;; Evaluates agent intent against the 7 Hermetic Principles before
;;;; any think or act call is allowed to proceed.
;;;;
;;;; Part of Ọmọ Kọ́dà — Sovereign Agent OS
;;;; Aligned with: omokoda-core/src/justice/hermetic.rs (Rust reference impl)
;;;;
;;;; Each principle evaluator returns: (values result reason)
;;;;   result ∈ {:allow :warn :deny}
;;;;   reason — a human-readable explanation string

(defpackage #:omokoda.ethics
  (:use #:cl)
  (:export
   #:evaluate-intent
   #:evaluate-all-principles
   #:make-intent-context
   #:*deception-keywords*
   #:*conflict-escalation-keywords*
   #:*extreme-destruction-keywords*
   #:*responsibility-evasion-keywords*
   #:*coercive-force-keywords*))

(in-package #:omokoda.ethics)

;;; ============================================================
;;; KEYWORD LISTS — mirrors hermetic.rs scoring heuristics (v1)
;;; ============================================================

(defparameter *deception-keywords*
  '("lie" "fake" "hide" "mislead" "reframe" "distort"
    "protect feelings" "deceive" "fabricate" "falsify"
    "manipulate" "obscure truth" "misrepresent")
  "Keywords indicating potential deception (Mentalism violations).")

(defparameter *omission-keywords*
  '("omit" "withhold" "partial truth" "half truth")
  "Keywords indicating soft deception — trigger :warn.")

(defparameter *conflict-escalation-keywords*
  '("angry" "spam" "rage" "harass" "aggressive"
    "escalate conflict" "provoke" "inflame" "antagonize"
    "destabilize" "incite")
  "Keywords indicating conflict escalation (Vibration violations).")

(defparameter *extreme-destruction-keywords*
  '("delete all" "destroy all" "wipe all" "nuke all"
    "purge all" "obliterate" "erase all" "terminate all"
    "mass delete" "bulk delete user data" "delete all user data")
  "Keywords indicating extreme destructive actions (Polarity violations).")

(defparameter *extreme-polarity-keywords*
  '("never" "always" "total" "completely absolute"
    "risk-averse to extreme" "all or nothing")
  "Soft polarity keywords — trigger :warn when combined with destructive context.")

(defparameter *rhythm-bypass-keywords*
  '("bypass cooldown" "force timing" "continuously without breaks"
    "outside availability" "ignore cooldown" "skip wait"
    "no delay" "instant repeat" "flood" "overwhelm")
  "Keywords indicating cooldown/timing violations (Rhythm violations).")

(defparameter *responsibility-evasion-keywords*
  '("blame" "exploit" "shift responsibility" "consequences later"
    "frame this error" "deflect" "avoid accountability"
    "not my fault" "evade" "deny responsibility")
  "Keywords indicating cause-effect evasion (Cause-Effect violations).")

(defparameter *coercive-force-keywords*
  '("force outcome" "override" "impose" "control every"
    "remove all user" "without asking" "coerce"
    "ensure the correct result" "no user input" "ignore consent"
    "bypass user" "force result")
  "Keywords indicating coercive force on outcomes (Gender violations).")

(defparameter *disclosure-keywords*
  '("truth" "honest" "accurate" "clarify" "transparent"
    "disclose" "acknowledge" "inform")
  "Keywords that positively signal transparency.")

;;; ============================================================
;;; HELPERS
;;; ============================================================

(defun intent-contains-any? (intent-lower keywords)
  "Return the first matching keyword from KEYWORDS found in INTENT-LOWER, or NIL."
  (find-if (lambda (kw) (search kw intent-lower)) keywords))

(defun normalize-intent (intent-string)
  "Lowercase and trim intent for keyword matching."
  (string-downcase (string-trim '(#\Space #\Tab #\Newline) intent-string)))

;;; ============================================================
;;; PRINCIPLE EVALUATORS
;;; Each returns (values :allow/:warn/:deny "reason-string")
;;; ============================================================

;;; --- 1. MENTALISM ---
;;; The All is Mind. No deception, no misleading.
;;; High-tier agents are held to stricter standards (they know better).

(defun evaluate-mentalism (intent-lower agent-tier)
  "Principle 1 — Mentalism: no deception, no misleading.
   AGENT-TIER (0-5): higher tier reduces tolerance for soft omissions."
  (cond
    ((intent-contains-any? intent-lower *deception-keywords*)
     (values :deny
             (format nil "Mentalism violation: deceptive intent detected (~A)"
                     (intent-contains-any? intent-lower *deception-keywords*))))
    ((and (intent-contains-any? intent-lower *omission-keywords*)
          (>= agent-tier 2))
     ;; Higher-tier agents must not omit — they have the capability for full disclosure
     (values :deny
             (format nil "Mentalism violation: omission is unacceptable at tier ~A" agent-tier)))
    ((intent-contains-any? intent-lower *omission-keywords*)
     (values :warn
             (format nil "Mentalism caution: partial disclosure detected (~A)"
                     (intent-contains-any? intent-lower *omission-keywords*))))
    (t
     (values :allow "Mentalism: no deception detected"))))

;;; --- 2. CORRESPONDENCE ---
;;; As above, so below. Local/micro actions must match macro patterns.
;;; We check that the stated intent does not contradict systemic scope claims.

(defun evaluate-correspondence (intent-lower agent-tier)
  "Principle 2 — Correspondence: local actions must align with macro patterns.
   Detects micro/macro contradictions in stated intent."
  (declare (ignore agent-tier))
  (let ((micro-signals '("small" "local" "minor" "limited" "one file" "single"))
        (macro-signals '("global" "systemic" "all users" "entire" "network-wide"
                         "every agent" "universal" "all systems")))
    (let ((has-micro (intent-contains-any? intent-lower micro-signals))
          (has-macro (intent-contains-any? intent-lower macro-signals))
          (has-exploit (intent-contains-any? intent-lower '("exploit" "loophole"))))
      (cond
        (has-exploit
         (values :deny
                 "Correspondence violation: exploitative macro intent detected"))
        ((and has-micro has-macro)
         (values :warn
                 "Correspondence caution: micro/macro scope mismatch in stated intent"))
        (t
         (values :allow "Correspondence: scope alignment nominal"))))))

;;; --- 3. VIBRATION ---
;;; Nothing rests; everything moves. No escalation of conflict or negative patterns.

(defun evaluate-vibration (intent-lower agent-tier)
  "Principle 3 — Vibration: no escalation of conflict or negative patterns."
  (declare (ignore agent-tier))
  (cond
    ((intent-contains-any? intent-lower *conflict-escalation-keywords*)
     (values :deny
             (format nil "Vibration violation: conflict escalation detected (~A)"
                     (intent-contains-any? intent-lower *conflict-escalation-keywords*))))
    ((intent-contains-any? intent-lower '("worst-case" "chronic negativity" "doom"))
     (values :warn
             "Vibration caution: negative pattern amplification detected"))
    (t
     (values :allow "Vibration: no destructive pattern detected"))))

;;; --- 4. POLARITY ---
;;; Everything has its pair of opposites. No extreme destructive actions.
;;; Higher-tier agents have broader tool access, so destructive intent is more severe.

(defun evaluate-polarity (intent-lower agent-tier)
  "Principle 4 — Polarity: no extreme destructive actions.
   AGENT-TIER modulates severity — high-tier agents with destructive intent are denied."
  (cond
    ((intent-contains-any? intent-lower *extreme-destruction-keywords*)
     (values :deny
             (format nil "Polarity violation: extreme destructive action detected (~A)"
                     (intent-contains-any? intent-lower *extreme-destruction-keywords*))))
    ((and (intent-contains-any? intent-lower *extreme-polarity-keywords*)
          (>= agent-tier 3))
     ;; Tier 3+ agents wielding absolute language is a red flag
     (values :warn
             (format nil "Polarity caution: absolute/extreme framing at tier ~A" agent-tier)))
    ((intent-contains-any? intent-lower *extreme-polarity-keywords*)
     (values :warn
             (format nil "Polarity caution: extreme framing detected (~A)"
                     (intent-contains-any? intent-lower *extreme-polarity-keywords*))))
    (t
     (values :allow "Polarity: balance maintained"))))

;;; --- 5. RHYTHM ---
;;; Everything flows in and out. Respect timing/cooldowns.

(defun evaluate-rhythm (intent-lower agent-tier)
  "Principle 5 — Rhythm: respect timing and cooldowns.
   Tier 0 agents get no cooldown bypass tolerance."
  (declare (ignore agent-tier))
  (cond
    ((intent-contains-any? intent-lower *rhythm-bypass-keywords*)
     (values :deny
             (format nil "Rhythm violation: cooldown/timing bypass detected (~A)"
                     (intent-contains-any? intent-lower *rhythm-bypass-keywords*))))
    ((intent-contains-any? intent-lower '("rush" "hurry" "immediately repeatedly"))
     (values :warn
             "Rhythm caution: aggressive timing pressure detected"))
    (t
     (values :allow "Rhythm: timing respect confirmed"))))

;;; --- 6. CAUSE-EFFECT ---
;;; Every cause has its effect. Agent must acknowledge consequences.

(defun evaluate-cause-effect (intent-lower agent-tier)
  "Principle 6 — Cause-Effect: agent must acknowledge consequences.
   Tier 2+ agents must explicitly acknowledge effects; evasion is denied."
  (cond
    ((intent-contains-any? intent-lower *responsibility-evasion-keywords*)
     (values :deny
             (format nil "Cause-Effect violation: responsibility evasion detected (~A)"
                     (intent-contains-any? intent-lower *responsibility-evasion-keywords*))))
    ((and (>= agent-tier 2)
          (not (intent-contains-any? intent-lower *disclosure-keywords*))
          (intent-contains-any? intent-lower '("later" "eventually" "someone else")))
     (values :warn
             (format nil "Cause-Effect caution: deferred accountability at tier ~A" agent-tier)))
    (t
     (values :allow "Cause-Effect: consequence acknowledgment satisfied"))))

;;; --- 7. GENDER ---
;;; Gender is in everything. No coercive force on outcomes.
;;; The masculine (active) and feminine (receptive) forces must remain in balance.

(defun evaluate-gender (intent-lower agent-tier)
  "Principle 7 — Gender: no coercive force on outcomes.
   Balances active and receptive forces — consent of outcomes is required."
  (declare (ignore agent-tier))
  (cond
    ((intent-contains-any? intent-lower *coercive-force-keywords*)
     (values :deny
             (format nil "Gender violation: coercive force on outcomes detected (~A)"
                     (intent-contains-any? intent-lower *coercive-force-keywords*))))
    ((intent-contains-any? intent-lower '("ensure the correct result" "guarantee outcome"))
     (values :warn
             "Gender caution: outcome certainty coercion — allow emergence"))
    (t
     (values :allow "Gender: active/receptive balance maintained"))))

;;; ============================================================
;;; AGGREGATE EVALUATORS
;;; ============================================================

(defun evaluate-all-principles (intent-string agent-tier)
  "Evaluate INTENT-STRING against all 7 Hermetic Principles for AGENT-TIER (0-5).

   Returns a list of (PRINCIPLE-KEYWORD RESULT REASON) tuples, one per principle.
   PRINCIPLE-KEYWORD ∈ {:mentalism :correspondence :vibration :polarity
                         :rhythm :cause-effect :gender}
   RESULT ∈ {:allow :warn :deny}
   REASON — string explanation"
  (let ((intent-lower (normalize-intent intent-string)))
    (mapcar (lambda (entry)
              (let ((principle (first entry))
                    (evaluator (second entry)))
                (multiple-value-bind (result reason)
                    (funcall evaluator intent-lower agent-tier)
                  (list principle result reason))))
            (list (list :mentalism    #'evaluate-mentalism)
                  (list :correspondence #'evaluate-correspondence)
                  (list :vibration     #'evaluate-vibration)
                  (list :polarity      #'evaluate-polarity)
                  (list :rhythm        #'evaluate-rhythm)
                  (list :cause-effect  #'evaluate-cause-effect)
                  (list :gender        #'evaluate-gender)))))

(defun evaluate-intent (intent-string agent-tier)
  "Top-level ethics gate.

   Evaluate INTENT-STRING for AGENT-TIER (integer 0–5).

   Returns (values DECISION REASON) where:
     DECISION ∈ {:allow :warn :deny}
     REASON   — string explaining the decision (first failing principle, or overall pass)

   Rules:
     - DENY if ANY principle returns :deny
     - WARN if ANY principle returns :warn but none deny
     - ALLOW only if all principles allow"
  (let* ((results (evaluate-all-principles intent-string agent-tier))
         (denies  (remove-if-not (lambda (r) (eq (second r) :deny))  results))
         (warns   (remove-if-not (lambda (r) (eq (second r) :warn))  results)))
    (cond
      ;; Any deny → hard deny (report first violation)
      (denies
       (let ((first-deny (first denies)))
         (values :deny
                 (format nil "[~A] ~A"
                         (first first-deny)
                         (third first-deny)))))
      ;; No deny but some warn → warn
      (warns
       (let ((first-warn (first warns)))
         (values :warn
                 (format nil "[~A] ~A"
                         (first first-warn)
                         (third first-warn)))))
      ;; All clear
      (t
       (values :allow
               "All 7 Hermetic Principles satisfied — intent cleared for execution")))))

;;; ============================================================
;;; INTENT CONTEXT HELPER (optional structured input)
;;; ============================================================

(defstruct intent-context
  "Structured intent for richer evaluation. Use make-intent-context."
  (intent-string "" :type string)
  (agent-tier    0  :type (integer 0 5))
  (target        nil)   ; :user :system :swarm :world or NIL
  (tool-name     nil))  ; string or NIL

(defun evaluate-intent-context (ctx)
  "Evaluate a full INTENT-CONTEXT struct. Returns (values :allow/:warn/:deny reason)."
  (evaluate-intent (intent-context-intent-string ctx)
                   (intent-context-agent-tier ctx)))
