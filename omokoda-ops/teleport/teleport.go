package teleport

import (
	"archive/tar"
	"compress/gzip"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sync"
	"time"
)

// Status tracks the lifecycle of a teleport operation.
type Status string

const (
	StatusIdle         Status = "idle"
	StatusBundling     Status = "bundling"
	StatusTransferring Status = "transferring"
	StatusApplying     Status = "applying"
	StatusDone         Status = "done"
	StatusFailed       Status = "failed"
)

// Manifest describes the contents and origin of a teleport bundle.
type Manifest struct {
	AgentID      string    `json:"agent_id"`
	Version      string    `json:"version"`
	CreatedAt    time.Time `json:"created_at"`
	SourceNode   string    `json:"source_node"`
	BundleSize   int64     `json:"bundle_size_bytes"`
	SessionFiles []string  `json:"session_files"`
	MemoryFiles  []string  `json:"memory_files"`
	Checksum     string    `json:"checksum"` // SHA-256 hex of the bundle data
}

// Bundle is an in-memory teleport bundle ready for transfer.
type Bundle struct {
	Manifest Manifest
	Data     []byte // gzipped tar
}

// Progress reports teleport operation progress.
type Progress struct {
	Status  Status `json:"status"`
	Percent int    `json:"percent"`
	Message string `json:"message"`
	Error   string `json:"error,omitempty"`
}

// Teleport orchestrates agent state migration between nodes.
type Teleport struct {
	sourceNode string
	mu         sync.Mutex
	status     Status
	progress   chan Progress
}

func New(sourceNode string) *Teleport {
	return &Teleport{
		sourceNode: sourceNode,
		status:     StatusIdle,
		progress:   make(chan Progress, 32),
	}
}

// Progress returns the channel that receives progress events.
func (t *Teleport) Progress() <-chan Progress { return t.progress }

// CurrentStatus returns the current teleport status.
func (t *Teleport) CurrentStatus() Status {
	t.mu.Lock()
	defer t.mu.Unlock()
	return t.status
}

func (t *Teleport) emit(p Progress) {
	t.mu.Lock()
	t.status = p.Status
	t.mu.Unlock()
	select {
	case t.progress <- p:
	default:
	}
}

// CreateBundle builds a gzipped tar bundle of agent state from statePath.
// It scans for session/*.json and memory/*.json files.
func (t *Teleport) CreateBundle(agentID, statePath string) (*Bundle, error) {
	t.emit(Progress{Status: StatusBundling, Percent: 0, Message: "scanning agent state"})

	var sessionFiles, memoryFiles []string

	// Collect session files
	sesDir := filepath.Join(statePath, "session")
	if entries, err := os.ReadDir(sesDir); err == nil {
		for _, e := range entries {
			if !e.IsDir() && filepath.Ext(e.Name()) == ".json" {
				sessionFiles = append(sessionFiles, filepath.Join(sesDir, e.Name()))
			}
		}
	}

	// Collect memory files
	memDir := filepath.Join(statePath, "memory")
	if entries, err := os.ReadDir(memDir); err == nil {
		for _, e := range entries {
			if !e.IsDir() && filepath.Ext(e.Name()) == ".json" {
				memoryFiles = append(memoryFiles, filepath.Join(memDir, e.Name()))
			}
		}
	}

	allFiles := append(sessionFiles, memoryFiles...)

	t.emit(Progress{Status: StatusBundling, Percent: 10,
		Message: fmt.Sprintf("bundling %d files", len(allFiles))})

	data, err := createTarGz(allFiles, statePath)
	if err != nil {
		t.emit(Progress{Status: StatusFailed, Error: err.Error()})
		return nil, err
	}

	checksum := sha256Hex(data)

	// Relative paths for manifest
	relSession := make([]string, len(sessionFiles))
	for i, f := range sessionFiles {
		rel, _ := filepath.Rel(statePath, f)
		relSession[i] = rel
	}
	relMemory := make([]string, len(memoryFiles))
	for i, f := range memoryFiles {
		rel, _ := filepath.Rel(statePath, f)
		relMemory[i] = rel
	}

	manifest := Manifest{
		AgentID:      agentID,
		Version:      "1",
		CreatedAt:    time.Now(),
		SourceNode:   t.sourceNode,
		BundleSize:   int64(len(data)),
		SessionFiles: relSession,
		MemoryFiles:  relMemory,
		Checksum:     checksum,
	}

	t.emit(Progress{Status: StatusBundling, Percent: 50,
		Message: fmt.Sprintf("bundle ready (%d bytes)", len(data))})

	return &Bundle{Manifest: manifest, Data: data}, nil
}

// Transfer simulates sending the bundle to a target node via HTTP POST.
// In production this posts to targetNodeURL/teleport/apply.
func (t *Teleport) Transfer(bundle *Bundle, targetNodeURL string) error {
	t.emit(Progress{Status: StatusTransferring, Percent: 60,
		Message: fmt.Sprintf("transferring %d bytes to %s", len(bundle.Data), targetNodeURL)})

	if targetNodeURL == "" {
		err := fmt.Errorf("targetNodeURL must not be empty")
		t.emit(Progress{Status: StatusFailed, Error: err.Error()})
		return err
	}

	// In a real implementation: http.Post(targetNodeURL+"/teleport/apply", ...)
	// Here we validate and simulate success.
	if !bundle.Verify() {
		err := fmt.Errorf("bundle checksum mismatch — data corrupted during transfer")
		t.emit(Progress{Status: StatusFailed, Error: err.Error()})
		return err
	}

	t.emit(Progress{Status: StatusTransferring, Percent: 80, Message: "transfer complete"})
	return nil
}

// Apply extracts the bundle contents into targetPath.
func (t *Teleport) Apply(bundle *Bundle, targetPath string) error {
	t.emit(Progress{Status: StatusApplying, Percent: 85, Message: "applying bundle"})

	if !bundle.Verify() {
		err := fmt.Errorf("bundle checksum mismatch — refusing to apply")
		t.emit(Progress{Status: StatusFailed, Error: err.Error()})
		return err
	}

	if err := extractTarGz(bundle.Data, targetPath); err != nil {
		t.emit(Progress{Status: StatusFailed, Error: err.Error()})
		return err
	}

	t.emit(Progress{Status: StatusDone, Percent: 100, Message: "teleport complete"})
	return nil
}

// Teleport runs the full pipeline: bundle -> transfer -> apply.
func (t *Teleport) Teleport(agentID, statePath, targetNodeURL, targetPath string) error {
	bundle, err := t.CreateBundle(agentID, statePath)
	if err != nil {
		return err
	}
	if err := t.Transfer(bundle, targetNodeURL); err != nil {
		return err
	}
	return t.Apply(bundle, targetPath)
}

// Verify checks the bundle's SHA-256 checksum.
func (b *Bundle) Verify() bool {
	return sha256Hex(b.Data) == b.Manifest.Checksum
}

// MarshalManifest returns the JSON-encoded manifest.
func (b *Bundle) MarshalManifest() ([]byte, error) {
	return json.Marshal(b.Manifest)
}

// -- helpers ------------------------------------------------------------------

func sha256Hex(data []byte) string {
	h := sha256.Sum256(data)
	return hex.EncodeToString(h[:])
}

func createTarGz(files []string, baseDir string) ([]byte, error) {
	pr, pw := io.Pipe()

	go func() {
		gz := gzip.NewWriter(pw)
		tw := tar.NewWriter(gz)

		for _, path := range files {
			info, err := os.Stat(path)
			if err != nil {
				continue
			}
			rel, err := filepath.Rel(baseDir, path)
			if err != nil {
				rel = filepath.Base(path)
			}
			hdr := &tar.Header{
				Name:    rel,
				Size:    info.Size(),
				Mode:    int64(info.Mode()),
				ModTime: info.ModTime(),
			}
			if err := tw.WriteHeader(hdr); err != nil {
				continue
			}
			f, err := os.Open(path)
			if err != nil {
				continue
			}
			io.Copy(tw, f) //nolint:errcheck
			f.Close()
		}

		tw.Close() //nolint:errcheck
		gz.Close() //nolint:errcheck
		pw.Close()
	}()

	buf, err := io.ReadAll(pr)
	if err != nil {
		return nil, err
	}
	return buf, nil
}

func extractTarGz(data []byte, destDir string) error {
	if err := os.MkdirAll(destDir, 0o755); err != nil {
		return err
	}

	gr, err := gzip.NewReader(
		&byteReader{data: data},
	)
	if err != nil {
		// Empty bundle (no real files) is OK for testing
		if len(data) == 0 {
			return nil
		}
		return err
	}
	defer gr.Close()

	tr := tar.NewReader(gr)
	for {
		hdr, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return err
		}
		target := filepath.Join(destDir, filepath.Clean(hdr.Name))
		if err := os.MkdirAll(filepath.Dir(target), 0o755); err != nil {
			return err
		}
		f, err := os.Create(target)
		if err != nil {
			return err
		}
		if _, err := io.Copy(f, tr); err != nil {
			f.Close()
			return err
		}
		f.Close()
	}
	return nil
}

// byteReader wraps []byte to implement io.Reader.
type byteReader struct {
	data []byte
	pos  int
}

func (r *byteReader) Read(p []byte) (int, error) {
	if r.pos >= len(r.data) {
		return 0, io.EOF
	}
	n := copy(p, r.data[r.pos:])
	r.pos += n
	return n, nil
}
