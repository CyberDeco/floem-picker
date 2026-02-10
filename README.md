# floem-picker

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/floem-picker.svg)](https://crates.io/crates/floem-picker)
[![Documentation](https://docs.rs/floem-picker/badge.svg)](https://docs.rs/floem-picker)
![Rust](https://img.shields.io/badge/rust-1.85+-orange?logo=rust)

</div>

<p align="center">
<img width="344" height="604" alt="Screenshot 2026-02-09 at 9 07 09 PM" src="https://github.com/user-attachments/assets/90b3288d-5462-4c46-8be7-9b7b5a6ff101" />
</p>

Simple color picker widget for Floem, works with both vger and vello backends.

The eyedropper functionality is only exposed for macOS because:
1. It has FFI bindings to `NSColorSampler`,
2. There's no native Windows equivalent (as far as I'm aware), and
3. The Linux equivalent [appears to be buggy](https://github.com/pop-os/xdg-desktop-portal-cosmic/issues/251).
