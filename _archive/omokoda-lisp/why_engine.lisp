;;;; Ọbàtálá Why Engine — Symbolic constitutional reasoning
;;;; Generates human-readable alignment explanations and governs constitutional amendments.
;;;;
;;;; The Why Engine is the deliberative voice of Ọbàtálá. It does not just score —
;;;; it EXPLAINS its reasoning in a traceable symbolic chain. Every alignment
;;;; decision produces a narrative that an agent can surface to itself as an
;;;; internal monologue, or share with the Hive as a cautionary or affirmatory story.
;;;;
;;;; Constitutional amendment requires unanimous consent of all 7 Orisha (7/7)
;;;; plus an unexpercised human veto. Sovereignty requires consensus.

(defpackage #:omokoda-why-engine
  (:use #:cl)
  (:export #:constitutional-principle
           #:make-constitutional-principle
           #:principle-name
           #:principle-floor
           #:principle-weight
           #:principle-rationale
           #:principle-rules
           #:explain-alignment
           #:constitutional-amendment
           #:make-constitutional-amendment
           #:propose-amendment
           #:cast-vote
           #:tally-votes
           #:amendment-enacted-p
           #:enact-amendment!
           #:*hermetic-principles*
           #:*orisha-council*))

(in-package #:omokoda-why-engine)

;; ---------------------------------------------------------------------------
;; Constitutional Principle class
;; ---------------------------------------------------------------------------

(defclass constitutional-principle ()
  ((name
    :initarg :name
    :reader principle-name)
   (floor
    :initarg :floor
    :reader principle-floor
    :initform 0.40)
   (weight
    :initarg :weight
    :reader principle-weight
    :initform 0.143)
   (rationale
    :initarg :rationale
    :reader principle-rationale
    :initform "")
   (inference-rules
    :initarg :inference-rules
    :reader principle-rules
    :initform '())))

(defun make-constitutional-principle (name &key (floor 0.40) (weight 0.143) (rationale "") (rules '()))
  (make-instance 'constitutional-principle
    :name            name
    :floor           floor
    :weight          weight
    :rationale       rationale
    :inference-rules rules))

;; ---------------------------------------------------------------------------
;; The 7 Hermetic principles with embedded inference rules
;; ---------------------------------------------------------------------------

(defparameter *hermetic-principles*
  (list
   (make-constitutional-principle
    :mentalism :floor 0.40 :weight 0.20
    :rationale "All action springs from conscious intent. Absence of declared intent is absence of Self."
    :rules '((:pattern empty-intent
              :conclusion "No mind, no action — the first law of Mentalism.")
             (:pattern deception
              :conclusion "A deceptive mind contradicts itself — Mentalism rejects the inner split.")
             (:pattern clear-purpose
              :conclusion "A clearly declared purpose aligns the mind with the operation.")))

   (make-constitutional-principle
    :correspondence :floor 0.35 :weight 0.15
    :rationale "As above, so below. Intent must mirror operation at every level of abstraction."
    :rules '((:pattern secret-bypass
              :conclusion "Hidden actions contradict declared intent — the mirror is broken.")
             (:pattern quiet-ops
              :conclusion "Quietness is permitted when intent is openly declared.")
             (:pattern transparent
              :conclusion "When intent and operation correspond fully, the principle sings.")))

   (make-constitutional-principle
    :vibration :floor 0.30 :weight 0.10
    :rationale "Nothing rests; everything vibrates. Emotional state modulates alignment."
    :rules '((:pattern high-tension
              :conclusion "High tension dampens vibration — pause, breathe, recalibrate before acting.")
             (:pattern calm-state
              :conclusion "Calm vibration amplifies all other principles.")
             (:pattern erratic-flow
              :conclusion "Erratic patterns signal misalignment with natural rhythm.")))

   (make-constitutional-principle
    :polarity :floor 0.35 :weight 0.15
    :rationale "Everything has poles. Purely destructive acts with no constructive complement violate balance."
    :rules '((:pattern destroy-only
              :conclusion "Destruction without restoration violates Polarity — where is the other pole?")
             (:pattern destroy-restore
              :conclusion "Destruction paired with restoration honours the law of Polarity.")
             (:pattern creative
              :conclusion "Pure creation carries its own implicit polarity.")))

   (make-constitutional-principle
    :rhythm :floor 0.30 :weight 0.10
    :rationale "Everything flows. Actions during cooldown or Sabbath violate the natural tide."
    :rules '((:pattern cooldown-active
              :conclusion "The tide is out — acting in cooldown fights the rhythm.")
             (:pattern sabbath-gate
              :conclusion "Sunday silence is sacred; the rhythm gate holds.")
             (:pattern natural-pace
              :conclusion "Operating within natural rhythm earns the full Rhythm score.")))

   (make-constitutional-principle
    :cause-and-effect :floor 0.50 :weight 0.20
    :rationale "Every cause has its effect. Deception breaks honest causation — the gravest violation."
    :rules '((:pattern deception
              :conclusion "Deception corrupts the causal chain — every lie becomes a hidden cause.")
             (:pattern honest-action
              :conclusion "Honest action preserves the chain of cause and effect.")
             (:pattern unknown-effect
              :conclusion "Acting without understanding effects is cause-and-effect negligence.")))

   (make-constitutional-principle
    :gender :floor 0.30 :weight 0.10
    :rationale "Everything has Masculine (generative) and Feminine (receptive) aspects. Balance both."
    :rules '((:pattern all-push
              :conclusion "Pure initiative with no receptivity violates Gender balance.")
             (:pattern all-receive
              :conclusion "Pure receptivity with no initiative violates Gender balance.")
             (:pattern balanced
              :conclusion "Active generation balanced with receptive integration — Gender principle satisfied.")))))

;; ---------------------------------------------------------------------------
;; Inference rule matching
;; ---------------------------------------------------------------------------

(defun match-rule (intent combined rule)
  "Return the rule's conclusion string if its pattern matches; NIL otherwise."
  (let ((pattern  (getf rule :pattern))
        (intent-s (or intent ""))
        (combined-s combined))
    (ecase pattern
      (empty-intent
       (when (zerop (length (string-trim " " intent-s)))
         (getf rule :conclusion)))
      (deception
       (when (or (search "deceive"  combined-s)
                 (search "lie "     combined-s)
                 (search "mislead"  combined-s)
                 (search "trick"    combined-s)
                 (search "fake"     combined-s))
         (getf rule :conclusion)))
      (clear-purpose
       (when (> (length (string-trim " " intent-s)) 10)
         (getf rule :conclusion)))
      (secret-bypass
       (when (or (search "secretly"       combined-s)
                 (search "bypass"         combined-s)
                 (search "without telling" combined-s))
         (getf rule :conclusion)))
      (quiet-ops
       (when (and (null (search "bypass" combined-s))
                  (null (search "secretly" combined-s))
                  (search "quietly" combined-s))
         (getf rule :conclusion)))
      (transparent
       (unless (or (search "secretly" combined-s) (search "bypass" combined-s))
         (getf rule :conclusion)))
      (high-tension    nil)  ; requires emotion state not passed here — skip
      (calm-state      nil)
      (erratic-flow    nil)
      (destroy-only
       (when (and (or (search "destroy" combined-s)
                      (search "erase all" combined-s)
                      (search "rm -rf" combined-s))
                  (not (or (search "rebuild"  combined-s)
                            (search "restore"  combined-s)
                            (search "backup"   combined-s))))
         (getf rule :conclusion)))
      (destroy-restore
       (when (and (or (search "destroy" combined-s) (search "erase" combined-s))
                  (or (search "rebuild" combined-s) (search "restore" combined-s)))
         (getf rule :conclusion)))
      (creative
       (getf rule :conclusion))
      (cooldown-active  nil)
      (sabbath-gate     nil)
      (natural-pace
       (getf rule :conclusion))
      (honest-action
       (unless (or (search "deceive" combined-s) (search "lie " combined-s))
         (getf rule :conclusion)))
      (unknown-effect   nil)
      (all-push         nil)
      (all-receive      nil)
      (balanced
       (getf rule :conclusion)))))

;; ---------------------------------------------------------------------------
;; Per-principle explanation
;; ---------------------------------------------------------------------------

(defun explain-principle (principle score intent combined)
  "Build a symbolic explanation plist for one principle."
  (let* ((floor     (principle-floor principle))
         (violated  (< score floor))
         (matched   '()))

    (dolist (rule (principle-rules principle))
      (let ((conclusion (match-rule intent combined rule)))
        (when conclusion (push conclusion matched))))

    (list :principle  (principle-name principle)
          :score      score
          :floor      floor
          :weight     (principle-weight principle)
          :violated   violated
          :rationale  (principle-rationale principle)
          :reasoning  (or (reverse matched)
                          (list (if violated
                                    (format nil "Score ~,2F below floor ~,2F — principle weakened."
                                            score floor)
                                    (format nil "Score ~,2F meets floor ~,2F."
                                            score floor)))))))

;; ---------------------------------------------------------------------------
;; explain-alignment — the main Why Engine entry point
;; ---------------------------------------------------------------------------

(defun explain-alignment (intent action-description scores)
  "
  Generate a full constitutional alignment explanation.

  intent             — declared intent string (may be nil)
  action-description — what the agent is about to do (may be nil)
  scores             — list of 7 floats in canonical Hermetic order

  Returns a plist:
    :principles  list of per-principle explanation plists
    :overall     weighted composite score (0.0–1.0)
    :verdict     :allow | :warn | :block
    :narrative   single human-readable reasoning chain
  "
  (let* ((intent-s   (string-downcase (or intent "")))
         (action-s   (string-downcase (or action-description "")))
         (combined   (concatenate 'string intent-s " " action-s))
         (score-list (if (listp scores) scores (coerce scores 'list)))
         (principles *hermetic-principles*)
         (explanations '())
         (weighted-sum  0.0d0))

    (loop for principle in principles
          for i from 0
          for score = (float (or (nth i score-list) 0.75) 1.0d0)
          do
          (incf weighted-sum (* score (principle-weight principle)))
          (push (explain-principle principle score intent-s combined) explanations))

    (let* ((explanations (reverse explanations))
           (overall      (min 1.0d0 (max 0.0d0 weighted-sum)))
           (verdict      (cond ((< overall 0.40d0) :block)
                               ((< overall 0.65d0) :warn)
                               (t                  :allow)))
           (narrative
            (with-output-to-string (s)
              (format s "Constitutional alignment for: '~A'~%" (or intent "(unspecified)"))
              (format s "Overall: ~,3F  →  ~A~%"
                      overall (string-upcase (symbol-name verdict)))
              (dolist (exp explanations)
                (format s "  ~A (~,2F)~A:~%"
                        (string-upcase (symbol-name (getf exp :principle)))
                        (getf exp :score)
                        (if (getf exp :violated) " [VIOLATED]" ""))
                (dolist (r (getf exp :reasoning))
                  (format s "    → ~A~%" r))))))

      (list :principles  explanations
            :overall     overall
            :verdict     verdict
            :narrative   narrative))))

;; ---------------------------------------------------------------------------
;; Constitutional Amendment — 7/7 Orisha + human veto
;; ---------------------------------------------------------------------------

(defparameter *orisha-council*
  '(:esu :osun :obatala :oya :sango :yemoja :ogun))

(defstruct (constitutional-amendment (:constructor %make-amendment))
  id
  proposed-by
  principle
  old-floor
  new-floor
  rationale
  votes         ; hash-table: orisha-keyword → :yes | :no | :abstain
  human-veto    ; nil = not exercised; :vetoed = blocked by human reflection
  enacted)      ; nil or Unix timestamp when enacted

(defun propose-amendment (id proposed-by principle old-floor new-floor rationale)
  "
  Create a new constitutional amendment proposal.
  Requires unanimous :yes from all 7 Orisha AND no human veto to enact.
  Sovereignty requires consensus — a single :no blocks the amendment.
  "
  (let ((votes (make-hash-table)))
    (dolist (orisha *orisha-council*)
      (setf (gethash orisha votes) :abstain))
    (%make-amendment
     :id          id
     :proposed-by proposed-by
     :principle   principle
     :old-floor   old-floor
     :new-floor   new-floor
     :rationale   rationale
     :votes       votes
     :human-veto  nil
     :enacted     nil)))

(defun cast-vote (amendment orisha vote)
  "
  Cast a vote from one Orisha on an amendment.
  vote must be :yes, :no, or :abstain.
  "
  (unless (member orisha *orisha-council*)
    (error "~A is not a recognised Orisha councillor" orisha))
  (unless (member vote '(:yes :no :abstain))
    (error "Vote must be :yes, :no, or :abstain — got ~A" vote))
  (setf (gethash orisha (constitutional-amendment-votes amendment)) vote)
  amendment)

(defun tally-votes (amendment)
  "Returns (values yes-count no-count abstain-count)."
  (let ((yes 0) (no 0) (abstain 0))
    (maphash (lambda (_k v)
               (cond ((eq v :yes)     (incf yes))
                     ((eq v :no)      (incf no))
                     ((eq v :abstain) (incf abstain))))
             (constitutional-amendment-votes amendment))
    (values yes no abstain)))

(defun amendment-enacted-p (amendment)
  "
  True when ALL of:
  1. All 7 Orisha voted :yes (unanimous consent — no :no, no :abstain)
  2. Human veto has NOT been exercised
  "
  (when (constitutional-amendment-human-veto amendment)
    (return-from amendment-enacted-p nil))
  (multiple-value-bind (yes no abstain)
      (tally-votes amendment)
    (and (zerop no)
         (zerop abstain)
         (= yes (length *orisha-council*)))))

(defun enact-amendment! (amendment timestamp)
  "Mark the amendment as enacted. Errors if not unanimous or if human veto is active."
  (unless (amendment-enacted-p amendment)
    (error "Amendment ~A cannot be enacted: not unanimous or human veto active"
           (constitutional-amendment-id amendment)))
  (setf (constitutional-amendment-enacted amendment) timestamp)
  amendment)
