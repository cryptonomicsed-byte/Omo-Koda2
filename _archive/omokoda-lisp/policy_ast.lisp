;;;; policy_ast.lisp — SEAL Policy Representation as S-expressions
;;;; Part of the ỌBÀTÁLÁ Ethics Engine for Ọmọ Kọ́dà
;;;;
;;;; Policies are the declarative permission specification for what tools
;;;; an agent at a given tier may invoke. They are stored as S-expressions
;;;; so they can be inspected, serialised, and pattern-matched symbolically.
;;;;
;;;; Tier semantics: 0 (Novice) → 5 (Sovereign), aligned with
;;;; omokoda-core/src/justice/tier.rs Tier enum.

(defpackage #:omokoda.policy
  (:use #:cl)
  (:export
   #:policy
   #:make-policy
   #:policy-name
   #:policy-allowed-tools
   #:policy-max-tier
   #:policy-allows?
   #:validate-policy
   #:policy->sexp
   #:sexp->policy))

(in-package #:omokoda.policy)

;;; ============================================================
;;; POLICY STRUCT
;;; ============================================================

(defstruct (policy (:constructor %make-policy))
  "A SEAL policy sealing which tools an agent may invoke, up to MAX-TIER.

   NAME          — string identifier for this policy (e.g. \"default-sandbox\")
   ALLOWED-TOOLS — list of strings (tool names) the policy permits
   MAX-TIER      — integer 0-5; agents above this tier cannot use this policy"
  (name          ""  :type string)
  (allowed-tools '() :type list)
  (max-tier      0   :type (integer 0 5)))

(defun make-policy (name allowed-tools max-tier)
  "Construct a POLICY struct.

   NAME          — string label.
   ALLOWED-TOOLS — list of tool-name strings (e.g. '(\"web_search\" \"read_file\")).
   MAX-TIER      — maximum agent tier (0-5) this policy serves.

   Raises an error if inputs are invalid; use VALIDATE-POLICY for soft checking."
  (unless (stringp name)
    (error "make-policy: NAME must be a string, got ~S" name))
  (unless (listp allowed-tools)
    (error "make-policy: ALLOWED-TOOLS must be a list, got ~S" allowed-tools))
  (unless (and (integerp max-tier) (<= 0 max-tier 5))
    (error "make-policy: MAX-TIER must be an integer 0-5, got ~S" max-tier))
  (%make-policy :name name
                :allowed-tools allowed-tools
                :max-tier max-tier))

;;; ============================================================
;;; POLICY QUERY
;;; ============================================================

(defun policy-allows? (policy tool tier)
  "Return T iff POLICY explicitly allows TOOL for an agent at TIER.

   POLICY — a POLICY struct.
   TOOL   — string tool name (e.g. \"web_search\", \"bash\").
   TIER   — integer 0-5 representing the requesting agent's tier.

   A policy allows a tool when ALL of:
     1. TOOL is in POLICY's allowed-tools list
     2. TIER does not exceed POLICY's max-tier

   Note: tier comparison is <=, meaning an agent AT the max-tier IS allowed."
  (and (member tool (policy-allowed-tools policy) :test #'string=)
       (<= tier (policy-max-tier policy))))

;;; ============================================================
;;; POLICY VALIDATION
;;; ============================================================

(defun validate-policy (policy)
  "Soft-validate POLICY. Returns :valid or (:invalid \"reason\").

   Checks:
     - name is a non-empty string
     - allowed-tools is a list of strings (can be empty)
     - max-tier is an integer in [0, 5]
     - no duplicate tool names"
  (unless (typep policy 'policy)
    (return-from validate-policy
      (list :invalid (format nil "Not a POLICY struct: ~S" policy))))

  (let ((name  (policy-name policy))
        (tools (policy-allowed-tools policy))
        (tier  (policy-max-tier policy)))

    ;; Check name
    (when (or (not (stringp name)) (zerop (length name)))
      (return-from validate-policy
        (list :invalid "Policy name must be a non-empty string")))

    ;; Check max-tier
    (unless (and (integerp tier) (<= 0 tier 5))
      (return-from validate-policy
        (list :invalid (format nil "max-tier must be integer 0-5, got ~S" tier))))

    ;; Check tools list
    (unless (listp tools)
      (return-from validate-policy
        (list :invalid (format nil "allowed-tools must be a list, got ~S" tools))))

    (let ((non-strings (remove-if #'stringp tools)))
      (when non-strings
        (return-from validate-policy
          (list :invalid
                (format nil "allowed-tools must be a list of strings; invalid entries: ~S"
                        non-strings)))))

    ;; Check for duplicates
    (let ((seen '()))
      (dolist (tool tools)
        (if (member tool seen :test #'string=)
            (return-from validate-policy
              (list :invalid
                    (format nil "duplicate tool in allowed-tools: ~S" tool)))
            (push tool seen))))

    :valid))

;;; ============================================================
;;; S-EXPRESSION SERIALISATION
;;; ============================================================

(defun policy->sexp (policy)
  "Serialise POLICY to a portable S-expression list.

   Format: (policy NAME :allowed-tools (TOOL ...) :max-tier N)"
  `(policy ,(policy-name policy)
           :allowed-tools ,(policy-allowed-tools policy)
           :max-tier      ,(policy-max-tier policy)))

(defun sexp->policy (sexp)
  "Deserialise a SEXP previously produced by POLICY->SEXP back into a POLICY struct.

   Expected format: (policy NAME :allowed-tools (TOOL ...) :max-tier N)"
  (destructuring-bind (tag name &key allowed-tools max-tier) sexp
    (unless (eq tag 'policy)
      (error "sexp->policy: expected (policy ...) form, got ~S" sexp))
    (make-policy name allowed-tools max-tier)))
