package teleport

import (
	"os"
	"path/filepath"
	"testing"
)

func TestNewTeleport(t *testing.T) {
	tp := New("node-1")
	if tp.CurrentStatus() != StatusIdle {
		t.Errorf("expected idle, got %s", tp.CurrentStatus())
	}
}

func TestCreateBundle_EmptyDir(t *testing.T) {
	tp := New("node-1")
	dir := t.TempDir()

	bundle, err := tp.CreateBundle("agent-42", dir)
	if err != nil {
		t.Fatal(err)
	}
	if bundle.Manifest.AgentID != "agent-42" {
		t.Errorf("expected AgentID agent-42, got %s", bundle.Manifest.AgentID)
	}
	if bundle.Manifest.SourceNode != "node-1" {
		t.Errorf("expected SourceNode node-1, got %s", bundle.Manifest.SourceNode)
	}
	if bundle.Manifest.Checksum == "" {
		t.Error("expected non-empty checksum")
	}
}

func TestCreateBundle_WithFiles(t *testing.T) {
	tp := New("node-1")
	dir := t.TempDir()

	// Create session and memory dirs with JSON files
	sesDir := filepath.Join(dir, "session")
	memDir := filepath.Join(dir, "memory")
	os.MkdirAll(sesDir, 0o755)
	os.MkdirAll(memDir, 0o755)
	os.WriteFile(filepath.Join(sesDir, "sess-1.json"), []byte(`{"id":"s1"}`), 0o644)
	os.WriteFile(filepath.Join(memDir, "mem-1.json"), []byte(`{"key":"val"}`), 0o644)

	bundle, err := tp.CreateBundle("agent-1", dir)
	if err != nil {
		t.Fatal(err)
	}
	if len(bundle.Manifest.SessionFiles) != 1 {
		t.Errorf("expected 1 session file, got %d", len(bundle.Manifest.SessionFiles))
	}
	if len(bundle.Manifest.MemoryFiles) != 1 {
		t.Errorf("expected 1 memory file, got %d", len(bundle.Manifest.MemoryFiles))
	}
	if bundle.Manifest.BundleSize <= 0 {
		t.Error("expected positive bundle size")
	}
}

func TestBundleVerify(t *testing.T) {
	tp := New("n1")
	dir := t.TempDir()
	bundle, _ := tp.CreateBundle("a1", dir)

	if !bundle.Verify() {
		t.Error("freshly created bundle should verify")
	}

	// Corrupt the data
	if len(bundle.Data) > 0 {
		bundle.Data[0] ^= 0xFF
		if bundle.Verify() {
			t.Error("corrupted bundle should not verify")
		}
	}
}

func TestTransfer(t *testing.T) {
	tp := New("n1")
	dir := t.TempDir()
	bundle, _ := tp.CreateBundle("a1", dir)

	if err := tp.Transfer(bundle, "http://node-2:8080"); err != nil {
		t.Fatal(err)
	}
	if tp.CurrentStatus() != StatusTransferring {
		// status may have advanced; just check it's not failed
		if tp.CurrentStatus() == StatusFailed {
			t.Error("transfer should not have failed")
		}
	}
}

func TestTransferEmptyURL(t *testing.T) {
	tp := New("n1")
	dir := t.TempDir()
	bundle, _ := tp.CreateBundle("a1", dir)
	if err := tp.Transfer(bundle, ""); err == nil {
		t.Error("expected error for empty targetNodeURL")
	}
}

func TestApply(t *testing.T) {
	tp := New("n1")
	srcDir := t.TempDir()
	sesDir := filepath.Join(srcDir, "session")
	os.MkdirAll(sesDir, 0o755)
	os.WriteFile(filepath.Join(sesDir, "s1.json"), []byte(`{}`), 0o644)

	bundle, _ := tp.CreateBundle("a1", srcDir)

	destDir := t.TempDir()
	if err := tp.Apply(bundle, destDir); err != nil {
		t.Fatal(err)
	}
	if tp.CurrentStatus() != StatusDone {
		t.Errorf("expected done, got %s", tp.CurrentStatus())
	}
	// Verify the file was extracted
	extracted := filepath.Join(destDir, "session", "s1.json")
	if _, err := os.Stat(extracted); err != nil {
		t.Errorf("expected extracted file at %s: %v", extracted, err)
	}
}

func TestProgressChannel(t *testing.T) {
	tp := New("n1")
	dir := t.TempDir()

	go func() {
		tp.CreateBundle("a1", dir)
	}()

	count := 0
	timeout := make(chan struct{})
	go func() {
		<-timeout
	}()

	for i := 0; i < 3; i++ {
		select {
		case p := <-tp.Progress():
			if p.Status == "" {
				t.Error("progress status should not be empty")
			}
			count++
		}
	}
	// At least 1 progress event emitted
	if count == 0 {
		t.Error("expected at least one progress event")
	}
}

func TestMarshalManifest(t *testing.T) {
	tp := New("n1")
	dir := t.TempDir()
	bundle, _ := tp.CreateBundle("a1", dir)
	data, err := bundle.MarshalManifest()
	if err != nil {
		t.Fatal(err)
	}
	if len(data) == 0 {
		t.Error("marshaled manifest should not be empty")
	}
}
