# Numbering System

This document describes the original documentation numbering system used before migrating to mdBook.

## Original System

Documents were numbered with a two-digit prefix:

| Number | Document | Purpose |
|--------|----------|---------|
| 00 | ARCHITECTURE | System design |
| 01 | ROADMAP | Development phases |
| 02 | TECH-SPEC | Technical specifications |
| 03 | DEV-SETUP | Development environment |
| 04 | IMPLEMENTATION-GUIDE | Code structure |
| 05 | CLI-SPECIFICATION | Command-line interface |
| 06 | TESTING | Test strategy |
| 07 | CI-CD | Continuous integration |
| 08 | SECURITY | Security guidelines |
| 09 | PERFORMANCE | Performance optimization |
| 10 | PROJECT-STATUS | Current status |

## Rationale

The numbering system provided:

- Clear reading order for new contributors
- Easy reference in discussions
- Logical progression from architecture to implementation

## Migration to mdBook

In Phase 5.5, documentation migrated to mdBook structure:

### Mapping

| Old | New |
|-----|-----|
| `docs/00-ARCHITECTURE.md` | `docs/src/development/architecture.md` |
| `docs/01-ROADMAP.md` | `docs/src/project/roadmap.md` |
| `docs/06-TESTING.md` | `docs/src/development/testing.md` |
| `docs/08-SECURITY.md` | `docs/src/security/overview.md` |
| `docs/10-PROJECT-STATUS.md` | `docs/src/project/status.md` |

### Benefits of Migration

- Better navigation with SUMMARY.md
- Searchable documentation
- Web-based viewing
- Organized by topic rather than number

## Legacy References

Some internal documents may still reference numbered files. Use this mapping to find the current location.

## See Also

- [Documentation Standards](../../development/doc-standards.md)
- [SUMMARY.md](../../SUMMARY.md)
