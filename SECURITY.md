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
