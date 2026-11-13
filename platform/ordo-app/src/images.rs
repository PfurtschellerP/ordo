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

// [impl->dsn~registration-card~1]
/// The texts of the QR codes in a photo (ADR-0072): drawn at most 1600 pixels wide, made
/// grey and read with rqrr, on the device. The photo is neither kept nor sent.
pub async fn read_qr(file: &web_sys::File) -> Result<Vec<String>, &'static str> {
    const EDGE: f64 = 1600.0;
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
    let scale = (EDGE / width.max(height)).min(1.0);
    let (w, h) = ((width * scale).round(), (height * scale).round());
    let pixels = (|| {
        let document = window.document()?;
        let canvas: web_sys::HtmlCanvasElement =
            document.create_element("canvas").ok()?.dyn_into().ok()?;
        canvas.set_width(w as u32);
        canvas.set_height(h as u32);
        let context: web_sys::CanvasRenderingContext2d =
            canvas.get_context("2d").ok()??.dyn_into().ok()?;
        context
            .draw_image_with_image_bitmap_and_dw_and_dh(&bitmap, 0.0, 0.0, w, h)
            .ok()?;
        Some(context.get_image_data(0.0, 0.0, w, h).ok()?.data().0)
    })();
    bitmap.close();
    let rgba = pixels.ok_or(text::PHOTO_FAILED)?;
    let (w, h) = (w as usize, h as usize);
    // Grey value per pixel (ITU-R BT.601 weights).
    let grey: Vec<u8> = rgba
        .chunks_exact(4)
        .map(|p| {
            ((299 * u32::from(p[0]) + 587 * u32::from(p[1]) + 114 * u32::from(p[2])) / 1000) as u8
        })
        .collect();
    if grey.len() != w * h {
        return Err(text::PHOTO_FAILED);
    }
    let mut prepared = rqrr::PreparedImage::prepare_from_greyscale(w, h, |x, y| grey[y * w + x]);
    Ok(prepared
        .detect_grids()
        .iter()
        .filter_map(|g| g.decode().ok().map(|(_, text)| text))
        .collect())
}
