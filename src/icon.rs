//! Tray icon rendering. We render the *same* Claude spark the KDE plasmoid shows
//! (`plasmoid/contents/icons/claude.png`, embedded at build time), so the icon is
//! identical across every desktop — tray, panel, everywhere. State is conveyed by
//! overall opacity (and the tooltip/menu text), never by changing the icon's shape
//! or colour.
//!
//! Pixel format required by the StatusNotifierItem spec (and ksni): ARGB32 in
//! network byte order, i.e. each pixel is 4 bytes laid out `[A, R, G, B]`.

use image::imageops::FilterType;
use std::sync::OnceLock;

/// The canonical Claude spark, baked into the binary so the tray needs no asset
/// files on disk and stays byte-identical to the plasmoid's icon.
const CLAUDE_PNG: &[u8] = include_bytes!("../plasmoid/contents/icons/claude.png");

/// Decode + scale the spark to `size` once, caching the straight-alpha RGBA bytes.
/// The tray only ever asks for one size, so a single-slot cache is plenty.
fn base_rgba(size: u32) -> &'static [u8] {
    static CACHE: OnceLock<Vec<u8>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            image::load_from_memory(CLAUDE_PNG)
                .expect("embedded claude.png must decode")
                .resize_exact(size, size, FilterType::Lanczos3)
                .to_rgba8()
                .into_raw()
        })
        .as_slice()
}

/// Render the Claude spark at `size` px, scaling every pixel's alpha by `alpha`
/// (0.0–1.0) so callers can dim it when idle or pulse it when busy.
pub fn claude_icon(size: i32, alpha: f32) -> ksni::Icon {
    let s = size.max(1) as u32;
    let rgba = base_rgba(s); // [R, G, B, A] per pixel
    let mul = alpha.clamp(0.0, 1.0);

    let mut data = vec![0u8; (s * s * 4) as usize];
    for px in 0..(s * s) as usize {
        let (r, g, b, a) = (
            rgba[px * 4],
            rgba[px * 4 + 1],
            rgba[px * 4 + 2],
            rgba[px * 4 + 3],
        );
        let i = px * 4;
        data[i] = (a as f32 * mul).round() as u8; // A
        data[i + 1] = r;
        data[i + 2] = g;
        data[i + 3] = b;
    }

    ksni::Icon {
        width: s as i32,
        height: s as i32,
        data,
    }
}
