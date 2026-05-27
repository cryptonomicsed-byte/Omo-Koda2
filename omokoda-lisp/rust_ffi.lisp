;;;; CFFI bindings — exposes Lisp functions to Rust via C ABI

(defpackage #:omokoda-ffi
  (:use #:cl #:omokoda-ethics #:omokoda-consent))

(in-package #:omokoda-ffi)

#+sbcl
(defun export-evaluate-intent (intent-ptr intent-len params-ptr params-len result-ptr)
  "C-callable: evaluates intent through all 7 gates.
   result-ptr receives 1 (pass) or 0 (fail)."
  (let* ((intent (sb-alien:cast intent-ptr sb-alien:(* sb-alien:char)))
         (params (sb-alien:cast params-ptr sb-alien:(* sb-alien:char))))
    (multiple-value-bind (passed failed)
        (omokoda-ethics:evaluate-intent
         (sb-alien:alien-string intent intent-len)
         (sb-alien:alien-string params params-len))
      (setf (sb-alien:deref result-ptr) (if passed 1 0))
      (if passed 0 1))))

#+sbcl
(defun export-check-consent (agent-ptr tool-ptr mode-ptr result-ptr)
  "C-callable: checks consent for agent+tool+mode combination."
  (declare (ignore agent-ptr tool-ptr mode-ptr result-ptr))
  0) ; stub — full implementation uses SBCL alien stack
