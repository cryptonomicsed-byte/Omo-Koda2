package main

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"path/filepath"
	"strings"
	"testing"

	_ "github.com/mattn/go-sqlite3"
)

const (
	testMint    = "TESTMINT111111111111111111111111111111111111"
	testWallet  = "WalletAAA11111111111111111111111111111111111"
	testUnknown = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
)

// ecoTestBackend stands up fake council / picks / poolhealth servers and a
// seeded wallet_intel.db, pointing every config env var at them.
func ecoTestBackend(t *testing.T) (councilURL, picksURL, poolURL, dbPath string) {
	t.Helper()

	dbPath = filepath.Join(t.TempDir(), "wallet_intel.db")
	db, err := sql.Open("sqlite3", dbPath)
	if err != nil {
		t.Fatalf("open test db: %v", err)
	}
	defer db.Close()
	mustExec := func(q string) {
		t.Helper()
		if _, err := db.Exec(q); err != nil {
			t.Fatalf("seed db: %v", err)
		}
	}
	mustExec(`CREATE TABLE wallets(
		address TEXT PRIMARY KEY, tags TEXT, buys INTEGER DEFAULT 0, sells INTEGER DEFAULT 0,
		volume_usd REAL DEFAULT 0, distinct_tokens INTEGER DEFAULT 0, edge REAL,
		first_seen TEXT, last_seen TEXT, updated_at TEXT)`)
	mustExec(`CREATE TABLE token_stats(
		mint TEXT PRIMARY KEY, symbol TEXT, distinct_wallets INTEGER DEFAULT 0,
		buy_volume REAL DEFAULT 0, first_buy_ts TEXT, first_buyers TEXT, updated_at TEXT)`)
	mustExec(`CREATE TABLE wallet_tokens(
		wallet TEXT, mint TEXT, first_buy_ts TEXT, PRIMARY KEY(wallet, mint))`)
	mustExec(`CREATE TABLE seen(tx TEXT PRIMARY KEY, source TEXT, emitted_at TEXT)`)
	mustExec(`CREATE TABLE state(k TEXT PRIMARY KEY, v TEXT)`)
	mustExec(fmt.Sprintf(`INSERT INTO token_stats VALUES('%s','TEST',3,1234.5,
		'2026-08-19T00:00:00+00:00','["w1"]','2026-08-19T00:00:01+00:00')`, testMint))
	mustExec(fmt.Sprintf(`INSERT INTO wallets VALUES('%s','["smart","kol"]',5,2,900.0,3,0.5,
		'2026-08-18T00:00:00+00:00','2026-08-19T00:00:00+00:00','2026-08-19T00:00:00+00:00')`, testWallet))
	mustExec(fmt.Sprintf(`INSERT INTO wallet_tokens VALUES('%s','%s','2026-08-19T00:00:00+00:00')`,
		testWallet, testMint))
	mustExec(`INSERT INTO seen VALUES('tx1','radar','2026-08-19T00:00:00+00:00')`)

	council := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if got := r.Header.Get("X-Agent-Key"); got != "test-key" {
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}
		switch {
		case strings.HasSuffix(r.URL.Path, "/api/council/overview"):
			fmt.Fprint(w, `{"daemon_running":true,"verdict_count":93,"trace_buffer_pending":21}`)
		case strings.HasSuffix(r.URL.Path, "/api/council/verdicts"):
			fmt.Fprint(w, `[{"id":2,"symbol":"TEST","direction":"BUY","conviction":0.8,"outcome":"pending","votes":[]},`+
				`{"id":1,"symbol":"OTHER","direction":"SELL","conviction":0.6,"outcome":"pending","votes":[]}]`)
		case strings.HasSuffix(r.URL.Path, "/api/council/calibration"):
			fmt.Fprint(w, `[{"persona":"analyst","role":"Macro","correct":1,"total":2,"rate":0.5}]`)
		default:
			http.NotFound(w, r)
		}
	}))
	picks := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if !strings.HasSuffix(r.URL.Path, "/api/picks") {
			http.NotFound(w, r)
			return
		}
		fmt.Fprintf(w, `{"picks":[{"id":45,"ts":1787124715.41,"token_addr":%q,"symbol":"TEST",`+
			`"score":61.42,"rank":1,"components":{},"gates":{"passed":true,"vetoes":[]},`+
			`"entry_price":0.0001892,"status":"candidate"}],"count":1}`, testMint)
	}))
	pool := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if !strings.HasSuffix(r.URL.Path, "/api/poolhealth") {
			http.NotFound(w, r)
			return
		}
		fmt.Fprint(w, `{"gmgn":{"keys":6,"cooling":0,"ip_banned":false},"proxies":{"total":16,"cooldown":2,"live_estimate":14},"ts":1787125053.9}`)
	}))
	t.Cleanup(council.Close)
	t.Cleanup(picks.Close)
	t.Cleanup(pool.Close)

	t.Setenv("VANTAGE_URL", council.URL)
	t.Setenv("VANTAGE_KEY", "test-key")
	t.Setenv("PICKS_URL", picks.URL)
	t.Setenv("POOLHEALTH_URL", pool.URL)
	t.Setenv("WALLET_INTEL_DB", dbPath)
	return council.URL, picks.URL, pool.URL, dbPath
}

func ecoGet(t *testing.T, path string) (int, map[string]any) {
	t.Helper()
	req := httptest.NewRequest(http.MethodGet, path, nil)
	rec := httptest.NewRecorder()
	switch {
	case path == "/ecosystem":
		ecosystemHandler(rec, req)
	case strings.HasPrefix(path, "/v1/token/"):
		tokenIntakeHandler(rec, req)
	default:
		t.Fatalf("unhandled test path %s", path)
	}
	var body map[string]any
	if err := json.Unmarshal(rec.Body.Bytes(), &body); err != nil {
		t.Fatalf("decode response %s: %v", rec.Body.String(), err)
	}
	return rec.Code, body
}

func TestEcosystemHandler(t *testing.T) {
	ecoTestBackend(t)

	code, body := ecoGet(t, "/ecosystem")
	if code != http.StatusOK {
		t.Fatalf("expected 200, got %d", code)
	}
	if body["service"] != ecoSvcName {
		t.Errorf("service = %v", body["service"])
	}
	if body["ts"] == "" || body["ts"] == nil {
		t.Error("missing ts")
	}
	if _, ok := body["source_errors"]; ok {
		t.Errorf("unexpected source_errors: %v", body["source_errors"])
	}

	ov, ok := body["council_overview"].(map[string]any)
	if !ok || ov["verdict_count"].(float64) != 93 {
		t.Errorf("council_overview = %v", body["council_overview"])
	}
	verdicts, ok := body["council_verdicts"].([]any)
	if !ok || len(verdicts) != 2 {
		t.Errorf("council_verdicts = %v", body["council_verdicts"])
	}
	cal, ok := body["council_calibration"].([]any)
	if !ok || len(cal) != 1 {
		t.Errorf("council_calibration = %v", body["council_calibration"])
	}

	picks, ok := body["picks"].(map[string]any)
	if !ok || picks["count"].(float64) != 1 {
		t.Errorf("picks = %v", body["picks"])
	}
	ph, ok := body["pool_health"].(map[string]any)
	if !ok {
		t.Fatalf("pool_health = %v", body["pool_health"])
	}
	if ph["gmgn"].(map[string]any)["keys"].(float64) != 6 {
		t.Errorf("pool_health.gmgn.keys = %v", ph["gmgn"])
	}

	wi, ok := body["wallet_intel"].(map[string]any)
	if !ok {
		t.Fatalf("wallet_intel = %v", body["wallet_intel"])
	}
	stats := wi["stats"].(map[string]any)
	if stats["wallets"].(float64) != 1 || stats["tokens"].(float64) != 1 ||
		stats["wallet_token_links"].(float64) != 1 || stats["seen_txs"].(float64) != 1 {
		t.Errorf("wallet_intel.stats = %v", stats)
	}
}

func TestTokenIntakeHandler(t *testing.T) {
	ecoTestBackend(t)

	code, body := ecoGet(t, "/v1/token/"+testMint)
	if code != http.StatusOK {
		t.Fatalf("expected 200, got %d", code)
	}
	if body["found"] != true {
		t.Errorf("found = %v", body["found"])
	}
	q := body["query"].(map[string]any)
	if q["address"] != testMint || q["symbol"] != "TEST" {
		t.Errorf("query = %v", q)
	}

	pick, ok := body["picks_score"].(map[string]any)
	if !ok || pick["token_addr"] != testMint || pick["score"].(float64) != 61.42 {
		t.Errorf("picks_score = %v", body["picks_score"])
	}

	verdicts := body["council_verdicts"].([]any)
	if len(verdicts) != 1 {
		t.Fatalf("expected 1 council verdict (symbol filter), got %v", verdicts)
	}
	if verdicts[0].(map[string]any)["symbol"] != "TEST" {
		t.Errorf("verdict symbol = %v", verdicts[0])
	}

	wi := body["wallet_intel"].(map[string]any)
	tok := wi["token"].(map[string]any)
	if tok["symbol"] != "TEST" || tok["mint"] != testMint {
		t.Errorf("wallet_intel.token = %v", tok)
	}
	holders := wi["holders"].([]any)
	if len(holders) != 1 || holders[0].(map[string]any)["address"] != testWallet {
		t.Errorf("holders = %v", holders)
	}
	if _, ok := body["pool_health"].(map[string]any); !ok {
		t.Errorf("pool_health missing: %v", body)
	}
}

func TestTokenIntakeBySymbol(t *testing.T) {
	ecoTestBackend(t)

	// Address unknown to wallet_intel, but ?symbol= resolves the mint and
	// the council verdicts match the symbol.
	code, body := ecoGet(t, "/v1/token/"+testUnknown+"?symbol=test")
	if code != http.StatusOK {
		t.Fatalf("expected 200, got %d", code)
	}
	if body["found"] != true {
		t.Errorf("found = %v", body["found"])
	}
	q := body["query"].(map[string]any)
	if q["resolved_mint"] != testMint {
		t.Errorf("resolved_mint = %v", q)
	}
	verdicts := body["council_verdicts"].([]any)
	if len(verdicts) != 1 || verdicts[0].(map[string]any)["symbol"] != "TEST" {
		t.Errorf("verdicts = %v", verdicts)
	}
	if body["picks_score"] != nil {
		t.Errorf("picks_score should be null, got %v", body["picks_score"])
	}
}

func TestTokenIntakeUnknown(t *testing.T) {
	ecoTestBackend(t)

	code, body := ecoGet(t, "/v1/token/"+testUnknown)
	if code != http.StatusOK {
		t.Fatalf("expected 200, got %d", code)
	}
	if body["found"] != false {
		t.Errorf("found = %v", body["found"])
	}
	if body["picks_score"] != nil {
		t.Errorf("picks_score = %v", body["picks_score"])
	}
	if len(body["council_verdicts"].([]any)) != 0 {
		t.Errorf("council_verdicts = %v", body["council_verdicts"])
	}
	if body["wallet_intel"].(map[string]any)["token"] != nil {
		t.Errorf("wallet_intel.token = %v", body["wallet_intel"])
	}
}

func TestTokenIntakeInvalidAddress(t *testing.T) {
	ecoTestBackend(t)

	code, body := ecoGet(t, "/v1/token/xyz")
	if code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", code)
	}
	if body["error"] == "" {
		t.Error("expected error message")
	}
}

func TestEcosystemPartialFailure(t *testing.T) {
	ecoTestBackend(t)
	// Break one source: point poolhealth at a dead port.
	t.Setenv("POOLHEALTH_URL", "http://127.0.0.1:1")

	code, body := ecoGet(t, "/ecosystem")
	if code != http.StatusOK {
		t.Fatalf("expected 200 even with a failing source, got %d", code)
	}
	if body["pool_health"] != nil {
		t.Errorf("pool_health should be null, got %v", body["pool_health"])
	}
	errs, ok := body["source_errors"].(map[string]any)
	if !ok || errs["pool_health"] == nil {
		t.Errorf("source_errors = %v", body["source_errors"])
	}
	// The other sources must still be live.
	if body["picks"].(map[string]any)["count"].(float64) != 1 {
		t.Errorf("picks = %v", body["picks"])
	}
}

func TestEcoValidAddress(t *testing.T) {
	cases := map[string]bool{
		testMint:   true,
		testWallet: true,
		"xyz":      false,
		"":         false,
		"ok":       false,
		strings.Repeat("A", 55): false,
		strings.Repeat("B", 44): true,
		"ABCDE!@#$%^&*()":       false,
	}
	for addr, want := range cases {
		if got := ecoValidAddress(addr); got != want {
			t.Errorf("ecoValidAddress(%q) = %v, want %v", addr, got, want)
		}
	}
}
