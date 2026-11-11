# 0016. Licence and public repository

- **Status:** Accepted
- **Date:** 2026-11-11
- **Ticket:** STU-50

Grado is licensed under the PolyForm Noncommercial License 1.0.0, and the repository is public.
Anyone may read, run, change and share it for a noncommercial purpose. Nobody else may sell it or
use it commercially. Outside contributions aren't accepted, and pull requests and issues are open
to collaborators only.

| Where | What |
|---|---|
| `LICENSE.md` | The licence text, after a `Required Notice:` line naming the copyright holder |
| `Cargo.toml` | `license = "PolyForm-Noncommercial-1.0.0"` for every crate, through the workspace |
| `CONTRIBUTING.md` | Says contributions aren't accepted |
| Repository settings | Pull requests and issues limited to collaborators. CodeQL default setup, secret scanning with push protection, and branch protection on `main` requiring the three CI jobs (ADR 0013) |

**Consequences:**
- The licence binds others, not the copyright holder, who can still relicense or sell Grado.
  Accepting no contributions keeps one copyright holder.
- Grado isn't open source in the OSI sense: it's source-available.
- Before going public, the full history is scanned for secrets with gitleaks.
- The commit history is published as it is, author emails included.
- Going public is what turns on branch protection and CodeQL, because GitHub's Free plan offers
  neither on a private repository. It also unblocks SonarQube Cloud's free plan (STU-51).
- A new dependency's licence is still checked by `deny.toml`. Our own crates are skipped there,
  because they're never published.
