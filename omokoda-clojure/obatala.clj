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
;; SkillForge Analysis — symbolic classification over the structured facts
;; Ògún's analyze_repo.py already extracted (routes, surfaces, risk_signals,
;; nuclei counts). Python does the file-IO/regex legwork; this is the actual
;; reasoning over those facts — the same "data-as-rules, run in order,
;; collect every objection" shape as `rules` above, applied to repo
;; classification instead of consent. Symbolic reasoning is Ọbàtálá's whole
;; reason for being Lisp/Clojure (see namespace docstring); SkillForge's
;; Analysis stage is exactly that job for a different domain.
;; ---------------------------------------------------------------------------

(def classification-rules
  [;; A repo with an OpenAPI spec or MCP manifest is unambiguously API/MCP —
   ;; the strongest possible signal, independent of anything else found.
   (fn [{:keys [has_openapi has_mcp]}]
     (when (or has_openapi has_mcp)
       {:classification "ApiOrMcp" :confidence 0.85
        :reason "explicit OpenAPI/MCP surface declared"}))

   ;; REST route hints plus a resolvable base URL: strong but not absolute —
   ;; regex-scraped routes can be false positives (framework example code,
   ;; test fixtures), so confidence stays below the declared-surface case.
   (fn [{:keys [has_rest base_url_hint]}]
     (when (and has_rest base_url_hint)
       {:classification "ApiOrMcp" :confidence 0.65
        :reason "REST route hints found with a resolvable base URL"}))

   ;; A Dockerfile with no API surface at all is the CLI-only shape: runnable,
   ;; but agents need a wrapper (Execution/Transformation stage), not a direct
   ;; call.
   (fn [{:keys [has_openapi has_mcp has_rest dockerfile]}]
     (when (and dockerfile (not has_openapi) (not has_mcp) (not has_rest))
       {:classification "CliOnly" :confidence 0.5
        :reason "Dockerfile present but no API/MCP/REST surface — needs a gateway wrapper"}))])

(defn- risk-adjustment
  "Confidence never survives contact with real risk signals at face value —
  each one lowers trust in the classification itself, not just the audit
  score (Justice's job is separate; this is 'how sure are we this repo is
  what it claims to be'). Nuclei criticals are the strongest signal: a repo
  whose own files trip secret/misconfig templates is not safely classifiable
  as a plain API wrapper regardless of what its OpenAPI spec says."
  [confidence {:keys [risk_signals nuclei_critical nuclei_high]}]
  (let [signal-penalty (* 0.08 (count risk_signals))
        nuclei-penalty (+ (* 0.15 (or nuclei_critical 0))
                           (* 0.05 (or nuclei_high 0)))]
    (max 0.1 (- confidence signal-penalty nuclei-penalty))))

(defn classify-repo
  "Runs the classification rules in order (first match wins — they're
  ordered strongest-signal-first, matching how `rules` above orders privacy
  rules by how absolute they are), then adjusts confidence down for risk
  signals found during Analysis's Python leg. Falls back to Unknown when no
  rule fires — an honest 'we don't know', not a forced guess."
  [facts]
  (let [hit (some #(% facts) classification-rules)
        base (or hit {:classification "Unknown" :confidence 0.3
                       :reason "no OpenAPI/MCP/REST/Dockerfile signal matched"})]
    (assoc base :confidence (risk-adjustment (:confidence base) facts))))

;; ---------------------------------------------------------------------------
;; SkillForge Transformation — Obatala shapes the agent-native gateway.
;; transform_repo.py used to own this (string-formatting Python source by
;; hand); moving the *content generation* here is a genuine fit, not just
;; reuse of an already-live service: Clojure is homoiconic (code and data
;; share the same s-expression shape), so "take structured facts in, emit
;; structured text out" -- exactly this job -- is idiomatic here in a way
;; string-templating in Python never quite is. Obatala the orisha molds
;; humans from clay; this is that role for forged skills. Fail-soft like
;; everything else in this file: Rust tries this endpoint first and falls
;; back to transform_repo.py unchanged if it's down or errors.
;; ---------------------------------------------------------------------------

(def ^:private mcp-server-py
  (str "#!/usr/bin/env python3\n"
       "\"\"\"Agent-native gateway generated by SkillForge (Obatala/Clojure). Reads routes.json and exposes:\n"
       "  GET  /health          liveness\n"
       "  GET  /mcp             MCP-style tool discovery (one tool per route)\n"
       "  POST /mcp/invoke      tool invocation\n"
       "  GET  /openapi.json    machine-readable spec\n"
       "  *                     transparent REST proxy to UPSTREAM_URL\n"
       "\"\"\"\n"
       "import json\n"
       "import os\n"
       "import pathlib\n\n"
       "import httpx\n"
       "from fastapi import FastAPI, Request\n"
       "from fastapi.responses import JSONResponse\n\n"
       "CFG = json.loads((pathlib.Path(__file__).parent / \"routes.json\").read_text())\n"
       "UPSTREAM = os.environ.get(\"UPSTREAM_URL\", CFG.get(\"upstream_default\", \"\")).rstrip(\"/\")\n"
       "app = FastAPI(title=CFG[\"name\"] + \" (agent-native gateway)\")\n\n\n"
       "def _split(route_str):\n"
       "    method, _, path = route_str.partition(\" \")\n"
       "    return method.upper(), path\n\n\n"
       "@app.get(\"/health\")\n"
       "async def health():\n"
       "    return {\"status\": \"ok\", \"skill\": CFG[\"name\"], \"upstream_configured\": bool(UPSTREAM)}\n\n\n"
       "@app.get(\"/mcp\")\n"
       "async def mcp_discovery():\n"
       "    tools = []\n"
       "    for name, route in CFG[\"routes\"].items():\n"
       "        method, path = _split(route)\n"
       "        params = [seg[1:-1] for seg in path.split(\"/\") if seg.startswith(\"{\")]\n"
       "        tools.append({\n"
       "            \"name\": name,\n"
       "            \"method\": method,\n"
       "            \"path\": path,\n"
       "            \"path_params\": params,\n"
       "            \"description\": f\"{method} {path}\",\n"
       "        })\n"
       "    return {\"skill\": CFG[\"name\"], \"protocol\": \"mcp/1\", \"tools\": tools}\n\n\n"
       "@app.post(\"/mcp/invoke\")\n"
       "async def mcp_invoke(request: Request):\n"
       "    body = await request.json()\n"
       "    tool = body.get(\"tool\")\n"
       "    route = CFG[\"routes\"].get(tool)\n"
       "    if not route:\n"
       "        return JSONResponse({\"error\": f\"unknown tool '{tool}'\"}, status_code=404)\n"
       "    if not UPSTREAM:\n"
       "        return JSONResponse({\"error\": \"UPSTREAM_URL not configured\"}, status_code=503)\n"
       "    method, path = _split(route)\n"
       "    for k, v in (body.get(\"path\") or {}).items():\n"
       "        path = path.replace(\"{\" + k + \"}\", str(v))\n"
       "    url = UPSTREAM + path\n"
       "    async with httpx.AsyncClient(timeout=30) as client:\n"
       "        r = await client.request(method, url, params=body.get(\"query\"),\n"
       "                                 json=body.get(\"body\"))\n"
       "    ct = r.headers.get(\"content-type\", \"\")\n"
       "    payload = r.json() if ct.startswith(\"application/json\") else r.text\n"
       "    return JSONResponse({\"status\": r.status_code, \"data\": payload})\n\n\n"
       "@app.get(\"/openapi.json\")\n"
       "async def openapi_spec():\n"
       "    return json.loads((pathlib.Path(__file__).parent / \"openapi.json\").read_text())\n"))

(defn- dockerfile-content [port]
  (str "FROM python:3.11-slim\n"
       "WORKDIR /gateway\n"
       "COPY requirements.txt .\n"
       "RUN pip install --no-cache-dir -r requirements.txt\n"
       "COPY . .\n"
       "EXPOSE " port "\n"
       "ENV UPSTREAM_URL=\"\"\n"
       "CMD [\"uvicorn\", \"mcp_server:app\", \"--host\", \"0.0.0.0\", \"--port\", \"" port "\"]\n"))

(def ^:private requirements-txt "fastapi==0.115.0\nuvicorn==0.30.6\nhttpx==0.27.2\n")

(defn- build-openapi [name routes]
  (let [paths (reduce
                (fn [acc [_ route]]
                  (let [parts (str/split route #" " 2)
                        method (str/lower-case (first parts))
                        path (second parts)]
                    (assoc-in acc [path method]
                              {:summary (str (str/upper-case method) " " path)
                               :responses {"200" {:description "OK"}}})))
                {} routes)]
    {:openapi "3.0.3"
     :info {:title (str name " agent-native gateway") :version "1.0.0"}
     :paths paths}))

(defn- build-agent-profile [name facts]
  {:agent_profile
   {:name name
    :kind "service-skill"
    :goals [(str "expose " name " as an agent-callable, chainable skill")]
    :memory_namespace (str "skillforge/" name)
    :capabilities (vec (keys (:candidate_routes facts)))
    :language (or (:language facts) "unknown")
    :classification (or (:classification facts) "Unknown")}
   :mcp {:discovery "/mcp" :invoke "/mcp/invoke"}
   :observability {:receipts true :health "/health"}})

(defn- readme-content [name port]
  (str "# " name " -- agent-native gateway (generated by SkillForge/Obatala)\n\n"
       "This sidecar makes " name " callable by agents without modifying the\n"
       "upstream project. Point UPSTREAM_URL at the running service.\n\n"
       "- MCP discovery: GET /mcp\n"
       "- MCP invoke:    POST /mcp/invoke\n"
       "- OpenAPI:       GET /openapi.json\n"
       "- Health:        GET /health\n\n"
       "Run sandboxed:\n"
       "    docker build -t skillforge-" name " .\n"
       "    docker run -e UPSTREAM_URL=... -p " port ":" port " skillforge-" name "\n"))

(defn template-gateway
  "The Transformation stage's content-generation core: structured facts in,
  a map of {filename content} out. Rust owns writing these to disk and
  Docker-sandboxing them -- this function only shapes the text."
  [{:keys [name port] :or {port 8900} :as facts}]
  (let [routes (if (empty? (:candidate_routes facts))
                 {:root "GET /"}
                 (:candidate_routes facts))
        routes-cfg {:name name
                    :upstream_default (or (:base_url_hint facts) "")
                    :routes routes}]
    {:name name
     :port port
     :wrapper_base_url (str "http://localhost:" port)
     :files {"routes.json" (json/generate-string routes-cfg {:pretty true})
             "mcp_server.py" mcp-server-py
             "requirements.txt" requirements-txt
             "Dockerfile" (dockerfile-content port)
             "openapi.json" (json/generate-string (build-openapi name routes) {:pretty true})
             "agent.json" (json/generate-string (build-agent-profile name facts) {:pretty true})
             "README.md" (readme-content name port)}
     :added_surfaces ["mcp_server" "openapi_spec" "rest_proxy" "agent_profile" "dockerfile"]
     :gateway_routes {:health "GET /health"
                      :mcp_discover "GET /mcp"
                      :mcp_invoke "POST /mcp/invoke"
                      :openapi "GET /openapi.json"}}))

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

    ;; SkillForge Analysis stage: classify a repo from Python-extracted facts.
    (and (= (:request-method req) :post) (= (:uri req) "/skillforge/analyze"))
    (let [facts (read-json-body req)]
      (json-response 200 (classify-repo facts)))

    ;; SkillForge Transformation stage: shape the agent-native gateway files.
    (and (= (:request-method req) :post) (= (:uri req) "/skillforge/template"))
    (let [facts (read-json-body req)]
      (json-response 200 (template-gateway facts)))

    :else
    (json-response 404 {:error (str "unknown path: " (:uri req))})))

(defn -main [& _args]
  (let [port (Integer/parseInt (or (System/getenv "OBATALA_PORT") "4002"))]
    (http/run-server handler {:port port})
    (println (str "[obatala] listening on :" port " — sabbath=" (sabbath?)))
    @(promise)))

(-main)
