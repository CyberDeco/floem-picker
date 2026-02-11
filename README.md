# floem-picker

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/floem-picker.svg)](https://crates.io/crates/floem-picker)
[![Documentation](https://docs.rs/floem-picker/badge.svg)](https://docs.rs/floem-picker)
![Rust](https://img.shields.io/badge/rust-1.85+-orange?logo=rust)

**Simple color picker widget for Floem, works with both vger (left) and vello (right) backends.**
</div>

<p align="center">
<img width="344" height="604" alt="Vger" src="https://github.com/user-attachments/assets/83bff2ee-f26d-4b53-a5b8-9d173c97df9f" />
<img width="344" height="604" alt="Vello" src="https://github.com/user-attachments/assets/a6230e2e-13be-412b-bfdb-8e73c8752c85" />
</p>

## Notes

The eyedropper functionality is only available for macOS because:
1. It has FFI bindings to `NSColorSampler`,
2. There's no built-in Windows equivalent (as far as I'm aware), and
3. The Linux equivalent [appears to be buggy](https://github.com/pop-os/xdg-desktop-portal-cosmic/issues/251).

If macOS isn't detected, it should simply disable the eyedropper button and retain the rest of the functionality, but if you want to explicitly exclude it, add this to your `Cargo.toml`:
```rust
[dependencies]
floem-picker = { version = "0.2", default-features = false, features = ["alpha"] }
```


## Credits

Inspired by [System Color Picker](https://sindresorhus.com/system-color-picker)
