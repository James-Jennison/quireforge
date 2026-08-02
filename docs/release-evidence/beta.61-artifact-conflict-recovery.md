# Beta.61 artifact conflict recovery

Beta.61 was an unpublished M58 candidate bound to source
`a1e9d5016f28b3cae54a0161ec1fdec2012e2b6c`. Independent pinned-container
builds produced identical desktop packages but divergent worker Debian packages.
The guarded finalizer correctly rejected the archive collision.

Both complete, internally consistent sets were moved atomically without byte
changes and are excluded from canonical selection:

- `archive/conflicts/0.1.0-beta.61/noncanonical-first-build-effacbf510d81bac`
  has worker SHA-256 `75380b6783afc7b6edbaab2966fd0810b9c8f95b8baafb685264ba20fb891747`
  and manifest SHA-256 `effacbf510d81bac85f7387e55534d190b0319fae9a9a481a1a1ba2af44cf8b0`.
- `archive/conflicts/0.1.0-beta.61/likely-installed-second-build-8a605e731a53db41`
  has worker SHA-256 `a79c26e34a051cbefe272f32e714ad9be12f85965229531b14242c2c5dfc0148`
  and manifest SHA-256 `8a605e731a53db41134009f2fe607f5a85994563c345ca1e607fdaaf8a2badcd`.

Both contain the identical desktop package SHA-256
`e4b93b633078b85f6293b3bb99023ec68336349729b0e986aa8e6002367cea5a`.
The installed beta.61 package provenance is likely the second set from the
authorized installation sequence, but is not cryptographically provable from
the installed package database alone. Neither set is canonical or publishable.
Beta.62 adds deterministic worker tree/compression timestamp normalization.
