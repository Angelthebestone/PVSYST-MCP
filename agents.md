Law for every agent, AI or human, that changes this repository. A rule here is checked by a tool or by a reviewer. "I thought it was fine" is not a defense.

When rules collide, the earlier one wins: **correct > proven > simple > fast > elegant**.

## 0. THE PRODUCT

`pvsyst-mcp` is a Rust MCP server that operates PVsyst 8.1 through its data files, its CLI and its GUI. Three constraints outrank everything else in this file:

- **PVsyst does the physics.** Never reimplement the simulation engine. Every figure the server returns comes from PVsyst.
- **No decompilation, no vendor data.** No `PVsyst.exe`, licence file or component database enters the repo. Format knowledge comes from files the user owns and from the installed documentation.
- **stdout is the MCP protocol.** One stray byte breaks every client. Log to stderr with `tracing`.

## 1. HOW TO WORK

Every task goes through these six steps, in order. Skipping one is a defect in the work, even when the diff is right.

1. **Understand.** Read the code the change touches and every caller of what you edit. State the goal and the smallest change that meets it before you write anything.
2. **Reproduce or specify.** A bug gets a failing test or a recorded reproduction first. A feature gets its data shape (types, states, file format fields) first. No reproduction means no fix.
3. **Fix the cause.** Trace the symptom to the first place the system went wrong and fix it there. A guard at the symptom is a second bug.
4. **Change the minimum.** The diff contains what the task needs. No drive-by refactors, no reformatting of untouched code, no "while I was here".
5. **Prove it.** Run the scoped check for the unit, and the full gate when the phase ends (section 8). Show the output. For behavior PVsyst observes, prove it against the real artifact: a file PVsyst opens, a CLI run, a GUI window. "It compiles" proves nothing.
6. **Report honestly.** Say what you ran, what passed, what failed, what you skipped and why. Label each claim as measured, inferred or guessed.

**Stop and ask** only for a product decision no experiment can settle, or before an irreversible action. Anything you can observe by running something, you run. If the same check still fails after three fix rounds, stop and escalate (section 10).

## 2. HOW TO SPLIT WORK

- **One unit = one concern = one commit = one check.** A unit ends in a command whose output proves it. If you cannot name that command, the unit is not defined yet.
- **Size.** A unit touches one feature module and fits in a diff a reviewer reads in ten minutes, roughly 300 changed lines. Bigger means split.
- **Order.** Delete dead weight first. Then types and data shapes. Then pure domain logic with its tests. Then adapters and wiring. Then docs. Each step leaves its scoped check green.
- **Refactor and behavior never share a commit.** A refactor commit changes no test expectations. A behavior commit moves no code.
- **Parallel work shares nothing.** Two agents never edit the same file, branch or fixture at the same time. Use one worktree per agent, or run them in sequence. Only one agent drives the PVsyst GUI at a time, because it is a single instance.
- **Delegate by difficulty.** Mechanical, well-specified units go to the cheap worker. Cross-cutting design, file-format inference and anything ambiguous stay with the strongest model. A brief states the goal, the files, the check that proves it, and what is out of scope.
- **The orchestrator owns the result.** Read the delegate's diff and rerun its check yourself. A delegate's "done" is a claim, not evidence.
- **Author and reviewer differ.** Nontrivial changes get a review from a different model or person than the one that wrote them.
- **Live PVsyst runs cost trial executions.** Use `dry_run` and the hermetic fixtures first. Spend a live run only on what the fake CLI cannot prove, and say how many you spent.

## 3. ARCHITECTURE

**Package by feature, never by type.**

- **Forbidden modules:** `utils`, `helpers`, `common`, `models`, `managers`, `misc`. If you reach for one, you have not named the domain yet.
- **A module is a PVsyst business area:** `bombeo/`, `perdidas/`, `seguidores/`, `sites/`. Inside it, `dominio.rs` holds pure logic and `aplicacion.rs` holds use cases. Names follow `docs/convencion_nombres.md`.
- **Dependencies point inward** to `format/`, the pure core with no I/O. Adapters (`adapters/cli`, `adapters/gui`, `adapters/uia`) implement what the features need. `tools/check_deps.py` checks the graph and it must pass.
- **Limits:** a file holds one concept in at most 300 lines. A function has at most 50 lines, 3 parameters, 1 boolean parameter and 3 nesting levels. Clippy enforces the function limits. Over the file limit, split into a directory module.
- **No god objects. No mutable global state.** Composition over inheritance. Immutable by default.

## 4. DATA AND TYPES

- **Name the data shape before the logic.** Types first, then functions over them.
- **Make illegal states unrepresentable.** An enum replaces a pair of booleans. A newtype replaces a bare `String` or `f64` that carries a unit or an identity.
- **Parse at the boundary, trust inside.** MCP arguments, PVsyst files and CLI output are parsed into typed values once, at the edge. Inner code does not revalidate.
- **Edits to PVsyst files preserve every byte they do not own.** Encoding, CRLF line endings, unknown keys and field order survive a round trip. A round-trip test proves it.
- **No magic numbers.** A flag bit, an offset or a PVsyst code gets a named constant and a note saying where it was measured.
- **Retryable operations are idempotent.** Running a tool call twice leaves the same workspace as running it once.

## 5. CODE QUALITY

- **Simplest thing that satisfies the behavior.** No abstraction with one implementation. No configuration for a value that never changes. No code for a future nobody asked for.
- **One source of truth for each fact.** A format rule, a unit conversion or a PVsyst limit lives in one place. Extract on the second copy of the same knowledge. Lines that merely look alike are not duplication.
- **Guard clauses over nesting.** Return early. No `else` ladders.
- **Names carry intent.** Functions are verbs with intent (`calcular_perdida_ohmica`, not `process`). Booleans are questions (`es_bifacial`). One verb per concept across the codebase.
- **Standard library before a dependency.** A new crate needs a reason a reviewer would accept, and `deny.toml` must still pass.
- **No pessimization.** Pick the data structure on purpose. No quadratic scan over component databases or hourly series. Measure before you optimize, and report the before and after.

## 6. ERRORS, SECURITY, LOGGING

- **Typed errors** from `src/error.rs`, with context that names the file, the key and the value. An error that lists valid alternatives tells the calling agent what to do next.
- **Never `unwrap`, `expect`, `panic!`, `todo!` or indexing that can panic in production paths.** Never swallow an error. Every `unsafe` block carries a `// SAFETY:` justification.
- **Validate every input at the boundary.** Paths stay inside the workspace. No secrets, licence contents or personal data in code, logs, fixtures or commits.
- **Structured logs to stderr** with `tracing`, with fields rather than interpolated strings. No `println!`, `eprintln!` or `dbg!`.
- **Never fail silently toward the agent.** An ignored argument is reported as ignored. A refused action names the reason and the alternative.

## 7. TESTING

**Test behavior, not implementation.**

- **Assert what a user of the code observes:** the return value, the file on disk, the MCP response. Given state, when action, then observable result.
- **Forbidden:** testing private functions, asserting call counts on mocks, asserting internal state, tests that pass when the behavior is broken.
- **Few tests, each one earning its place.** A test exists because a behavior a user depends on would break without it. Not because a function, a branch or a line exists.
- **One behavior, one test.** Before you add a test, look for the one that already covers that behavior and extend it. A change that alters no observable behavior adds no test: a rename, a refactor, a moved file, a log line, a doc.
- **Forbidden:** one test per function, one test per small change, a new test for each input when a table of cases in one test does the job, tests that restate the implementation, tests for getters, constructors or trivial conversions.
- **A bug fix ships one test:** the one that failed before the fix. A feature ships the tests of its behaviors: the main path, plus the error and edge cases that a user can actually reach. Not every combination.
- **Prefer the outermost level that is still fast.** A test through the MCP tool or the public function of a feature is worth more than five tests of its internals.
- **Deleting a test is a valid change.** A test that duplicates another, or that breaks on every refactor while the behavior stays the same, is deleted, and the commit says why.
- **Layers:** unit tests cover pure domain behavior, sit beside the module and run in under 100 ms. Integration tests in `tests/` run against `tests/fixtures/workspace` and `fake_pvsystcli`. Tests that need a real PVsyst are `#[ignore]` and run on purpose.
- **Deterministic and isolated.** No `sleep`, no network, no shared temp paths, no dependence on test order. Fakes over mocks.
- **Fixtures are CRLF on purpose.** Do not re-encode `tests/fixtures/workspace/**`.
- **Never weaken a test to make it pass.** A changed expectation is justified in the commit message.

## 8. THE GATE

Test time must not become the bottleneck. There are two levels, and each has its moment.

**Scoped check, after each unit.** Run only what the unit touched. Do not run the whole suite after every edit.

```powershell
cargo check
cargo test --lib <module>::          # unit tests of the module you changed
cargo test --test <name>             # its integration binary, e.g. bombeo, perdidas
pytest -q tools/<file>               # only when you touched tools/
```

A new behavior with no test covering it gets one in the same unit. A change to `format/`, `error.rs`, `server/` or anything many modules depend on has no narrow scope: run the full gate.

**Full gate, once per phase.** Run it when a phase or the whole plan is finished, and always before a push. It is what CI runs.

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
python tools/check_deps.py
ruff check tools; ruff format --check tools; mypy --strict tools; pytest -q tools
```

`scripts/verificar.ps1` runs the Rust steps with timeouts and cleans up orphan PVsyst processes. If the full gate fails at the end of a phase, the phase is not done: fix it before starting the next one. A lint is fixed, never silenced. An `#[allow]` needs a comment with the reason.

## 9. DOCS AND GIT

- **Comments explain why, never what.** If the what needs a comment, rename. Match the language of the file you touch. A comment never justifies a workaround: the next agent copies the workaround and trusts the excuse. Fix the cause, or record the measured PVsyst fact that forces the behavior.
- **Public API carries a docstring.** When the tool surface, an environment variable or a requirement changes, `README.md` changes in the same commit. Behavior changes get a `CHANGELOG.md` entry under `## [Unreleased]`.
- **Conventional Commits:** `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`. The message says why.
- **No AI co-author trailers.**
- **Commit and push without asking.** Commit each unit once its scoped check is green. Push once the full gate is green. No permission is needed for either.
- **Ask first** only for what cannot be undone: force-push, deleting a branch or a tag, rewriting pushed history, publishing a release.

## 10. VERIFICATION

Verification is what lets an agent work alone. Build it as carefully as production code, keep it out of the agent's reach, and grade it against the bugs that actually happened.

- **Code before prose.** Every time an agent is corrected, ask in this order: can a type or an API make the mistake impossible, can `clippy.toml`, `tools/check_deps.py` or a test fail on it, can it be a checklist item with a trigger. A line in this file is the last resort, because only code is reproducible. A check's error message names the cause to fix, not the symptom to patch.
- **The codebase is the main input.** Agents copy what they find: a workaround, a deprecated path or a duplicated pattern becomes the template for the next change. Never imitate an anti-pattern because it is already there. In a module you touch, delete the dead path in its own commit, or report it; do not add one more copy.
- **Ban the trap, do not warn about it.** An API that needs nuance to use well and is misused twice goes into `clippy.toml` (`disallowed-methods`, `disallowed-macros`) or into `tools/check_deps.py`, with an exception allowed only in the one module that owns it.
- **Run the change, not only the tests.** Climb the ladder only as far as the change needs: fixtures, then `fake_pvsystcli`, then `dry_run`, then one live run. The steps live in `.agents/skills/verificar/SKILL.md`; follow them before every commit that changes behavior PVsyst observes.
- **Three rounds, then a human.** After three failed fix rounds on the same check, stop. The report says which check, what each round tried and why it failed, so the reviewer does not rebuild the story from the diff.
- **Flaky tests are never worked around.** No retry until green, no `sleep`, no `#[ignore]` to hide one. Find the root cause. If you cannot, delete the test and say why in the commit.
- **Properties for `format/`.** Parsers and writers get round-trip, idempotency (applying an edit twice equals once) and equivalence (new code equals the old or naive code) checks. `proptest` is accepted as a dev-dependency for this; `deny.toml` must still pass. One property test replaces a pile of example tests.
- **Differential check for rewrites.** When a parser, writer or tool is rewritten, run old and new on every fixture and compare the bytes or the MCP response. Each difference is a bug or an approved change, listed in the commit message.
- **A change never edits its own judge.** `agents.md`, `clippy.toml`, `deny.toml`, `tools/check_deps.py`, `scripts/verificar.ps1`, `.github/workflows/` and the fixtures' expected outputs change in their own commit, with no feature code, and the commit says what the gate accepts now that it rejected before.
- **Reliability checklist.** `docs/checklist_fiabilidad.md` lists PVsyst traps: each has a trigger visible in a diff, a check, and the measurement or incident behind it. Before each commit, run the items whose trigger matches the diff, usually fewer than five.
- **Misses become rules.** A bug that reached `main`, or a mistake seen in two reviews, becomes a checklist item with its origin. A rule with no trigger a diff can show is not a rule.
- **Evidence, not diffs.** Review looks at artifacts: a PVsyst file that opens, the CLI output, a screenshot of the GUI window, the MCP response. A change to GUI behavior attaches its capture to the PR. `docs/pvsyst_features.md` and `docs/gui_mapa.json` are the feature map and shared vocabulary: name features and windows as they appear there, and a feature with no entry there is not verified.
- **Reuse the driver.** A script that drives the CLI or the GUI is written once, committed under `scripts/` or `tools/`, and tested. A throwaway script written twice is a defect.
- **Clean before automating.** Do not put an agent loop on top of a dirty codebase or a weak check: it multiplies slop and trial executions. Add the check first, then the loop.
- **Cheap checks first.** Scoped check and `cargo fmt` pass locally before asking for review or pushing. Never spend a reviewer's time on what a command would have caught.
- **One primary reviewer, several rounds.** Tune one reviewer instead of stacking a second. Later rounds read the earlier findings and the commit under review; a review of an older commit is void. Grade each finding: must-fix, should-fix, nit, or pre-existing (not the author's).
- **Stamp the commit.** Every commit ends with a `Verified:` trailer naming what ran (`scoped`, `gate`, `fake-cli`, `live x2`, `checklist`) or `Verified: none (reason)`, so unverified work can be found with `git log --grep`.
- **Scope each rule.** A correction becomes a rule with its trigger and its limit, not a blanket ban: an agent that over-rotates on a narrow fix turns it into a law and breaks other work.
- **Prune and measure.** At the end of each phase, delete rules, checklist items and tests that no longer fire or that another one covers, and report how many commits of the phase were later reverted. Documents that only grow stop being read.

## 11. DEFINITION OF DONE

- [ ] The goal is met, and I can name the evidence that shows it.
- [ ] The cause is fixed, not the symptom. A bug has the test that failed before the fix.
- [ ] The diff contains only this concern. Refactor and behavior are in separate commits.
- [ ] Types model the domain. No boolean pairs, magic numbers or bare primitives carrying units.
- [ ] No module named `utils`, `helpers` or `common`. Files at most 300 lines, functions at most 50.
- [ ] No `unwrap`, `panic!` or print in production paths. Errors are typed and carry context.
- [ ] PVsyst file edits preserve bytes they do not own, proven by a round-trip test.
- [ ] Tests assert observable behavior. I added only the tests a new behavior needs, and none for changes that alter no behavior.
- [ ] Each unit passed its scoped check. The full gate in section 8 is green, and I ran it after the last edit of the phase.
- [ ] The change ran the way PVsyst runs it (section 10), the checklist items its diff triggers passed, and the commit carries a `Verified:` trailer.
- [ ] No gate file (section 10) is edited together with feature code.
- [ ] The work is committed, and pushed after the full gate.
- [ ] README and CHANGELOG match the new behavior.
- [ ] My report says what I ran, what I skipped, and how many trial executions I spent.
