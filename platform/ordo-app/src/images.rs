//! Images taken or chosen on the device: patient photos (ADR-0028), profile pictures
//! (ADR-0047) and registration cards (ADR-0072).

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

// [impl->dsn~registration-card~2]
/// The texts of the QR codes in a photo (ADR-0072, ADR-0073). The photo is taken from a
/// canvas at full size, a strip of rows at a time (small enough for any browser's canvas),
/// and averaged in Rust to at most [`qr::EDGE`](crate::qr::EDGE) pixels: a browser
/// shrinking it would sample pixels and turn a screen's pixel grid into moiré. Then rqrr
/// reads it in the [attempts](crate::qr::ATTEMPTS), between which the page stays
/// responsive. On the device; the photo is neither kept nor sent.
pub async fn read_qr(file: &web_sys::File) -> Result<Vec<String>, &'static str> {
    use crate::qr::{self, Grey};
    let window = web_sys::window().ok_or(text::PHOTO_FAILED)?;
    let promise = window
        .create_image_bitmap_with_blob(file)
        .map_err(|_| text::PHOTO_FAILED)?;
    let bitmap: web_sys::ImageBitmap = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|_| text::PHOTO_FAILED)?
        .dyn_into()
        .map_err(|_| text::PHOTO_FAILED)?;
    let (width, height) = (bitmap.width() as usize, bitmap.height() as usize);
    let k = qr::block(width, height);
    let (w, h) = (width / k, height / k);
    // Rows per strip: whole blocks, about a million pixels.
    let strip = ((1 << 20) / width.max(1) / k * k).max(k);
    let pixels = (|| {
        let document = window.document()?;
        let canvas: web_sys::HtmlCanvasElement =
            document.create_element("canvas").ok()?.dyn_into().ok()?;
        canvas.set_width(width as u32);
        canvas.set_height(strip as u32);
        let context: web_sys::CanvasRenderingContext2d =
            canvas.get_context("2d").ok()??.dyn_into().ok()?;
        let mut grey = Vec::with_capacity(w * h);
        let mut top = 0;
        while top < h * k {
            let rows = strip.min(h * k - top);
            let (fw, fr) = (width as f64, rows as f64);
            context
                .draw_image_with_image_bitmap_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    &bitmap, 0.0, top as f64, fw, fr, 0.0, 0.0, fw, fr,
                )
                .ok()?;
            let rgba = context.get_image_data(0.0, 0.0, fw, fr).ok()?.data().0;
            grey.extend(qr::grey_blocks(&rgba, width, k));
            top += rows;
        }
        Some(grey)
    })();
    bitmap.close();
    let pixels = pixels
        .filter(|p| w > 0 && p.len() == w * h)
        .ok_or(text::PHOTO_FAILED)?;
    let grey = Grey {
        pixels,
        width: w,
        height: h,
    };
    for &attempt in qr::ATTEMPTS {
        let texts = qr::attempt(&grey, attempt);
        if !texts.is_empty() {
            return Ok(texts);
        }
        crate::core::sleep_ms(0).await; // lets the page answer between attempts
    }
    Ok(Vec::new())
}
