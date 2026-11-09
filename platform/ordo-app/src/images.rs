//! Images taken or chosen on the device: patient photos (ADR-0028) and profile pictures
//! (ADR-0047).

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use wasm_bindgen::JsCast;

use crate::text;

/// Downscales an image on the device and re-encodes it as JPEG, without camera or location
/// metadata. `steps` are (longest edge in pixels, JPEG quality), tried in order until the
/// result fits into `max_bytes`.
pub async fn downscale(
    file: &web_sys::File,
    steps: &[(f64, f64)],
    max_bytes: usize,
) -> Result<Vec<u8>, &'static str> {
    let window = web_sys::window().ok_or(text::PHOTO_FAILED)?;
    let promise = window
        .create_image_bitmap_with_blob(file)
        .map_err(|_| text::PHOTO_FAILED)?;
    let bitmap: web_sys::ImageBitmap = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|_| text::PHOTO_FAILED)?
        .dyn_into()
        .map_err(|_| text::PHOTO_FAILED)?;
    let (width, height) = (f64::from(bitmap.width()), f64::from(bitmap.height()));
    let document = window.document().ok_or(text::PHOTO_FAILED)?;
    let result = (|| {
        for &(edge, quality) in steps {
            let scale = (edge / width.max(height)).min(1.0);
            let (w, h) = ((width * scale).round(), (height * scale).round());
            let canvas: web_sys::HtmlCanvasElement =
                document.create_element("canvas").ok()?.dyn_into().ok()?;
            canvas.set_width(w as u32);
            canvas.set_height(h as u32);
            let context: web_sys::CanvasRenderingContext2d =
                canvas.get_context("2d").ok()??.dyn_into().ok()?;
            context
                .draw_image_with_image_bitmap_and_dw_and_dh(&bitmap, 0.0, 0.0, w, h)
                .ok()?;
            let url = canvas
                .to_data_url_with_type_and_encoder_options(
                    "image/jpeg",
                    &wasm_bindgen::JsValue::from_f64(quality),
                )
                .ok()?;
            let bytes = BASE64.decode(url.split_once(',')?.1).ok()?;
            if bytes.len() <= max_bytes {
                return Some(Ok(bytes));
            }
        }
        Some(Err(text::PHOTO_TOO_LARGE))
    })();
    bitmap.close();
    result.unwrap_or(Err(text::PHOTO_FAILED))
}

/// A `data:` URL for a JPEG, PNG or WebP image.
pub fn data_url(bytes: &[u8]) -> String {
    let mime = if bytes.starts_with(b"\x89PNG") {
        "image/png"
    } else if bytes.len() > 12 && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else {
        "image/jpeg"
    };
    format!("data:{mime};base64,{}", BASE64.encode(bytes))
}
