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

      (format t "~%Results: ~a passed, ~a failed~%" passed failed)
      (= failed 0))))

(run-tests)
