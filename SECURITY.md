# Security

Onyx Brain is experimental software.

## Supported Versions

| Version | Status |
| --- | --- |
| v0.0.1 | Experimental |

## Reporting Security Issues

If possible, report security issues privately to the project maintainer before public disclosure.

## Security Model

Onyx Brain is designed around conservative local execution:

- File operations must remain sandboxed.
- Terminal commands must remain allowlisted.
- Arbitrary shell execution is not allowed.
- Network access is not enabled by default.
- Generated runtime data should stay under `data/`.
- Generated projects should stay under `sandbox/`.
- Snapshots, rollback, transactions, and journals should not be bypassed for risky actions.

## Limitations

Do not run untrusted prompts or tasks without reviewing tool permissions. Do not use Onyx Brain to operate sensitive production systems. The current release is an experimental runtime skeleton, not a hardened security product.
