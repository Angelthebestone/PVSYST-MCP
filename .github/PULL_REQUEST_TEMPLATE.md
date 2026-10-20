# Pull request

## What and why

<!-- What changes, and which problem it solves. The why matters more than the what. -->

## How it was verified

<!-- Commands you ran, and anything you could not verify (real PVsyst, GUI, licence). -->

## Checklist

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` is clean
- [ ] `cargo test` passes without a PVsyst installation
- [ ] `python tools/check_deps.py` passes (module graph still acyclic)
- [ ] Tests assert observable behavior (input → output), not mock interactions
- [ ] New files live in the feature folder they belong to and stay under 300 lines
- [ ] No `unwrap`, `expect`, `panic` or `todo` added to production paths
- [ ] `README.md` / `CHANGELOG.md` updated if the tool surface, arguments or requirements changed
- [ ] No PVsyst binaries, licence files, vendor data or private project files added
- [ ] Commits follow Conventional Commits and carry no AI co-author trailer
