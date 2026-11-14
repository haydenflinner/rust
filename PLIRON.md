# rustc with the pliron codegen backend

This fork vendors [rustc_codegen_pliron](https://github.com/haydenflinner/rustc_codegen_pliron)
into `compiler/rustc_codegen_pliron` — a rustc codegen backend with no LLVM:

```
rustc MIR → rustc_codegen_ssa → pliron LLVM dialect (+ pliron-IR passes)
→ Cranelift IR (+ CLIF passes) → cranelift-object (native) or waffle (wasm)
→ linker
```

Everything needed to build it is in this tree. The vendored copy is a
snapshot of the backend repo; `git log -- compiler/rustc_codegen_pliron`
shows the sync history.

## Build

```sh
git clone https://github.com/haydenflinner/rust
cd rust
cp bootstrap.pliron.toml bootstrap.toml   # then fix the paths, see below
./x build --stage 1 library
./x build --stage 2 compiler
```

With `codegen-backends = ["pliron"]` in `bootstrap.toml` (which the
example file sets), pliron is the *only* backend built — no LLVM is
compiled, and the resulting stage2 rustc emits through pliron by default:

```sh
build/<host>/stage2/bin/rustc -O hello.rs
PLIRON_VERSION=1 build/<host>/stage2/bin/rustc -vV   # which backend rev built this
```

## `bootstrap.pliron.toml` notes

- `[rust] codegen-backends = ["pliron"]` — build and select the pliron
  backend.
- `[build] full-bootstrap = true` — stage2 std is built by the
  pliron-compiled stage2 rustc rather than uplifted from stage1.
- `[llvm] download-ci-llvm = false` — no LLVM.
- `[target.*]` sections are machine-specific examples. Native Mach-O
  links fine with the system `ld` — on macOS just delete those sections.
  On Linux, point `linker` at `selfhost/cc-wild` inside
  `compiler/rustc_codegen_pliron/selfhost/` (or build
  `tools/pliron-wasm-ld` for the wasm target), and set `wasi-root` for
  wasm32-wasip1.

## Headstart (parallel metadata scheduling)

This fork also carries a port of
[PowderworksCode/headstart](https://github.com/PowderworksCode/headstart):
`-Zearly-metadata` makes this rustc write an `*.early-rmeta` once a
crate's interfaces are checked, and `-Zheadstart` (in cargo) starts
dependent crates on that early metadata instead of waiting for the full
rlib. It's orthogonal to the pliron backend — it cuts dependency-chain
idle time in the rustc front half, which the backend doesn't touch.

Note the submodule: `src/tools/cargo` points at `haydenflinner/cargo`'s
headstart branch, not `rust-lang/cargo` — clone with `--recursive` (or
`git submodule update --init`) to get it. Upstream headstart carries no
license, so this port is a personal-fork experiment, not something to
upstream or redistribute; see
`compiler/rustc_codegen_pliron/docs/headstart.md` for the evaluation and
`harness/headstart.py` for the scheduling simulator.

## Using the backend without building rustc

The backend also builds standalone against a pinned nightly
(`nightly-2026-10-22`, needs `rustc-dev`, `rust-src`, `llvm-tools-preview`)
as a dylib you can hand any compatible rustc:

```sh
cd compiler/rustc_codegen_pliron
cargo build            # target/debug/librustc_codegen_pliron.{dylib,so}
rustc -O -Zcodegen-backend=$PWD/target/debug/librustc_codegen_pliron.dylib file.rs
```

## Layout

| path | what |
|---|---|
| `compiler/rustc_codegen_pliron/` | the backend, vendored from `haydenflinner/rustc_codegen_pliron` (own workspace: `pliron-cfg`, `pliron-clif-*`, `pliron-ir-opt`, `vendor/` patched Cranelift/regalloc2/waffle/rsasm, `harness/` gate+bisect tools, `selfhost/` scripts, `docs/`) |
| `compiler/rustc_codegen_pliron_static/` | stub that recompiles `src/lib.rs` statically — for wasm-hosted rustc, which can't load dylib backends |
| `compiler/pliron_wasm_ld/` | stub exposing `tools/pliron-wasm-ld` so `rustc_codegen_ssa` can link wasm objects in-process (wasm-hosted rustc can't spawn a linker) |
| `src/bootstrap/.../compile.rs` | `PlironCodegenBackend` step (`cargo build -p rustc_codegen_pliron`); skipped for wasm hosts, which use the static crate instead |

## Developing on the backend

Work happens in the standalone repo
(`haydenflinner/rustc_codegen_pliron`), which has the fast loops
(`cargo test -p pliron-clif-opt`, `harness/quick.py`, `harness/gate.py` —
see its `AGENTS.md` and `docs/PARITY.md`). Two ways to try it inside
rustc:

- **Selfhost worktree** (throwaway, pinned nightly): from the backend
  repo, `selfhost/setup.sh [RUST_CHECKOUT]` creates `<RUST>-sh`, applies
  `bootstrap.patch` + `rustc-session.patch`, drops in
  `selfhost/bootstrap.toml`, and rsyncs the backend in.
  `selfhost/verify.sh` then runs rustc's UI suite under a pliron-built
  stage2.
- **Committed vendor** (this branch): rsync the repo into
  `compiler/rustc_codegen_pliron` — the same excludes `setup.sh` uses —
  keep the two stub `Cargo.toml`s and the root `[patch.crates-io]` table
  in sync with the backend's dep set, regen `Cargo.lock`
  (`cargo metadata`), and commit as `Sync rustc_codegen_pliron <rev>`.

`PLIRON_VERSION=1 <rustc> -vV` prints the source revision a backend
dylib was built from — check it before trusting a self-host or bisect
result.
