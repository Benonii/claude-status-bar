//! Procedural icon rendering. We draw the "spark" directly into an ARGB32 buffer
//! every frame instead of shipping PNG assets, so animation is just maths.
//!
//! Pixel format required by the StatusNotifierItem spec (and ksni): ARGB32 in
//! network byte order, i.e. each pixel is 4 bytes laid out `[A, R, G, B]`.

#[derive(Clone, Copy)]
pub struct Rgb(pub u8, pub u8, pub u8);

fn lerp_u8(from: u8, to: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    (from as f32 + (to as f32 - from as f32) * t).round() as u8
}

/// Render a four-pointed sparkle.
///
/// * `size`   – icon edge length in px (square).
/// * `color`  – arm colour.
/// * `extent` – how far the arms reach (animate this to "pulse").
/// * `alpha`  – overall opacity 0.0–1.0 (animate this to "breathe").
///
/// The star shape is the level set of `sqrt(|dx|) + sqrt(|dy|) <= extent`, an
/// astroid-like curve whose concave sides read as a sparkle. A small bright core
/// is blended in and whitened for a touch of glow.
pub fn spark(size: i32, color: Rgb, extent: f32, alpha: f32) -> ksni::Icon {
    let (w, h) = (size, size);
    let cx = (w as f32 - 1.0) / 2.0;
    let cy = (h as f32 - 1.0) / 2.0;
    let aa = 0.7_f32; // anti-alias falloff width, in shape-function units
    let core_r = size as f32 * 0.11;

    let mut data = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 - cx).abs();
            let dy = (y as f32 - cy).abs();

            // Star body with a soft edge.
            let f = dx.sqrt() + dy.sqrt();
            let star = if f <= extent {
                1.0
            } else {
                (1.0 - (f - extent) / aa).max(0.0)
            };

            // Round bright core.
            let r = (dx * dx + dy * dy).sqrt();
            let core = (1.0 - r / core_r).clamp(0.0, 1.0);

            let cov = star.max(core);
            if cov <= 0.0 {
                continue;
            }

            let rr = lerp_u8(color.0, 255, 0.7 * core);
            let gg = lerp_u8(color.1, 255, 0.7 * core);
            let bb = lerp_u8(color.2, 255, 0.7 * core);
            let a = (cov * alpha * 255.0).round().clamp(0.0, 255.0) as u8;

            let idx = ((y * w + x) * 4) as usize;
            data[idx] = a;
            data[idx + 1] = rr;
            data[idx + 2] = gg;
            data[idx + 3] = bb;
        }
    }

    ksni::Icon {
        width: w,
        height: h,
        data,
    }
}
