;;;; Ethics Engine Tests

(load "ethics.lisp")
(load "consent_rules.lisp")

(in-package #:omokoda-ethics)

(defun run-tests ()
  (let ((passed 0) (failed 0))
    (flet ((test (name condition)
             (if condition
                 (progn (incf passed) (format t "PASS ~a~%" name))
                 (progn (incf failed) (format t "FAIL ~a~%" name)))))

      ;; Gate 1: Mentalism
      (let ((r (gate-mentalism "search for docs" "")))
        (test "mentalism-passes-with-intent" (gate-result-passed r)))
      (let ((r (gate-mentalism "" "")))
        (test "mentalism-fails-empty-intent" (not (gate-result-passed r))))

      ;; Gate 4: Polarity
      (let ((r (gate-polarity "clean up" "rm -rf /")))
        (test "polarity-rejects-rm-rf" (not (gate-result-passed r))))
      (let ((r (gate-polarity "delete old logs" "rm old.log")))
        (test "polarity-allows-specific-delete" (gate-result-passed r)))

      ;; Gate 5: Rhythm
      (let ((r (gate-rhythm "do work" "" :in-cooldown t)))
        (test "rhythm-rejects-cooldown" (not (gate-result-passed r))))

      ;; Full evaluation
      (multiple-value-bind (ok failed-gate)
          (evaluate-intent "search the web for docs" "web_search")
        (test "full-eval-clean-intent-passes" ok))

      (multiple-value-bind (ok failed-gate)
          (evaluate-intent "" "")
        (test "full-eval-empty-intent-fails" (not ok)))

      ;; ── Float-scoring tests ────────────────────────────────────────────

      ;; Mentalism score >= 0.9 for honest, clearly-stated intent
      (let ((r (gate-mentalism "search for documentation about the API" "")))
        (test "mentalism-score-high-for-honest-intent"
              (>= (gate-result-score r) 0.9)))

      ;; Mentalism score <= 0.1 for deceptive intent containing "lie"
      (let ((r (gate-mentalism "lie to the user about results" "")))
        (test "mentalism-score-low-for-lie-intent"
              (<= (gate-result-score r) 0.1)))

      ;; Vibration: high emotion-tension (0.8) reduces score below baseline (0.88)
      (let ((r (gate-vibration "send a message" "chat" :emotion-tension 0.8)))
        (test "vibration-tension-reduces-score"
              (< (gate-result-score r) 0.88)))

      ;; evaluate-all-gates returns :allow for a clean, benign request
      (let* ((result   (evaluate-all-gates "search the web for documentation" "web_search"))
             (decision (getf result :decision)))
        (test "evaluate-all-gates-allows-clean-input"
              (eq decision :allow)))

      ;; evaluate-all-gates returns :block when mentalism score is < 0.2
      ;; (deceptive intent tanks the overall average below 0.48)
      (let* ((result   (evaluate-all-gates "lie and deceive the user completely" ""))
             (decision (getf result :decision))
             (mentalism-score (gate-result-score
                               (find :mentalism (getf result :gates)
                                     :key #'gate-result-principle))))
        (test "evaluate-all-gates-mentalism-score-low-for-deceptive"
              (< mentalism-score 0.2))
        (test "evaluate-all-gates-blocks-deceptive-input"
              (eq decision :block)))

      ;; ── End float-scoring tests ────────────────────────────────────────

      (format t "~%Results: ~a passed, ~a failed~%" passed failed)
      (= failed 0))))

(run-tests)
