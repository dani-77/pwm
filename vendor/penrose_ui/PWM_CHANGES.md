# Local changes to penrose_ui

Unmodified copy of penrose_ui 0.4.0 from crates.io, apart from:

- `src/core/mod.rs`, `Draw::new_window`: a round trip on the x11rb connection
  after creating the window. Without it, the status bar's first `XCopyArea` on
  the Xlib connection could reach the X server before the window existed. Xlib's
  default error handler then ended pwm with `BadDrawable`.
