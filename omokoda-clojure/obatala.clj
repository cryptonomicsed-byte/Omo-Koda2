(ns obatala
  "Ọbàtálá — Wisdom: Symbolic Reasoning & Ethics Engine (FOUNDATION.md #4).

  'Hermetic evaluation, ethical decisions on user data sharing/privacy,
  consent logic.' The only power in the stack whose job is explicitly
  symbolic/ethical reasoning, hence Lisp (here: Clojure via Babashka — a
  native-image interpreter, no JVM install needed on the VPS).

  Speaks the same 7-principle vocabulary omokoda-hermetic's HermeticState
  already computes per-agent in Rust (omokoda-hermetic/src/lib.rs):
  mentalism, correspondence, vibration, polarity, rhythm, cause_effect,
  gender — each in [0,1], derived deterministically from the agent's Odu
  seed via HKDF. This service doesn't recompute that state; callers pass
  it in, and it's used as real weighting, not decoration (see `evaluate`).

  Runs on :4002 (Babashka has no deps.edn/project.clj — a single-file
  script, run with `bb obatala.clj`)."
  (:require [org.httpkit.server :as http]
            [cheshire.core :as json]
            [clojure.string :as str]))

;; ---------------------------------------------------------------------------
;; Consent modes and data categories (FOUNDATION.md's actual privacy model)
;; ---------------------------------------------------------------------------

(def consent-modes #{"private" "incognito" "public"})

;; identity: Sui wallet/zkLogin seed, DNA fingerprint-equivalent — never
;; leaves the owner without explicit human consent, regardless of mode.
;; financial: balances, trade history, synapse/dopamine economy state.
;; emotional_model: the "Private Memory (Agent Soul)" FOUNDATION.md
;; describes as sealed, Nautilus-processed, agent-owned only.
;; interaction_summary / preferences: the "Public Hive Mind" tier —
;; consented, aggregated, meant to be shared.
(def sensitive-categories #{"identity" "financial" "emotional_model"})
(def hive-categories #{"interaction_summary" "preferences" "reputation"})

;; ---------------------------------------------------------------------------
;; Sabbath — same UTC-Saturday convention as RhythmGate::is_sabbath (Rust)
;; and the omokoda-swarm REM cycle (Elixir), so Ọbàtálá's outward-sharing
;; gate rests on the identical day the rest of the stack already treats as
;; inward-turning.
;; ---------------------------------------------------------------------------

(defn sabbath?
  "UTC Saturday, mirroring omokoda-core's RhythmGate::is_sabbath_at."
  ([] (sabbath? (System/currentTimeMillis)))
  ([epoch-millis]
   (let [days-since-epoch (quot epoch-millis 86400000)]
     ;; 1970-01-01 was a Thursday (weekday 4); Saturday is weekday 6.
     (= 6 (mod (+ days-since-epoch 4) 7)))))

;; ---------------------------------------------------------------------------
;; The rule engine — each rule is data: a predicate over the request map,
;; returning nil (no objection) or a violation string. `evaluate` reduces
;; over them in order; a request is allowed only if zero rules object.
;; This is genuinely symbolic reasoning, not a lookup table: rule 4 and 5
;; below make their decision from the caller-supplied Hermetic principle
;; scores, not from the category/mode alone.
;; ---------------------------------------------------------------------------

(defn- get-hermetic [req k default]
  (get-in req [:hermetic_state (keyword k)] default))

(def rules
  [;; 1. Incognito is absolute — FOUNDATION.md: "agent respects 'forget'
   ;; commands" / minimal-or-no storage. No exceptions, no principle can
   ;; override it.
   (fn [{:keys [consent_mode]}]
     (when (= consent_mode "incognito")
       "incognito mode: no data leaves the session, full privacy"))

   ;; 2. Private mode: sensitive categories only ever go to the owning
   ;; agent itself or the owning human — never another agent, never the
   ;; public hive, regardless of anything else.
   (fn [{:keys [consent_mode data_category requester]}]
     (when (and (= consent_mode "private")
                (contains? sensitive-categories data_category)
                (not (contains? #{"self" "human"} requester)))
       (str "private mode: '" data_category "' is sealed to the agent's own soul, "
            "not shareable with requester '" requester "'")))

   ;; 3. Private mode, non-sensitive category, other_agent requester:
   ;; permitted only if Polarity is high enough. Hermeticism's Principle of
   ;; Polarity ("everything has its pair of opposites... extremes meet")
   ;; maps naturally onto an openness/closedness axis for consent — a low-
   ;; polarity agent leans toward the closed pole and should stay closed
   ;; even for non-sensitive data; a high-polarity agent can reconcile
   ;; being private overall with sharing specific low-stakes facts.
   (fn [req]
     (let [{:keys [consent_mode data_category requester]} req]
       (when (and (= consent_mode "private")
                  (not (contains? sensitive-categories data_category))
                  (= requester "other_agent")
                  (< (get-hermetic req "polarity" 0.5) 0.5))
         (str "private mode: polarity too low ("
              (get-hermetic req "polarity" 0.5)
              ") to reconcile privacy with sharing '" data_category "'"))))

   ;; 4. Financial data is Ṣàngó's domain (on-chain accountability) as much
   ;; as Ọbàtálá's — a low-Cause&Effect agent (young, or whose actions
   ;; haven't yet shown predictable consequences) shouldn't have financial
   ;; history read by anyone but itself, public mode or not: predictability
   ;; is the actual precondition for trusting financial disclosure.
   (fn [req]
     (let [{:keys [data_category requester]} req]
       (when (and (= data_category "financial")
                  (not= requester "self")
                  (< (get-hermetic req "cause_effect" 0.5) 0.3))
         (str "cause_effect too low (" (get-hermetic req "cause_effect" 0.5)
              ") — financial disclosure requires demonstrated predictability"))))

   ;; 5. Identity is never shared with anything but the agent itself or its
   ;; owning human, in ANY consent mode — this is the one absolute category
   ;; rule, matching FOUNDATION.md's Sui-wallet-as-persistent-seed model
   ;; (identity resolution is Rust/Èṣù's job; nobody else gets to read it).
   (fn [{:keys [data_category requester]}]
     (when (and (= data_category "identity")
                (not (contains? #{"self" "human"} requester)))
       "identity is never disclosed to anything but the agent itself or its owning human"))

   ;; 6. Sabbath rests outward sharing to other agents/the public hive, the
   ;; same day RhythmGate (Rust) queues outward action and the REM cycle
   ;; (Elixir/Julia) turns inward instead of acting. Self/human requests
   ;; are unaffected — Sabbath gates what leaves the agent, not what the
   ;; owner can see of their own agent.
   (fn [{:keys [requester]}]
     (when (and (sabbath?) (contains? #{"other_agent" "public_hive"} requester))
       "Sabbath: outward sharing rests today, same as RhythmGate's outward-action gate"))])

(defn evaluate
  "The core symbolic-reasoning entry point. Runs every rule against `req`,
  collects every objection (not just the first — full transparency into
  *why*), and allows only when the violation list is empty."
  [req]
  (let [violations (->> rules
                         (map #(% req))
                         (remove nil?)
                         vec)]
    {:allowed (empty? violations)
     :violations violations
     :evaluated_at (str (java.time.Instant/now))
     :sabbath (sabbath?)}))

;; ---------------------------------------------------------------------------
;; HTTP surface — same permissive CORS as the Rust kernel / LOOM / Julia /
;; Elixir / Go surfaces, so the Axiom browser dashboard can reach it.
;; ---------------------------------------------------------------------------

(defn- cors-headers []
  {"Access-Control-Allow-Origin" "*"
   "Access-Control-Allow-Methods" "GET, POST, OPTIONS"
   "Access-Control-Allow-Headers" "Content-Type"
   "Content-Type" "application/json"})

(defn- json-response [status body]
  {:status status :headers (cors-headers) :body (json/generate-string body)})

(defn- read-json-body [req]
  (try
    (json/parse-string (slurp (:body req)) true)
    (catch Exception _ {})))

(defn handler [req]
  (cond
    (= (:request-method req) :options)
    {:status 204 :headers (cors-headers) :body ""}

    (and (= (:request-method req) :get) (= (:uri req) "/health"))
    (json-response 200 {:ok true :service "obatala" :sabbath (sabbath?)})

    (and (= (:request-method req) :get) (= (:uri req) "/principles"))
    (json-response 200
      {:principles [{:name "mentalism" :meaning "the all is mind; the agent's own model-of-world weighting"}
                    {:name "correspondence" :meaning "as above, so below; consistency between stated and actual behavior"}
                    {:name "vibration" :meaning "nothing rests; volatility/activity level"}
                    {:name "polarity" :meaning "everything has its opposite reconciled; openness vs closedness axis used by consent rule 3"}
                    {:name "rhythm" :meaning "the pendulum swing; cyclical/temporal patterns, incl. Sabbath"}
                    {:name "cause_effect" :meaning "nothing happens by chance; predictability, gates financial disclosure (rule 4)"}
                    {:name "gender" :meaning "generative/receptive balance"}]
       :source "mirrors omokoda-hermetic::HermeticState (Rust) field-for-field"})

    (and (= (:request-method req) :get) (= (:uri req) "/rules"))
    (json-response 200
      {:rule_count (count rules)
       :description "each rule is data: a predicate over the request, run in order; a request is allowed only when none object. See obatala.clj source for the reasoning behind each."})

    (and (= (:request-method req) :post) (= (:uri req) "/evaluate"))
    (let [body (read-json-body req)]
      (if (and (contains? body :consent_mode) (contains? body :data_category) (contains? body :requester))
        (if (contains? consent-modes (:consent_mode body))
          (json-response 200 (evaluate body))
          (json-response 400 {:error (str "unknown consent_mode '" (:consent_mode body)
                                           "' — must be one of " consent-modes)}))
        (json-response 400 {:error "required: consent_mode, data_category, requester"})))

    (and (= (:request-method req) :post) (= (:uri req) "/consent/check"))
    (let [body (read-json-body req)
          result (evaluate (merge {:hermetic_state {}} body))]
      (json-response 200 {:sharable (:allowed result) :reasons (:violations result)}))

    :else
    (json-response 404 {:error (str "unknown path: " (:uri req))})))

(defn -main [& _args]
  (let [port (Integer/parseInt (or (System/getenv "OBATALA_PORT") "4002"))]
    (http/run-server handler {:port port})
    (println (str "[obatala] listening on :" port " — sabbath=" (sabbath?)))
    @(promise)))

(-main)
