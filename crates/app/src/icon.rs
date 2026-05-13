/// Raw PNG bytes embedded at compile time.
const PNG_BYTES: &[u8] = include_bytes!("../../../assets/icon.png");

/// Decode the icon PNG and return (rgba_bytes, width, height).
/// Resizes to `target_size` if specified.
pub fn load(target_size: Option<u32>) -> (Vec<u8>, u32, u32) {
    use image::GenericImageView;

    let img = image::load_from_memory(PNG_BYTES)
        .expect("Failed to decode assets/icon.png");

    let img = match target_size {
        Some(s) => img.resize(s, s, image::imageops::FilterType::Lanczos3),
        None    => img,
    };

    let (w, h) = img.dimensions();
    let rgba   = img.to_rgba8().into_raw();
    (rgba, w, h)
}

/// Convenience: returns 32×32 RGBA for tray icon / window icon.
pub fn rgba_32() -> (Vec<u8>, u32, u32) {
    load(Some(32))
}

/// Convenience: returns 256×256 RGBA for high-DPI use.
pub fn rgba_256() -> (Vec<u8>, u32, u32) {
    load(Some(256))
}
