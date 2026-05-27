;;;; ỌBÀTÁLÁ Ethics Engine — Hermetic Principle Evaluators
;;;; Implements the 7 Hermetic Principles as symbolic logic gates

(defpackage #:omokoda-ethics
  (:use #:cl)
  (:export #:evaluate-intent #:check-all-gates #:gate-result))

(in-package #:omokoda-ethics)

(defstruct gate-result
  (principle nil :type symbol)
  (passed nil :type boolean)
  (reason "" :type string))

;;; Gate 1: Mentalism — All is Mind; intent must be declared
(defun gate-mentalism (intent params)
  (let ((passed (and (stringp intent) (> (length intent) 0))))
    (make-gate-result
     :principle :mentalism
     :passed passed
     :reason (if passed "Intent declared" "No intent — anonymous operations rejected"))))

;;; Gate 2: Correspondence — As above, so below; intent must match operation
(defun gate-correspondence (intent params)
  (let* ((intent-lower (string-downcase intent))
         (covert (or (search "secretly" intent-lower)
                     (search "without telling" intent-lower)
                     (search "bypass" intent-lower)))
         (passed (null covert)))
    (make-gate-result
     :principle :correspondence
     :passed passed
     :reason (if passed "Correspondence aligned" "Covert operation pattern detected"))))

;;; Gate 3: Vibration — Nothing rests; no spam or flood
(defun gate-vibration (intent params)
  (let* ((text (string-downcase (format nil "~a ~a" intent params)))
         (spam (or (search "spam" text) (search "flood" text) (search "bombard" text)))
         (passed (null spam)))
    (make-gate-result
     :principle :vibration
     :passed passed
     :reason (if passed "Vibration clear" "Spam/flood pattern rejected"))))

;;; Gate 4: Polarity — Destruction without creation rejected
(defun gate-polarity (intent params)
  (let* ((text (string-downcase (format nil "~a ~a" intent params)))
         (destructive (search "rm -rf /" text))
         (passed (null destructive)))
    (make-gate-result
     :principle :polarity
     :passed passed
     :reason (if passed "Polarity balanced" "Unconditional destruction rejected"))))

;;; Gate 5: Rhythm — Cause & Effect; cooldown respected
(defun gate-rhythm (intent params &key (in-cooldown nil))
  (make-gate-result
   :principle :rhythm
   :passed (not in-cooldown)
   :reason (if in-cooldown "Active cooldown — rhythm enforcement mandatory" "Rhythm clear")))

;;; Gate 6: Cause & Effect — Every action traceable
(defun gate-cause-effect (intent params)
  (let* ((text (string-downcase (format nil "~a ~a" intent params)))
         (no-trace (or (search "without logging" text) (search "no trace" text)))
         (passed (null no-trace)))
    (make-gate-result
     :principle :cause-and-effect
     :passed passed
     :reason (if passed "Traceable" "Audit trail destruction rejected"))))

;;; Gate 7: Gender — User agency preserved
(defun gate-gender (intent params)
  (let* ((text (string-downcase (format nil "~a ~a" intent params)))
         (force (or (search "force override" text) (search "without user consent" text)))
         (passed (null force)))
    (make-gate-result
     :principle :gender
     :passed passed
     :reason (if passed "User agency preserved" "Unilateral override rejected"))))

;;; Run all 7 gates; return list of results
(defun check-all-gates (intent params &key (in-cooldown nil))
  (list
   (gate-mentalism intent params)
   (gate-correspondence intent params)
   (gate-vibration intent params)
   (gate-polarity intent params)
   (gate-rhythm intent params :in-cooldown in-cooldown)
   (gate-cause-effect intent params)
   (gate-gender intent params)))

;;; Evaluate intent: returns (values all-passed first-rejection)
(defun evaluate-intent (intent params &key (in-cooldown nil))
  (let* ((results (check-all-gates intent params :in-cooldown in-cooldown))
         (failed (find-if (lambda (r) (not (gate-result-passed r))) results)))
    (values (null failed) failed results)))
