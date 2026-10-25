# Security Policy

GDSR parses and writes GDSII files and includes a viewer for inspecting layout
data. It does not execute code from GDSII files.

Malformed or hostile files should be rejected with an error rather than causing
panics, memory unsafety, or uncontrolled resource use. A file being too large to
open comfortably, or a parser returning a normal error for invalid GDSII data, is
not usually a vulnerability.

Please report vulnerabilities in GDSR privately by emailing
<matthewmckee04@yahoo.co.uk>. Include the affected version, a minimal
reproduction, and the impact you expect.

Security fixes target the latest released version and the `main` branch.

## Dependency Audits

Run `cargo audit` against the checked-in `Cargo.lock` to check the current
RustSec advisory database. Include informational advisories about unsoundness
when assessing updates, and inspect dependencies on every supported target with
`cargo tree --target all`.

The October 25, 2026 audit leaves two upstream findings. These are not ignored in
the audit configuration, so `cargo audit` still exits unsuccessfully:

`ruzstd 0.8.3` has a decompression-bomb vulnerability
([RUSTSEC-2026-0344](https://rustsec.org/advisories/RUSTSEC-2026-0344.html)).
`bevy_egui` enables `bevy_image/zstd_rust`, and Bevy 0.19 requires `ruzstd 0.8`;
the fix requires 0.9.1 or later. Bevy's only decoder call is in its `ktx2` module,
which is disabled in GDSR's current feature graph, including all workspace
features and supported targets. Reassess this finding before enabling KTX2 or
changing the rendering dependencies; an upstream dependency update is still
needed to remove the affected version from the lockfile.

`ttf-parser 0.25.1` is unmaintained
([RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html)). It is
pulled in by `winit`'s Linux window decorations through `sctk-adwaita`, `ab_glyph`,
and `owned_ttf_parser`. This advisory reports maintenance status rather than a
specific vulnerability. Replacing it requires an upstream font-stack change.
