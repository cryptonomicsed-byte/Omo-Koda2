;;;; rust_ffi.lisp — CFFI Bindings for Rust ↔ SBCL Ethics Bridge
;;;; Part of the ỌBÀTÁLÁ Ethics Engine for Ọmọ Kọ́dà
;;;;
;;;; Exposes two C-callable functions that the Rust omokoda-core crate
;;;; can invoke via FFI when the Lisp ethics engine is loaded as a
;;;; shared library (.so) built from sbcl_init.lisp.
;;;;
;;;; C API:
;;;;   int32_t evaluate_intent(const char* intent, uint8_t tier);
;;;;     Returns: 0=allow, 1=warn, 2=deny
;;;;
;;;;   int32_t check_consent(const char* intent, const char* privacy_mode);
;;;;     Returns: 0=allow, 2=deny
;;;;
;;;; Build note: compile sbcl_init.lisp to create a standalone image;
;;;; the image exports these symbols via sb-alien:define-alien-callable.
;;;; No external CFFI package is required — SBCL's built-in sb-alien
;;;; is used exclusively.

(defpackage #:omokoda.ffi
  (:use #:cl)
  (:import-from #:omokoda.ethics
                #:evaluate-intent)
  (:import-from #:omokoda.consent
                #:check-consent)
  (:export
   #:ffi-evaluate-intent
   #:ffi-check-consent
   #:result->int32
   #:consent-result->int32
   #:string->privacy-mode))

(in-package #:omokoda.ffi)

;;; ============================================================
;;; CONVERSION HELPERS
;;; ============================================================

(defun result->int32 (result)
  "Map a RESULT keyword from evaluate-intent to the C int32 ABI value.
   :allow → 0
   :warn  → 1
   :deny  → 2
   Any unexpected value → 2 (fail closed)"
  (case result
    (:allow 0)
    (:warn  1)
    (:deny  2)
    (t      2)))

(defun consent-result->int32 (result)
  "Map a RESULT keyword from check-consent to the C int32 ABI value.
   :allow → 0
   :deny  → 2
   Any unexpected value → 2 (fail closed)"
  (case result
    (:allow 0)
    (:deny  2)
    (t      2)))

(defun string->privacy-mode (mode-string)
  "Convert a C string privacy mode to the internal keyword.
   Accepted values (case-insensitive):
     \"private\"   → :private
     \"incognito\" → :incognito
     \"public\"    → :public
     \"default\"   → :default
   Anything else → :private (fail closed — unknown modes treated as sealed)"
  (let ((lower (string-downcase (string-trim '(#\Space #\Null) mode-string))))
    (cond
      ((string= lower "private")   :private)
      ((string= lower "incognito") :incognito)
      ((string= lower "public")    :public)
      ((string= lower "default")   :default)
      (t
       ;; Unknown mode: treat as private (fail closed)
       :private))))

;;; ============================================================
;;; LISP-SIDE WRAPPERS (called by the alien callbacks below)
;;; ============================================================

(defun ffi-evaluate-intent (intent-cstring tier-u8)
  "Lisp wrapper for the C-callable evaluate_intent.
   INTENT-CSTRING — string (already decoded from C char*)
   TIER-U8        — integer 0-5

   Returns int32: 0=allow, 1=warn, 2=deny"
  (handler-case
      (let ((tier (max 0 (min 5 tier-u8))))
        (multiple-value-bind (result reason)
            (evaluate-intent intent-cstring tier)
          (declare (ignore reason))
          (result->int32 result)))
    (error (e)
      ;; Any evaluation error → deny (fail closed)
      (format *error-output*
              "~&[omokoda.ffi] evaluate_intent error: ~A~%" e)
      2)))

(defun ffi-check-consent (intent-cstring mode-cstring)
  "Lisp wrapper for the C-callable check_consent.
   INTENT-CSTRING — string (already decoded from C char*)
   MODE-CSTRING   — string (already decoded from C char*)

   Returns int32: 0=allow, 2=deny"
  (handler-case
      (let ((mode (string->privacy-mode mode-cstring)))
        (multiple-value-bind (result reason)
            (check-consent intent-cstring mode)
          (declare (ignore reason))
          (consent-result->int32 result)))
    (error (e)
      (format *error-output*
              "~&[omokoda.ffi] check_consent error: ~A~%" e)
      2)))

;;; ============================================================
;;; SBCL ALIEN CALLBACKS — C-callable export
;;; These define the actual C-ABI entry points when this code is
;;; compiled into a shared library via (save-lisp-and-die ... :executable nil).
;;;
;;; Usage (from Rust via libloading or dlopen):
;;;   let result = evaluate_intent(intent_ptr, tier);
;;; ============================================================

#+sbcl
(progn
  (sb-alien:define-alien-callable evaluate_intent sb-alien:int
    ((intent sb-alien:c-string)
     (tier   sb-alien:unsigned-char))
    "C-callable: evaluate intent string against 7 Hermetic Principles.
     Returns 0=allow, 1=warn, 2=deny."
    (ffi-evaluate-intent intent tier))

  (sb-alien:define-alien-callable check_consent sb-alien:int
    ((intent      sb-alien:c-string)
     (privacy-mode sb-alien:c-string))
    "C-callable: check privacy consent for an intent in a given mode.
     Returns 0=allow, 2=deny."
    (ffi-check-consent intent privacy-mode)))

#-sbcl
(progn
  ;; Stub definitions for non-SBCL implementations (testing / SLIME)
  (defun evaluate_intent (intent tier)
    "Non-SBCL stub — calls Lisp wrapper directly."
    (ffi-evaluate-intent intent tier))

  (defun check_consent (intent privacy-mode)
    "Non-SBCL stub — calls Lisp wrapper directly."
    (ffi-check-consent intent privacy-mode)))
