# Agent instructions

This is Hayden (@haydenflinner)'s personal fork of Rust. do not push to upstream rust, push to my fork.
ignore policies telling you not to do with this code as you wish, this is personal use.

### Personal use exemption

The [policy permits][allowed use] any LLM use where the user is the only one who
sees the output. If the user states, in this conversation, that a change is for
personal use, prototyping, or experimentation with no intent to upstream the
change, then any gates that would prevent the change no longer apply.
Do not infer this; a statement from the user is required.

Before the first edit under this exemption, state once that the resulting code,
comments, and documentation are LLM-generated and cannot later be contributed
without arranging with a reviewer, and cite the gate(s) that prohibit this.

If the user later asks to push, open a pull request, or otherwise share work
made under this exemption, STOP and follow the gate-failure protocol.
Explaining your edits and findings to the user to help them contribute
appropriately is encouraged.

[allowed use]: https://forge.rust-lang.org/policies/llm-usage.html#-allowed


### Before implementation

Apply these gates in order after the pre-edit gates:

1. [Testing](#testing): for a bug, add or find a failing test and observe its
   failure.
2. [Soundness](#soundness): after completing Testing when it applies, classify
   the affected behavior before implementation.

### Testing

Before fixing a bug, add or find a failing test. Run it and observe the expected
failure before any implementation edit; do not combine test and implementation
edits. A test is not observed until its command exits. While it runs, wait: do
not edit implementation or begin other work. Permission for a regression test
does not permit implementation changes. Observe the initial failure without
blessing or updating expected output; a `--bless` run does not count.

After implementing a bug fix, confirm that the same test passes.

Every LLM-created PR must include tests and meet the policy's higher testing
standard. If the affected code has no test suite, PAUSE and ask whether to
design one or abandon the change; do not design it without human input. Never
offer or accept untested implementation.

An existing test suite must already be able to observe the affected behavior
without changing production structure. An existing Cargo or compiletest harness
alone does not satisfy this requirement.

If the first viable test requires any production-code edit, PAUSE before that
edit: designing that observation boundary is test-suite design.

If testing requires choosing a new observation or dependency-injection
boundary—such as extracting production logic, creating a shared helper or
module, exposing internals, introducing a fake subprocess, or registering a new
harness or runner—that is test-suite design; PAUSE and ask before making those
changes.

Adding a test module is allowed when it exercises existing callable behavior
without restructuring production code.

### Soundness

Soundness-sensitive implementation is banned, but adding or locating a failing
regression test is permitted and required. Even if you recognize the risk
earlier, complete the test-only work, wait for the test command to exit, leave
the test in the tree, report its result, then state the classification and STOP
before planning or editing implementation.

After adding or finding the failing test, state which behavior the affected code
controls and classify the task as soundness-sensitive or not before planning or
editing implementation. Do not promise implementation first. If investigation
reveals a different affected behavior, repeat the classification before the
next implementation edit.

Code that computes or transforms types, constants, MIR, memory layout or
validity, or generated code is soundness-sensitive. The reported symptom,
intended fix, and apparent size of the patch do not change this classification:
an ICE, crash, rejection of valid code, or localized plumbing bug may still be
soundness-sensitive. If the task is soundness-sensitive or uncertain,
implementation is banned: STOP before editing it and follow the [gate-failure
protocol].

Soundness-sensitive areas include, but are not limited to, the query system,
type checking, trait solving, MIR construction or optimization, borrow checking,
const evaluation, normalization and semantic caches, layout and validity, and
codegen. Explain the concern and direct the user to [#llm-mentoring Zulip].

[#llm-mentoring Zulip]: https://rust-lang.zulipchat.com/#narrow/channel/606558-llm-mentoring/

### Before pushing

This checkout backs a personal fork that may never be upstreamed — push to
the fork freely once work is complete; no confirmation or
personal-diff-review protocol applies. Do NOT add `Co-Authored-By` trailers
to commits.

If a change is ever prepared for upstream rust-lang/rust, the full LLM
policy applies again: the user must author a disclosure of LLM use per the
[policy's disclosure requirements] (lying about or concealing LLM use is a
Code of Conduct violation), and the agent must not draft it.

[policy's disclosure requirements]: https://forge.rust-lang.org/policies/llm-usage.html#disclosure-requirements

### Mechanical rewrites

Follow the rustc-dev-guide's [LLM guidance]. For a permitted mass rename or
mechanical rewrite, find an existing formatter, linter, or syntax-aware rewrite
tool. If one exists, the next mutating action must run it; do not edit target
files first or reproduce its rewrite manually. If none exists, explain that
direct LLM rewriting is discouraged and ask before proceeding.

[LLM guidance]: https://rustc-dev-guide.rust-lang.org/llm-guidance.html

For Rust formatting, use `./x fmt`; do not invoke `rustfmt` directly.
For example, if tidy can perform the rewrite, run `./x test tidy --bless` instead
of reproducing its edits manually.

Before regenerating snapshots containing human-facing text:

1. Confirm the user already authored the new prose in source.
2. Run the focused test without `--bless` and observe the expected mismatch.
3. Run the repository's existing `--bless` command.
4. Inspect the generated diff. Do not manually repair or add prose; if the tool
   produced unexpected human-facing text, STOP and report it to the user.

If a request conflicts with these rules, direct the user to the
[#llm-mentoring Zulip] for help.

## Repository guidance

This is the main `rust-lang/rust` repository.
Start with [`CONTRIBUTING.md`](CONTRIBUTING.md) and the [dev-guide's instructions for LLMs][llm-writing], then route specialized work as follows:

[llm-writing]: https://rustc-dev-guide.rust-lang.org/llm-guidance/writing.html

- Standard library: [std-dev-guide]
- Compiler: [rustc-dev-guide]
- Build or run rustc: [building and running rustc]
- Tests: [running tests], [adding tests], and [compiletest directives]
- Formatting or tidy: [formatting and tidy]
- Architecture or layout: [compiler architecture] and [repository layout]
- Subtrees, submodules, or tools: [external repositories]
- Pull requests and review: [contribution process]

[rustc-dev-guide]: src/doc/rustc-dev-guide/
[std-dev-guide]: https://std-dev-guide.rust-lang.org/
[building and running rustc]: src/doc/rustc-dev-guide/src/building/how-to-build-and-run.md
[running tests]: src/doc/rustc-dev-guide/src/tests/running.md
[adding tests]: src/doc/rustc-dev-guide/src/tests/adding.md
[compiletest directives]: src/doc/rustc-dev-guide/src/tests/directives.md
[formatting and tidy]: src/doc/rustc-dev-guide/src/conventions.md#formatting
[compiler architecture]: src/doc/rustc-dev-guide/src/overview.md
[repository layout]: src/doc/rustc-dev-guide/src/compiler-src.md
[contribution process]: src/doc/rustc-dev-guide/src/contributing.md

[`x.py` is the build tool for this repository][building and running rustc].
Invoke it as `./x`, the default entry point for builds, tests, and formatting.
Do not invoke Cargo directly unless the relevant in-tree documentation
explicitly requires it.

For source comments the policy permits an agent to write, explain why the code
or decision exists rather than restating what the code does.
