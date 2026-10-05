English | [Deutsch](SECURITY.de.md)

# Security policy

## Reporting a vulnerability

Please report security problems privately through GitHub: **Security → Report a vulnerability** on this repository. Do not open a public issue.

Include what you found, how to reproduce it, and which version or commit you tested. You will get an answer within a week. Once a fix is released, the report is published with credit unless you prefer otherwise.

## Scope

Especially relevant for Hows:

- anything that stores or exports typed text, which the recorder must never do;
- files written outside the folder the user chose, other than the default folder Hows uses when that folder is unusable;
- crafted `.steps` files that crash the app, run code, or write files when opened;
- network traffic of any kind, since the app is meant to make none.

What Hows records and stores by design is described in [Privacy](docs/en/privacy.md).

## Supported versions

Only the latest release gets security fixes.
