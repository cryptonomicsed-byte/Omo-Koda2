;;;; sbcl_init.lisp — SBCL Startup Script for the ỌBÀTÁLÁ Ethics Engine
;;;; Part of the ỌBÀTÁLÁ Ethics Engine for Ọmọ Kọ́dà
;;;;
;;;; Usage:
;;;;   # Interactive / development REPL:
;;;;   sbcl --load sbcl_init.lisp
;;;;
;;;;   # Build a standalone binary (batch mode):
;;;;   sbcl --script sbcl_init.lisp --build
;;;;
;;;;   # Or from within SBCL:
;;;;   (load "sbcl_init.lisp")
;;;;   (omokoda.build:build-standalone)   ; creates ./omokoda-ethics binary
;;;;
;;;; The standalone binary enters a simple line-oriented REPL that reads
;;;; evaluate-intent and check-consent queries from stdin and writes
;;;; allow/warn/deny responses to stdout.  This lets Rust/other callers
;;;; spawn it as a child process when full FFI is not needed.
;;;;
;;;; No external ASDF systems or Quicklisp packages are required.
;;;; All code uses only SBCL's standard image.

;;; ============================================================
;;; PATHS — all files relative to this script's location
;;; ============================================================

(defparameter *omokoda-lisp-dir*
  (or *load-truename*
      (truename "."))
  "Directory containing the omokoda-lisp source files.")

(defun omokoda-path (filename)
  "Return full pathname for FILENAME inside the omokoda-lisp directory."
  (merge-pathnames filename *omokoda-lisp-dir*))

;;; ============================================================
;;; LOADER — loads files in dependency order
;;; ============================================================

(defun load-omokoda-ethics ()
  "Load all ỌBÀTÁLÁ Ethics Engine source files in dependency order."
  (format t "~&[omokoda-init] Loading ethics engine...~%")
  (let ((files '("ethics.lisp"
                 "consent_rules.lisp"
                 "policy_ast.lisp"
                 "rust_ffi.lisp")))
    (dolist (f files)
      (let ((path (omokoda-path f)))
        (format t "~&[omokoda-init]   Loading ~A~%" path)
        (load path))))
  (format t "~&[omokoda-init] Ethics engine loaded.~%"))

;;; ============================================================
;;; SIMPLE STDIO REPL — for subprocess / pipe usage from Rust
;;; ============================================================

(defun run-ethics-repl ()
  "Run a simple line-oriented REPL on stdin/stdout.

   Protocol (line-based):
     Request:  EVALUATE-INTENT <tier:int> <intent:rest-of-line>
     Response: RESULT allow|warn|deny <reason>

     Request:  CHECK-CONSENT <mode:word> <intent:rest-of-line>
     Response: RESULT allow|deny <reason>

     Request:  QUIT
     Response: BYE (then exit 0)"
  (format t "~&OMOKODA-ETHICS-READY~%")
  (force-output)
  (loop
    (let ((line (read-line *standard-input* nil nil)))
      (unless line (return))
      (let ((trimmed (string-trim '(#\Space #\Tab #\Return) line)))
        (cond
          ((string= trimmed "") nil) ; skip blanks

          ((string= (string-upcase trimmed) "QUIT")
           (format t "BYE~%")
           (force-output)
           (return))

          ((and (>= (length trimmed) 15)
                (string= (string-upcase (subseq trimmed 0 15)) "EVALUATE-INTENT"))
           ;; EVALUATE-INTENT <tier> <intent...>
           (handler-case
               (let* ((rest    (string-trim " " (subseq trimmed 15)))
                      (sp      (position #\Space rest))
                      (tier    (if sp (parse-integer (subseq rest 0 sp) :junk-allowed t) 0))
                      (intent  (if sp (subseq rest (1+ sp)) rest))
                      (safe-tier (if (and tier (<= 0 tier 5)) tier 0)))
                 (multiple-value-bind (result reason)
                     (omokoda.ethics:evaluate-intent intent safe-tier)
                   (format t "RESULT ~A ~A~%" result reason)))
             (error (e)
               (format t "RESULT deny error: ~A~%" e)))
           (force-output))

          ((and (>= (length trimmed) 13)
                (string= (string-upcase (subseq trimmed 0 13)) "CHECK-CONSENT"))
           ;; CHECK-CONSENT <mode> <intent...>
           (handler-case
               (let* ((rest   (string-trim " " (subseq trimmed 13)))
                      (sp     (position #\Space rest))
                      (mode-s (if sp (subseq rest 0 sp) rest))
                      (intent (if sp (subseq rest (1+ sp)) ""))
                      (mode   (omokoda.ffi:string->privacy-mode mode-s)))
                 (multiple-value-bind (result reason)
                     (omokoda.consent:check-consent intent mode)
                   (format t "RESULT ~A ~A~%" result reason)))
             (error (e)
               (format t "RESULT deny error: ~A~%" e)))
           (force-output))

          (t
           (format t "ERROR unknown command: ~A~%" trimmed)
           (force-output)))))))

;;; ============================================================
;;; BUILD ENTRY POINT
;;; ============================================================

(defpackage #:omokoda.build
  (:use #:cl)
  (:export #:build-standalone))

(in-package #:omokoda.build)

(defun build-standalone (&optional (output-path "omokoda-ethics"))
  "Build a standalone SBCL binary at OUTPUT-PATH that embeds the ethics engine.
   The binary runs the stdio REPL when invoked with no arguments,
   or executes a single query when called as:
     omokoda-ethics eval <tier> <intent>
     omokoda-ethics consent <mode> <intent>"
  #+sbcl
  (progn
    (cl-user::load-omokoda-ethics)
    (sb-ext:save-lisp-and-die
     output-path
     :executable t
     :toplevel (lambda ()
                 ;; Parse minimal CLI args
                 (let ((args (rest sb-ext:*posix-argv*)))
                   (cond
                     ;; Batch mode: omokoda-ethics eval <tier> <intent>
                     ((and (>= (length args) 3)
                           (string= (first args) "eval"))
                      (let* ((tier   (parse-integer (second args) :junk-allowed t))
                             (intent (third args))
                             (safe-tier (if (and tier (<= 0 tier 5)) tier 0)))
                        (multiple-value-bind (result reason)
                            (omokoda.ethics:evaluate-intent intent safe-tier)
                          (format t "~A ~A~%" result reason))
                        (sb-ext:exit :code (case result (:allow 0) (:warn 1) (t 2)))))
                     ;; Batch mode: omokoda-ethics consent <mode> <intent>
                     ((and (>= (length args) 3)
                           (string= (first args) "consent"))
                      (let* ((mode   (omokoda.ffi:string->privacy-mode (second args)))
                             (intent (third args)))
                        (multiple-value-bind (result reason)
                            (omokoda.consent:check-consent intent mode)
                          (format t "~A ~A~%" result reason))
                        (sb-ext:exit :code (if (eq result :allow) 0 2))))
                     ;; REPL mode (default)
                     (t
                      (cl-user::run-ethics-repl)
                      (sb-ext:exit :code 0)))))
     :compression t))
  #-sbcl
  (error "build-standalone requires SBCL"))

(in-package #:cl-user)

;;; ============================================================
;;; AUTO-LOAD on script entry
;;; ============================================================

;; Always load the engine when this file is loaded/scripted
(load-omokoda-ethics)

;; When invoked as --script (non-interactive), check if --build was requested
#+sbcl
(when (and (not *load-truename*)  ; running as --script, not --load
           (member "--build" sb-ext:*posix-argv* :test #'string=))
  (omokoda.build:build-standalone))
