# Changelog

All notable changes to `pulse-system-types` are documented here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the crate
adheres to [Semantic Versioning](https://semver.org/) (pre-1.0: a minor bump
may break source compatibility).

Releases before 0.6.3 are described by the git history and tags.

## [0.7.0] - Unreleased

pulse-null renamed its agent concept from "entity" to "pulse"
(dnacenta/pulse-null#115). This release carries the rename into the shared
contract types.

**Source-breaking, on-disk compatible.** Code that names the renamed items must
be updated; serialized data written by earlier versions still loads.

### Changed

- `TaskCreator::Entity` is now `TaskCreator::Pulse`. It serializes as
  `"pulse"`. The legacy value `"entity"` still deserializes as
  `TaskCreator::Pulse` through a serde alias, so existing `schedule.json` files
  with `"created_by": "entity"` keep loading. They are rewritten as
  `"created_by": "pulse"` the next time they are saved.
- `PluginContext::entity_root` is now `PluginContext::pulse_root`, and
  `PluginContext::entity_name` is now `PluginContext::pulse_name`.
  `PluginContext` has no serde derives, so this only affects source.
- Doc comments and the crate keywords now use "pulse" (or "agent") in place of
  "entity" for the agent concept.

### Migration

| Before (≤ 0.6)          | After (0.7)            |
|-------------------------|------------------------|
| `TaskCreator::Entity`   | `TaskCreator::Pulse`   |
| `ctx.entity_root`       | `ctx.pulse_root`       |
| `ctx.entity_name`       | `ctx.pulse_name`       |

| Wire value                   | ≤ 0.6 reads as        | 0.7 reads as          | 0.7 writes  |
|------------------------------|-----------------------|-----------------------|-------------|
| `"created_by": "entity"`     | `TaskCreator::Entity` | `TaskCreator::Pulse`  | `"pulse"`   |
| `"created_by": "pulse"`      | error                 | `TaskCreator::Pulse`  | `"pulse"`   |

Downgrading is not on-disk safe: a 0.6 reader rejects `"pulse"`, so a
`schedule.json` written by a 0.7 consumer will not load under 0.6.

`Plugin`, `Tool`, `SetupPrompt`, `ScheduledTask` and the other contract types
are distinct types in 0.6 and 0.7. A host and its plugins have to move to 0.7
together.

## [0.6.3] - 2026-04-01

### Added

- Optional `prediction` and `valence` fields on `monitoring::OutcomeRecord`.
