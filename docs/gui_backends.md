# GUI Backend Overview

This document summarizes the purpose of each legacy GUI source file located in `src`.

- `gui_beval.c` – Implements balloon evaluation tooltips and mouse tracking used by several GUI backends.
- `gui_gtk_f.c` – Custom GTK+ floating container widget managing children at arbitrary positions.
- `gui_gtk_x11.c` – GTK+ interface targeting the X11 window system.
- `gui_motif.c` – Motif toolkit based GUI implementation.
- `gui_photon.c` – Backend for the QNX Photon windowing system.
- `gui_xim.c` – X Input Method (XIM) support for multi-byte input on X11/GTK.
- `gui_xmdlg.c` – Dialog implementation for the Motif GUI variant.
- `gui_xmebw.c` – External Motif menu and toolbar widget utilities.

Rust rewrites of these backends live in crates named `rust_gui_*` within this repository.  Platform specific code should be kept inside those crates so that the core editor remains portable.
