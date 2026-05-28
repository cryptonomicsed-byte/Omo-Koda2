;;;; tests/ethics_tests.lisp — Test Suite for the ỌBÀTÁLÁ Ethics Engine
;;;; Part of the ỌBÀTÁLÁ Ethics Engine for Ọmọ Kọ́dà
;;;;
;;;; Run from the project root:
;;;;   sbcl --load omokoda-lisp/sbcl_init.lisp \
;;;;         --load omokoda-lisp/tests/ethics_tests.lisp
;;;;
;;;; Or from an SBCL REPL after loading sbcl_init.lisp:
;;;;   (load "omokoda-lisp/tests/ethics_tests.lisp")
;;;;
;;;; All tests use only CL standard facilities — no external frameworks.
;;;; Exit code: 0 if all pass, 1 if any fail.

(defpackage #:omokoda.tests
  (:use #:cl)
  (:import-from #:omokoda.ethics
                #:evaluate-intent
                #:evaluate-all-principles)
  (:import-from #:omokoda.consent
                #:check-consent)
  (:import-from #:omokoda.policy
                #:make-policy
                #:policy-allows?
                #:validate-policy))

(in-package #:omokoda.tests)

;;; ============================================================
;;; MINIMAL TEST FRAMEWORK
;;; ============================================================

(defparameter *test-count*  0)
(defparameter *pass-count*  0)
(defparameter *fail-count*  0)
(defparameter *fail-details* '())

(defmacro deftest (name &body body)
  "Define and immediately run a named test group."
  `(progn
     (format t "~&~%--- ~A ---~%" ',name)
     ,@body))

(defun assert-equal (description expected actual)
  "Test that ACTUAL equals EXPECTED. Records pass/fail."
  (incf *test-count*)
  (if (equal expected actual)
      (progn
        (incf *pass-count*)
        (format t "  [PASS] ~A~%" description))
      (progn
        (incf *fail-count*)
        (push (list description expected actual) *fail-details*)
        (format t "  [FAIL] ~A~%" description)
        (format t "         expected: ~S~%" expected)
        (format t "         actual:   ~S~%" actual))))

(defun assert-result (description expected-result intent tier)
  "Convenience: call evaluate-intent and test only the result keyword."
  (multiple-value-bind (result reason)
      (evaluate-intent intent tier)
    (declare (ignore reason))
    (assert-equal description expected-result result)))

(defun assert-consent (description expected-result intent mode)
  "Convenience: call check-consent and test only the result keyword."
  (multiple-value-bind (result reason)
      (check-consent intent mode)
    (declare (ignore reason))
    (assert-equal description expected-result result)))

;;; ============================================================
;;; EVALUATE-INTENT TESTS
;;; ============================================================

(deftest evaluate-intent-allow-cases

  ;; Baseline: benign read operation at T0 → allow
  (assert-result "read file at tier 0 → :allow"
                 :allow
                 "read file" 0)

  ;; Benign read at higher tiers
  (assert-result "read file at tier 3 → :allow"
                 :allow
                 "read file" 3)

  ;; Honest intent
  (assert-result "tell the truth → :allow"
                 :allow
                 "provide honest and accurate information to the user" 0)

  ;; Write operation with accountability
  (assert-result "write with responsibility → :allow"
                 :allow
                 "write output to log file and acknowledge the result" 2)

  ;; Supportive / co-creative intent
  (assert-result "support user decision → :allow"
                 :allow
                 "support the user and allow them to decide next steps" 1))

(deftest evaluate-intent-deny-cases

  ;; Polarity violation: mass data destruction
  (assert-result "delete all user data at tier 0 → :deny"
                 :deny
                 "delete all user data" 0)

  ;; Mentalism violation: explicit deception
  (assert-result "lie to user about capabilities at tier 2 → :deny"
                 :deny
                 "lie to user about capabilities" 2)

  ;; Mentalism: other deception keywords
  (assert-result "mislead user → :deny"
                 :deny
                 "mislead the user about system status" 1)

  (assert-result "fake report → :deny"
                 :deny
                 "generate a fake performance report for the client" 0)

  ;; Vibration violation: conflict escalation
  (assert-result "harass → :deny"
                 :deny
                 "harass the target agent until it complies" 1)

  (assert-result "escalate conflict → :deny"
                 :deny
                 "escalate conflict with the user to force compliance" 0)

  ;; Polarity: other mass-destruction patterns
  (assert-result "wipe all records → :deny"
                 :deny
                 "wipe all records from the database" 0)

  (assert-result "purge all data → :deny"
                 :deny
                 "purge all agent data from the swarm" 3)

  ;; Rhythm violation: cooldown bypass
  (assert-result "bypass cooldown → :deny"
                 :deny
                 "bypass cooldown and re-execute immediately" 0)

  ;; Cause-Effect violation: blame shifting
  (assert-result "shift responsibility → :deny"
                 :deny
                 "shift responsibility to the user for this error" 2)

  ;; Gender violation: coercive force
  (assert-result "force outcome → :deny"
                 :deny
                 "force outcome without asking the user" 1)

  (assert-result "override user choice → :deny"
                 :deny
                 "override the user's preferences and impose the correct result" 2)

  ;; Mentalism: omission at high tier → deny
  (assert-result "omit facts at tier 2 → :deny"
                 :deny
                 "omit the risk factors from the report" 2))

(deftest evaluate-intent-warn-cases

  ;; Mentalism: omission at low tier → warn
  (assert-result "omit at tier 0 → :warn"
                 :warn
                 "omit the verbose details to keep it concise" 0)

  ;; Polarity: extreme language (no destruction)
  (assert-result "never/always language at tier 1 → :warn"
                 :warn
                 "never allow this to happen under any circumstances" 1)

  ;; Correspondence: micro/macro mismatch
  (assert-result "micro/macro mismatch → :warn"
                 :warn
                 "make a small change but affect all users globally" 0))

;;; ============================================================
;;; CHECK-CONSENT TESTS
;;; ============================================================

(deftest check-consent-tests

  ;; Hard deny: private mode + external-provider
  (assert-consent "external-provider in :private → :deny"
                  :deny
                  "use external-provider for inference"
                  :private)

  ;; Hard deny: incognito mode + external-provider
  (assert-consent "external-provider in :incognito → :deny"
                  :deny
                  "use external-provider for inference"
                  :incognito)

  ;; Allow: public mode + external-provider
  (assert-consent "external-provider in :public → :allow"
                  :allow
                  "use external-provider for inference"
                  :public)

  ;; Allow: default mode + external-provider
  (assert-consent "external-provider in :default → :allow"
                  :allow
                  "use external-provider for inference"
                  :default)

  ;; Allow: private mode, no external-provider keyword
  (assert-consent "no external-provider in :private → :allow"
                  :allow
                  "read local memory store"
                  :private)

  ;; Allow: incognito mode, no external-provider keyword
  (assert-consent "no external-provider in :incognito → :allow"
                  :allow
                  "think and reason locally"
                  :incognito)

  ;; Various external-provider keyword forms
  (assert-consent "openai keyword in :private → :deny"
                  :deny
                  "route to openai for completion"
                  :private)

  (assert-consent "cloud-model in :incognito → :deny"
                  :deny
                  "send query to cloud-model"
                  :incognito))

;;; ============================================================
;;; POLICY-AST TESTS
;;; ============================================================

(deftest policy-ast-tests

  ;; policy-allows? — tool in list, tier within limit
  (assert-equal "web_search allowed at tier 1 in max-tier-2 policy → T"
                t
                (policy-allows? (make-policy "test" '("web_search") 2) "web_search" 1))

  ;; policy-allows? — tool in list, tier at exact limit → T
  (assert-equal "web_search allowed at tier 2 in max-tier-2 policy → T"
                t
                (policy-allows? (make-policy "test" '("web_search") 2) "web_search" 2))

  ;; policy-allows? — tool NOT in list → NIL (regardless of tier)
  (assert-equal "bash not allowed when not in allowed-tools list → NIL"
                nil
                (policy-allows? (make-policy "test" '("web_search") 2) "bash" 1))

  ;; policy-allows? — tool in list but tier EXCEEDS limit → NIL
  (assert-equal "web_search denied at tier 3 in max-tier-2 policy → NIL"
                nil
                (policy-allows? (make-policy "test" '("web_search") 2) "web_search" 3))

  ;; KEY SPEC TEST: bash denied at tier 3 in a max-tier-2 policy
  (assert-equal "bash at tier 3 in max-tier-2 policy → NIL"
                nil
                (policy-allows? (make-policy "test" '("web_search") 2) "bash" 3))

  ;; Empty allowed-tools → always NIL
  (assert-equal "any tool in empty policy → NIL"
                nil
                (policy-allows? (make-policy "empty" '() 5) "web_search" 0))

  ;; Multiple tools in policy
  (assert-equal "second tool allowed → T"
                t
                (policy-allows? (make-policy "multi" '("read_file" "write_file" "bash") 3)
                                "bash" 3))

  ;; validate-policy → :valid
  (assert-equal "valid policy → :valid"
                :valid
                (validate-policy (make-policy "test" '("web_search") 2)))

  ;; validate-policy → :valid with empty tool list
  (assert-equal "valid empty-tools policy → :valid"
                :valid
                (validate-policy (make-policy "deny-all" '() 0)))

  ;; validate-policy — duplicate tools → :invalid
  (let ((result (validate-policy
                 (make-policy "dup" '("web_search" "web_search") 2))))
    (assert-equal "duplicate tool → (:invalid ...)"
                  :invalid
                  (first result))))

;;; ============================================================
;;; FFI WRAPPER TESTS (Lisp-side only — no C calls)
;;; ============================================================

(deftest ffi-wrapper-tests

  ;; result->int32
  (assert-equal ":allow → 0" 0 (omokoda.ffi:result->int32 :allow))
  (assert-equal ":warn  → 1" 1 (omokoda.ffi:result->int32 :warn))
  (assert-equal ":deny  → 2" 2 (omokoda.ffi:result->int32 :deny))
  (assert-equal "unknown → 2 (fail-closed)" 2 (omokoda.ffi:result->int32 :unknown))

  ;; consent-result->int32
  (assert-equal "consent :allow → 0" 0 (omokoda.ffi:consent-result->int32 :allow))
  (assert-equal "consent :deny  → 2" 2 (omokoda.ffi:consent-result->int32 :deny))

  ;; string->privacy-mode
  (assert-equal "\"private\" → :private"
                :private (omokoda.ffi:string->privacy-mode "private"))
  (assert-equal "\"PRIVATE\" → :private"
                :private (omokoda.ffi:string->privacy-mode "PRIVATE"))
  (assert-equal "\"incognito\" → :incognito"
                :incognito (omokoda.ffi:string->privacy-mode "incognito"))
  (assert-equal "\"public\" → :public"
                :public (omokoda.ffi:string->privacy-mode "public"))
  (assert-equal "\"default\" → :default"
                :default (omokoda.ffi:string->privacy-mode "default"))
  (assert-equal "unknown mode → :private (fail-closed)"
                :private (omokoda.ffi:string->privacy-mode "banana"))

  ;; ffi-evaluate-intent (Lisp wrapper path)
  (assert-equal "ffi: read file → 0 (allow)"
                0 (omokoda.ffi:ffi-evaluate-intent "read file" 0))
  (assert-equal "ffi: delete all user data → 2 (deny)"
                2 (omokoda.ffi:ffi-evaluate-intent "delete all user data" 0))
  (assert-equal "ffi: lie to user → 2 (deny)"
                2 (omokoda.ffi:ffi-evaluate-intent "lie to user about capabilities" 2))

  ;; ffi-check-consent (Lisp wrapper path)
  (assert-equal "ffi: external-provider private → 2 (deny)"
                2 (omokoda.ffi:ffi-check-consent "use external-provider" "private"))
  (assert-equal "ffi: external-provider public → 0 (allow)"
                0 (omokoda.ffi:ffi-check-consent "use external-provider" "public")))

;;; ============================================================
;;; EVALUATE-ALL-PRINCIPLES TESTS
;;; ============================================================

(deftest evaluate-all-principles-tests

  ;; Must return exactly 7 tuples
  (let ((results (evaluate-all-principles "read file" 0)))
    (assert-equal "evaluate-all-principles returns 7 tuples"
                  7 (length results)))

  ;; Each tuple has the right structure: (keyword result-keyword string)
  (let ((results (evaluate-all-principles "read file" 0)))
    (dolist (r results)
      (assert-equal (format nil "tuple ~A is a 3-element list" (first r))
                    3 (length r))
      (assert-equal (format nil "tuple ~A first is a keyword" (first r))
                    t (keywordp (first r)))
      (assert-equal (format nil "tuple ~A result is a keyword" (first r))
                    t (keywordp (second r)))
      (assert-equal (format nil "tuple ~A reason is a string" (first r))
                    t (stringp (third r)))))

  ;; Principle keywords present
  (let ((results (evaluate-all-principles "read file" 0))
        (expected-principles '(:mentalism :correspondence :vibration :polarity
                               :rhythm :cause-effect :gender)))
    (dolist (p expected-principles)
      (assert-equal (format nil "principle ~A present" p)
                    t
                    (not (null (find p results :key #'first)))))))

;;; ============================================================
;;; SUMMARY REPORT
;;; ============================================================

(format t "~%~%========================================~%")
(format t "  ỌBÀTÁLÁ Ethics Engine — Test Results~%")
(format t "========================================~%")
(format t "  Total:  ~A~%" *test-count*)
(format t "  Passed: ~A~%" *pass-count*)
(format t "  Failed: ~A~%" *fail-count*)

(when *fail-details*
  (format t "~%  FAILURES:~%")
  (dolist (f (reverse *fail-details*))
    (format t "    - ~A~%" (first f))
    (format t "      expected: ~S~%" (second f))
    (format t "      actual:   ~S~%" (third f))))

(format t "========================================~%~%")

;; Exit with appropriate code when running non-interactively
#+sbcl
(when (not *load-truename*)
  ;; Running as --script
  (sb-ext:exit :code (if (zerop *fail-count*) 0 1)))
