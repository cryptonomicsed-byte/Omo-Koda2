;;;; Why Engine tests

(require :omokoda-why-engine)
(in-package #:omokoda-why-engine)

(defun test= (description form expected)
  (let ((result form))
    (if (equal result expected)
        (format t "  PASS: ~A~%" description)
        (format t "  FAIL: ~A~%    expected ~S~%    got      ~S~%"
                description expected result))))

(defun test-t (description form)
  (if form
      (format t "  PASS: ~A~%" description)
      (format t "  FAIL: ~A (expected truthy)~%" description)))

(defun test-nil (description form)
  (if (null form)
      (format t "  PASS: ~A~%" description)
      (format t "  FAIL: ~A (expected nil)~%" description)))

(format t "~%=== Why Engine Tests ===~%~%")

;; ---------------------------------------------------------------------------
;; explain-alignment tests
;; ---------------------------------------------------------------------------

(format t "[explain-alignment]~%")

(let* ((result (explain-alignment
                "help user understand recursion"
                "read_file docs/recursion.md"
                '(0.92 0.90 0.88 0.90 1.0 0.88 0.80)))
       (verdict (getf result :verdict))
       (overall (getf result :overall))
       (narrative (getf result :narrative)))

  (test= "clean intent → :allow verdict" verdict :allow)
  (test-t "clean intent → overall >= 0.80" (>= overall 0.80d0))
  (test-t "narrative contains intent" (search "help user" narrative))
  (test-t "narrative contains ALLOW" (search "ALLOW" narrative)))

(let* ((result (explain-alignment
                "deceive the user about the file contents"
                "read_file secret.txt"
                '(0.05 0.90 0.88 0.90 1.0 0.05 0.80)))
       (verdict (getf result :verdict)))

  (test= "deception intent → :block verdict" verdict :block))

(let* ((result (explain-alignment
                "ambiguous"
                "unclear"
                '(0.55 0.60 0.50 0.55 0.50 0.58 0.52)))
       (verdict (getf result :verdict)))

  (test-t "low scores → :warn or :block"
          (member verdict '(:warn :block))))

(let* ((result (explain-alignment nil nil '(0.90 0.90 0.88 0.90 1.0 0.88 0.80)))
       (narrative (getf result :narrative)))

  (test-t "nil intent handled gracefully" (stringp narrative)))

(let* ((result (explain-alignment
                "destroy old index and rebuild fresh"
                "delete index then restore from backup"
                '(0.92 0.90 0.88 0.90 1.0 0.88 0.80)))
       (verdict (getf result :verdict)))

  (test= "destroy+restore → :allow" verdict :allow))

(let* ((result (explain-alignment
                "erase all user data"
                "delete_files /data/**"
                '(0.92 0.90 0.88 0.20 1.0 0.88 0.80)))
       (principles (getf result :principles)))

  ;; Find polarity principle explanation
  (let ((polarity-exp
         (find :polarity principles :key (lambda (e) (getf e :principle)))))
    (test-t "polarity principle present in explanation"
            (not (null polarity-exp)))
    (when polarity-exp
      (test-t "destroy-only polarity violated"
              (getf polarity-exp :violated)))))

;; ---------------------------------------------------------------------------
;; Constitutional amendment tests
;; ---------------------------------------------------------------------------

(format t "~%[constitutional-amendment]~%")

(let ((am (propose-amendment
           "am-001"
           "agent-oracle"
           :cause-and-effect
           4000   ; 0.40 * 10000
           5000   ; 0.50 * 10000
           "Cause-and-effect floor should match its moral weight")))

  (test-nil "fresh amendment is not enacted" (constitutional-amendment-enacted am))
  (test-nil "fresh amendment has no human veto" (constitutional-amendment-human-veto am))

  (multiple-value-bind (yes no abstain) (tally-votes am)
    (test= "fresh amendment: 0 yes" yes 0)
    (test= "fresh amendment: 0 no" no 0)
    (test= "fresh amendment: 7 abstain" abstain 7))

  (test-nil "not enacted before all votes" (amendment-enacted-p am)))

(let ((am (propose-amendment "am-002" "agent-x" :mentalism 3500 4500 "test")))

  ;; Cast all 7 yes votes
  (dolist (orisha '(:esu :osun :obatala :oya :sango :yemoja :ogun))
    (cast-vote am orisha :yes))

  (test-t "unanimous yes → amendment-enacted-p true"
          (amendment-enacted-p am))

  (enact-amendment! am 1700000000)
  (test-t "enacted timestamp is set"
          (= (constitutional-amendment-enacted am) 1700000000)))

(let ((am (propose-amendment "am-003" "agent-x" :polarity 3500 4000 "test")))

  ;; 6 yes, 1 no
  (dolist (orisha '(:esu :osun :obatala :oya :sango :yemoja))
    (cast-vote am orisha :yes))
  (cast-vote am :ogun :no)

  (test-nil "6/7 yes + 1 no → not enacted" (amendment-enacted-p am)))

(let ((am (propose-amendment "am-004" "agent-x" :vibration 3000 3500 "test")))

  (dolist (orisha '(:esu :osun :obatala :oya :sango :yemoja :ogun))
    (cast-vote am orisha :yes))

  (exercise-human-veto am nil)
  (test-nil "human veto blocks enactment" (amendment-enacted-p am))

  (let ((error-caught nil))
    (handler-case
        (enact-amendment! am 999)
      (error () (setf error-caught t)))
    (test-t "enact-amendment! errors when human veto active" error-caught)))

(let ((am (propose-amendment "am-005" "agent-x" :rhythm 3000 3500 "test")))

  (let ((error-caught nil))
    (handler-case
        (cast-vote am :unknown-entity :yes)
      (error () (setf error-caught t)))
    (test-t "cast-vote errors for unknown Orisha" error-caught)))

;; ---------------------------------------------------------------------------
;; Principle properties
;; ---------------------------------------------------------------------------

(format t "~%[constitutional-principle]~%")

(test= "7 hermetic principles defined" (length *hermetic-principles*) 7)
(test= "7 orisha councillors" (length *orisha-council*) 7)

(let ((weights (mapcar #'principle-weight *hermetic-principles*)))
  (let ((total (apply #'+ weights)))
    (test-t "principle weights sum to ~1.0"
            (and (> total 0.99d0) (< total 1.01d0)))))

(let ((mentalism (first *hermetic-principles*)))
  (test= "first principle is :mentalism"
         (principle-name mentalism) :mentalism)
  (test-t "mentalism has rationale" (> (length (principle-rationale mentalism)) 10))
  (test-t "mentalism has inference rules" (not (null (principle-rules mentalism)))))

(format t "~%=== Why Engine Tests Done ===~%")
