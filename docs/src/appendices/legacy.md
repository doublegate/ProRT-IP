# Appendix C: Legacy Documentation

This appendix contains documentation from earlier project phases that has been superseded but remains valuable for historical reference.

## Purpose

Legacy documentation preserves:

- **Historical Context** - How the project evolved
- **Migration Guides** - Upgrading from older approaches
- **Reference Material** - Understanding deprecated features
- **Audit Compliance** - Complete documentation history

## Legacy Documents

### [Phase 4 Compliance](./legacy/phase4-compliance.md)

Documentation of Phase 4 compliance requirements and verification processes.

### [Phase 4 Enhancements](./legacy/phase4-enhancements.md)

Detailed documentation of Phase 4 feature enhancements including zero-copy processing and NUMA optimization.

### [Regression Strategy](./legacy/regression-strategy.md)

The regression testing strategy used during Phase 4-5 transition.

### [Numbering System](./legacy/numbering-system.md)

The original documentation numbering system (00-XX format) and migration to mdBook.

### [Examples (Legacy)](./legacy/examples-legacy.md)

Original command-line examples from earlier versions.

### [Benchmarking (Legacy)](./legacy/benchmarking-legacy.md)

Original benchmarking methodology and results from Phase 4.

## Migration Notes

### From Numbered Docs to mdBook

The project migrated from numbered markdown files to mdBook in Phase 5.5:

| Old Path | New Path |
|----------|----------|
| `docs/00-ARCHITECTURE.md` | `docs/src/development/architecture.md` |
| `docs/01-ROADMAP.md` | `docs/src/project/roadmap.md` |
| `docs/06-TESTING.md` | `docs/src/development/testing.md` |

## Deprecation Policy

Legacy documentation is preserved, marked as legacy, and maintained for accuracy only.

## See Also

- [Phase Archives](./archives.md) - Complete phase documentation
- [Sprint Reports](./sprint-reports.md) - Development history
- [Documentation Standards](../development/doc-standards.md) - Current conventions
