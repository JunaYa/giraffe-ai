## 📝 Rust Rules Review – Modification Plan

> Author: AI assistant
> Date: 

### 1. Overview

After a thorough review of the current Rust rule-set under `.cursor/rules/rust/`, the guidelines are largely complete, orthogonal, and dynamically loadable via **`main.mdc`**.  However, two gaps were identified:

1. **Testing best-practices module is missing.**  `main.mdc` references `quality/testing.mdc`, but the file does not exist.  This breaks the rule-loading table and leaves projects without unified testing standards.
2. **`core/code-quality.mdc` lacks the mandatory decision-making Mermaid chart.**  All other core/feature files now include a chart; adding one keeps the format consistent and aids discoverability.

No other structural issues were found.  Directory layout remains logical (`core`, `features`, `quality`, `simple`, `complex`, `workflows`).  Core and feature rules comprehensively cover Rust best-practices and remain orthogonal.

### 2. Proposed Changes

| #   | Change                                                                                                                                | Affected File                                | Rationale                                                                                                                                     |
| --- | ------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Create **`quality/testing.mdc`** with complete Rust testing standards and a decision-flow Mermaid chart                               | `./.cursor/rules/rust/quality/testing.mdc`   | Fulfils missing reference in `main.mdc`; provides unit/integration/bench test guidelines, mocking, CI hints, coverage, property testing, etc. |
| 2   | Add **Mermaid decision chart** to the top of **`core/code-quality.mdc`**                                                              | `./.cursor/rules/rust/core/code-quality.mdc` | Brings file in line with the "chart-first" convention adopted across the rule-set.                                                            |
| 3   | Update module table in **`main.mdc`** (if needed) to confirm `quality/testing.mdc` exists                                             | `./.cursor/rules/rust/main.mdc`              | Keeps documentation accurate.                                                                                                                 |
| 4   | *Optional*: add CI/CD & release workflow rules in future (`workflows/ci-cd.mdc`) – **not required immediately** but noted for roadmap | –                                            | Complete lifecycle guidance (build, test, release)                                                                                            |

### 3. Deliverables

1. **`quality/testing.mdc`** – roughly 8-10 sections:
   * Testing strategy selection chart (unit vs integration vs e2e vs property vs benchmark)
   * Cargo test organisation (`#[cfg(test)]`, `/tests`, `/benches`)
   * Mocking & test doubles (`mockall`, `axum_test`, `wiremock`)
   * Async testing with Tokio
   * Property testing (`proptest`)
   * Coverage (`cargo tarpaulin`, GitHub Actions snippet)
   * Benchmarks (`criterion`)
   * CI integration checklist
2. **Chart insertion** at top of `core/code-quality.mdc` – simple flow for adopting code-quality rules (naming, file size, lint enforcement).

### 4. Compatibility

The additions are non-breaking and purely additive:

* **Legacy projects** gain a clear path to adopt unified testing standards without altering existing code.
* **New projects** continue to rely on dynamic rule loading – the missing file removal fixes table inconsistencies.

### 5. Next Steps

1. Implement the above changes via follow-up code edits.
2. Run `cargo clippy` on representative sample projects to verify lint rules referenced still align.
3. Regenerate rule index documentation if tooling exists.

---
*End of plan*
