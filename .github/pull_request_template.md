## What changed

<!-- One or two sentences. -->

## Invariants

Confirm the ones this touches (see `CLAUDE.md`):

- [ ] No C/C++ pulled into a default build
- [ ] Sealed segments stay immutable
- [ ] Read path stays zero-copy; no `Vec<Vec<f32>>`
- [ ] On-disk / archive formats are versioned; unknown versions refused
- [ ] Ordinals do not escape the process
- [ ] Authorization stays a pre-filter, never a post-filter
- [ ] No cross-segment state introduced
- [ ] Every file still under 200 lines

## Recall

<!-- Any ANN index change needs recall@k against the flat index.
     "It's faster" without a recall number is not a result. -->
