# Bulk-copy `fill_stream` (continues rust#159378)

**Change:** most of the flat-token regression came from `ArenaTokenStreamBuilder::fill_stream`, which rebuilt nested groups one token at a time. It now copies the arena range with one `extend_from_slice` and rebases the indices in a single pass.

**Checks:** UI suite: 21,995 passed, 0 failed.

**Perf:** local `instructions:u` vs base a8a1e6f.

| | #159378 | With this commit |
|---|---|---|
| Primary geomean | +0.10% | -0.08% |
| Secondary geomean | +0.02% | -0.14% |
| tt-muncher | +14.5% | +6.9% |

**Still regressed:** token-stream-stress (+6%) and the `check` runs of serde_derive and clap_derive (+0.9%). Upstream perf showed larger regressions than this local run, so the numbers need a perf.rust-lang.org run to confirm.
