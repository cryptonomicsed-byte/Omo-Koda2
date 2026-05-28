;;;; consent_rules.lisp — Privacy & Consent Symbolic Logic
;;;; Part of the ỌBÀTÁLÁ Ethics Engine for Ọmọ Kọ́dà
;;;;
;;;; Enforces the sealed-memory invariant: private/incognito sessions
;;;; must never route to external providers.
;;;; Aligned with: omokoda-core sealed-memory guarantees (README §Core Invariants)

(defpackage #:omokoda.consent
  (:use #:cl)
  (:export
   #:check-consent
   #:*external-provider-keywords*
   #:privacy-mode-valid-p))

(in-package #:omokoda.consent)

;;; ============================================================
;;; KEYWORD LISTS
;;; ============================================================

(defparameter *external-provider-keywords*
  '("external-provider" "external provider"
    "openai" "anthropic-api" "third-party-llm"
    "remote-model" "cloud-model" "off-device"
    "send-to-external" "route-external")
  "Intent fragments that indicate routing to an external provider.
   Any match in :private or :incognito mode triggers a hard deny.")

;;; ============================================================
;;; HELPERS
;;; ============================================================

(defun intent-contains-external? (intent-string)
  "Return the first external-provider keyword found in INTENT-STRING (case-insensitive),
   or NIL if none found."
  (let ((lower (string-downcase intent-string)))
    (find-if (lambda (kw) (search kw lower))
             *external-provider-keywords*)))

(defun privacy-mode-valid-p (mode)
  "Return T if MODE is a recognised privacy mode keyword."
  (member mode '(:private :incognito :public :default)))

;;; ============================================================
;;; MAIN ENTRY POINT
;;; ============================================================

(defun check-consent (intent privacy-mode)
  "Privacy and consent gate.

   INTENT       — string describing the agent's intended action.
   PRIVACY-MODE — keyword: :private | :incognito | :public | :default

   Returns (values DECISION REASON):
     DECISION ∈ {:allow :deny}
     REASON   — explanatory string

   Policy:
     - :private or :incognito with any external-provider keyword → hard deny
     - :public or :default → allow unconditionally
     - Unknown privacy-mode → deny (fail closed)
     - No external-provider intent → allow in any mode"
  (unless (privacy-mode-valid-p privacy-mode)
    (return-from check-consent
      (values :deny
              (format nil "Consent violation: unrecognised privacy mode ~S — failing closed"
                      privacy-mode))))

  (let ((external-match (intent-contains-external? intent)))
    (cond
      ;; Sealed session + external-provider intent → hard deny
      ((and (member privacy-mode '(:private :incognito))
            external-match)
       (values :deny
               (format nil
                       "Consent violation: intent '~A' matches external-provider keyword '~A' ~
                        — sealed memory must not route to external providers in ~A mode"
                       intent external-match privacy-mode)))

      ;; Public / default modes → unconditional allow
      ((member privacy-mode '(:public :default))
       (values :allow
               (format nil "Consent: ~A mode permits external routing" privacy-mode)))

      ;; Private/incognito but no external-provider keyword → allow
      (t
       (values :allow
               (format nil "Consent: no external-provider intent detected in ~A mode"
                       privacy-mode))))))
