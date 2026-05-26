;;;; Consent and Privacy Symbolic Logic

(defpackage #:omokoda-consent
  (:use #:cl)
  (:export #:check-consent #:privacy-mode-allows-p))

(in-package #:omokoda-consent)

(defparameter *consent-log* '())

(defun check-consent (agent-id tool privacy-mode)
  "Returns T if the agent may use this tool under the given privacy mode."
  (cond
    ((string= privacy-mode "public") t)
    ((string= privacy-mode "private")
     ;; Private: only local tools allowed
     (member tool '("read_file" "glob" "grep" "note_taking") :test #'string=))
    ((string= privacy-mode "incognito")
     ;; Incognito: same as private but no logging
     (member tool '("read_file" "glob" "grep" "note_taking") :test #'string=))
    (t nil)))

(defun privacy-mode-allows-p (privacy-mode provider)
  "Returns T if the privacy mode permits the given provider."
  (cond
    ((string= privacy-mode "public") t)
    ((or (string= privacy-mode "private") (string= privacy-mode "incognito"))
     (member provider '("webllm" "ollama" "local") :test #'string=))
    (t nil)))
