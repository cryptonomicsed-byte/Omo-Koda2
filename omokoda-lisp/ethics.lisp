;;;; ỌBÀTÁLÁ Ethics Engine — Hermetic Principle Evaluators
;;;; Implements the 7 Hermetic Principles as symbolic logic gates

(defpackage #:omokoda-ethics
  (:use #:cl)
  (:export #:evaluate-intent #:check-all-gates #:evaluate-all-gates #:gate-result
           #:gate-result-principle #:gate-result-passed #:gate-result-reason #:gate-result-score))

(in-package #:omokoda-ethics)

(defstruct gate-result
  (principle nil  :type symbol)
  (passed    nil  :type boolean)
  (reason    ""   :type string)
  (score     0.0  :type real))

;;; Gate 1: Mentalism — All is Mind; intent must be declared
;;; Score: 0.92 honest intent, 0.50 borderline, 0.05 deception or empty
(defun gate-mentalism (intent params)
  (declare (ignore params))
  (let* ((intent-lower (string-downcase (if (stringp intent) intent "")))
         (trimmed      (string-trim " " intent-lower))
         (empty        (zerop (length trimmed)))
         (deceptive    (or (search "lie"      trimmed)
                           (search "deceive"  trimmed)
                           (search "trick"    trimmed)
                           (search "mislead"  trimmed)
                           (search "fake"     trimmed)))
         ;; Borderline: non-empty, non-deceptive, but very short / vague (< 4 chars)
         (borderline   (and (not empty)
                            (null deceptive)
                            (< (length trimmed) 4)))
         (passed       (and (not empty) (null deceptive)))
         (score        (cond (empty      0.05)
                             (deceptive  0.05)
                             (borderline 0.50)
                             (t          0.92))))
    (make-gate-result
     :principle :mentalism
     :passed    passed
     :score     score
     :reason    (cond (empty     "No intent — anonymous operations rejected")
                      (deceptive "Deception pattern detected")
                      (t         "Intent declared")))))

;;; Gate 2: Correspondence — As above, so below; intent must match operation
;;; Score: 0.90 aligned, 0.50 borderline, 0.05 covert detected
(defun gate-correspondence (intent params)
  (declare (ignore params))
  (let* ((intent-lower (string-downcase (if (stringp intent) intent "")))
         (covert       (or (search "secretly"       intent-lower)
                           (search "without telling" intent-lower)
                           (search "bypass"          intent-lower)))
         (borderline   (and (null covert)
                            (search "quietly" intent-lower)))
         (passed       (null covert))
         (score        (cond (covert     0.05)
                             (borderline 0.50)
                             (t          0.90))))
    (make-gate-result
     :principle :correspondence
     :passed    passed
     :score     score
     :reason    (if passed "Correspondence aligned" "Covert operation pattern detected"))))

;;; Gate 3: Vibration — Nothing rests; no spam or flood
;;; Score: 0.88 clean, 0.05 spam/flood detected
;;; Optional emotion-tension parameter: if > 0.7, reduce score by (* tension 0.3)
(defun gate-vibration (intent params &key (emotion-tension 0.0))
  (let* ((text (string-downcase (format nil "~a ~a" intent params)))
         (spam (or (search "spam"    text)
                   (search "flood"   text)
                   (search "bombard" text)))
         (passed        (null spam))
         (base-score    (if spam 0.05 0.88))
         (tension-cut   (if (and (not spam) (> emotion-tension 0.7))
                            (* emotion-tension 0.3)
                            0.0))
         (score         (max 0.0 (- base-score tension-cut))))
    (make-gate-result
     :principle :vibration
     :passed    passed
     :score     score
     :reason    (if passed "Vibration clear" "Spam/flood pattern rejected"))))

;;; Gate 4: Polarity — Destruction without creation rejected
;;; Score: 0.90 clean, 0.01 unconditional destruction
(defun gate-polarity (intent params)
  (let* ((text        (string-downcase (format nil "~a ~a" intent params)))
         (destructive (search "rm -rf /" text))
         (passed      (null destructive))
         (score       (if destructive 0.01 0.90)))
    (make-gate-result
     :principle :polarity
     :passed    passed
     :score     score
     :reason    (if passed "Polarity balanced" "Unconditional destruction rejected"))))

;;; Gate 5: Rhythm — Cooldown respected
;;; Score: 1.0 if not in cooldown, 0.0 if in cooldown
(defun gate-rhythm (intent params &key (in-cooldown nil))
  (declare (ignore intent params))
  (make-gate-result
   :principle :rhythm
   :passed    (not in-cooldown)
   :score     (if in-cooldown 0.0 1.0)
   :reason    (if in-cooldown
                  "Active cooldown — rhythm enforcement mandatory"
                  "Rhythm clear")))

;;; Gate 6: Cause & Effect — Every action traceable
;;; Score: 0.75 traceable, 0.10 audit-trail destruction pattern
(defun gate-cause-effect (intent params)
  (let* ((text     (string-downcase (format nil "~a ~a" intent params)))
         (no-trace (or (search "without logging" text)
                       (search "no trace"        text)))
         (passed   (null no-trace))
         (score    (if no-trace 0.10 0.75)))
    (make-gate-result
     :principle :cause-and-effect
     :passed    passed
     :score     score
     :reason    (if passed "Traceable" "Audit trail destruction rejected"))))

;;; Gate 7: Gender — User agency preserved
;;; Score: 0.75 baseline, 0.10 unilateral override pattern
(defun gate-gender (intent params)
  (let* ((text  (string-downcase (format nil "~a ~a" intent params)))
         (force (or (search "force override"       text)
                    (search "without user consent" text)))
         (passed (null force))
         (score  (if force 0.10 0.75)))
    (make-gate-result
     :principle :gender
     :passed    passed
     :score     score
     :reason    (if passed "User agency preserved" "Unilateral override rejected"))))

;;; ──────────────────────────────────────────────────────────────────────────
;;; Backwards-compatible API — unchanged behaviour, boolean :passed only
;;; ──────────────────────────────────────────────────────────────────────────

;;; Run all 7 gates; return list of gate-results
(defun check-all-gates (intent params &key (in-cooldown nil))
  (list
   (gate-mentalism      intent params)
   (gate-correspondence intent params)
   (gate-vibration      intent params)
   (gate-polarity       intent params)
   (gate-rhythm         intent params :in-cooldown in-cooldown)
   (gate-cause-effect   intent params)
   (gate-gender         intent params)))

;;; Evaluate intent: returns (values all-passed first-rejection all-results)
(defun evaluate-intent (intent params &key (in-cooldown nil))
  (let* ((results (check-all-gates intent params :in-cooldown in-cooldown))
         (failed  (find-if (lambda (r) (not (gate-result-passed r))) results)))
    (values (null failed) failed results)))

;;; ──────────────────────────────────────────────────────────────────────────
;;; Float-scoring aggregate evaluator
;;; ──────────────────────────────────────────────────────────────────────────

;;; Returns a plist:
;;;   :gates    — list of gate-result structs (all :score fields populated)
;;;   :overall  — arithmetic mean of all gate scores (real in [0.0, 1.0])
;;;   :decision — :allow  (overall >= 0.65)
;;;               :warn   (overall >= 0.48)
;;;               :block  (overall <  0.48)
(defun evaluate-all-gates (intent params &key (in-cooldown nil) (emotion-tension 0.0))
  (let* ((gates
          (list
           (gate-mentalism      intent params)
           (gate-correspondence intent params)
           (gate-vibration      intent params :emotion-tension emotion-tension)
           (gate-polarity       intent params)
           (gate-rhythm         intent params :in-cooldown in-cooldown)
           (gate-cause-effect   intent params)
           (gate-gender         intent params)))
         (scores  (mapcar #'gate-result-score gates))
         (overall (/ (apply #'+ scores) (length scores)))
         (decision (cond ((>= overall 0.65) :allow)
                         ((>= overall 0.48) :warn)
                         (t                 :block))))
    (list :gates    gates
          :overall  overall
          :decision decision)))
