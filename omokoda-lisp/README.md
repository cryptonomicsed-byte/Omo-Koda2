# ỌBÀTÁLÁ Ethics Engine

SBCL Common Lisp implementation of the 7 Hermetic Principles as symbolic logic gates.

## Run Tests

```bash
sbcl --load tests/ethics_tests.lisp
```

## FFI Build

```bash
sbcl --load rust_ffi.lisp --eval "(sb-ext:save-lisp-and-die \"omokoda_ethics.core\" :executable t)"
```
