use rand::prelude::SliceRandom;
use rand::RngCore;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};

#[wasm_bindgen(start)]
pub async fn start() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
}

#[wasm_bindgen]
pub async fn generate_hug(
    hue: f64,
    avatar: web_sys::ImageBitmap,
    for_name: String,
    valid: String,
    code: String,
    attribution: bool,
) -> Result<(), wasm_bindgen::JsValue> {
    run(hue, &avatar, &for_name, &valid, &code, attribution)
        .await
        .map_err(|e| e.to_string().into())
}

pub async fn run(
    hue: f64,
    avatar: &web_sys::ImageBitmap,
    for_name: &str,
    valid: &str,
    code: &str,
    attribution: bool,
) -> anyhow::Result<()> {
    let data = include_bytes!("../assets/hug-coupon.png");
    let img_in = image::load_from_memory(data)
        .map_err(|e| anyhow::anyhow!(e))?
        .to_rgba8();

    let image = image::imageops::huerotate(&img_in, hue.rem_euclid(360.0).round() as i32);

    let window = web_sys::window().ok_or_else(|| {
        anyhow::anyhow!("No window found! Are you running this application in a browser?")
    })?;

    let document = window.document().ok_or_else(|| {
        anyhow::anyhow!("No document found! Are you running this application in a browser?")
    })?;

    let canvas = document
        .get_element_by_id("canvas")
        .ok_or_else(|| anyhow::anyhow!("No canvas found!"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|e| anyhow::anyhow!("Expected a canvas, got: {e:?}"))?;

    let context = canvas
        .get_context("2d")
        .map_err(|e| anyhow::anyhow!("Could not create a rendering context: {:?}", e))?
        .ok_or_else(|| anyhow::anyhow!("Could not create a rendering context"))?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|e| anyhow::anyhow!("Expected a 2d rendering context, got: {e:?}"))?;

    context.clear_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());

    context.begin_path();
    context.set_fill_style(&JsValue::from_str("white"));
    context
        .ellipse(
            581.5,
            199.5,
            134.5,
            134.5,
            0.0,
            0.0,
            2.0 * std::f64::consts::PI,
        )
        .map_err(|e| anyhow::anyhow!("Could not draw ellipse: {:?}", e))?;
    context.fill();

    context
        .set_global_composite_operation("source-in")
        .map_err(|e| anyhow::anyhow!("Could not set global composite operation: {:?}", e))?;

    context
        .draw_image_with_image_bitmap_and_dw_and_dh(avatar, 447.0, 65.0, 269.0, 269.0)
        .map_err(|e| anyhow::anyhow!("Could not draw image: {:?}", e))?;

    let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
        wasm_bindgen::Clamped(&image),
        image.width(),
        image.height(),
    )
    .map_err(|e| anyhow::anyhow!("Could not create a bitmap: {:?}", e))?;

    let bitmap_promise = window
        .create_image_bitmap_with_image_data(&image_data)
        .map_err(|e| anyhow::anyhow!("Could not create a bitmap: {:?}", e))?;

    let bitmap = wasm_bindgen_futures::JsFuture::from(bitmap_promise)
        .await
        .map_err(|e| anyhow::anyhow!("Could not create a bitmap: {:?}", e))?
        .dyn_into::<web_sys::ImageBitmap>()
        .map_err(|e| anyhow::anyhow!("Expected an image bitmap, got: {e:?}"))?;

    context
        .set_global_composite_operation("destination-over")
        .map_err(|e| anyhow::anyhow!("Could not set global composite operation: {:?}", e))?;

    context
        .draw_image_with_image_bitmap(&bitmap, 0., 0.)
        .map_err(|e| anyhow::anyhow!("Could not draw image: {:?}", e))?;

    context
        .set_global_composite_operation("source-over")
        .map_err(|e| anyhow::anyhow!("Could not set global composite operation: {:?}", e))?;

    context.set_fill_style(&JsValue::from_str(
        format!("hsl({}, 100%, 20%)", hue).as_str(),
    ));
    context.set_font("italic 24px JetBrains Mono");

    let mut rand = rand::thread_rng();

    context
        .fill_text(for_name, 75.0, 183.0)
        .map_err(|e| anyhow::anyhow!("Could not draw text: {:?}", e))?;

    let valid_times = include_str!("../assets/valid_times.txt")
        .lines()
        .collect::<Vec<_>>();

    let valid_final = if valid.is_empty() {
        valid_times.choose(&mut rand).unwrap()
    } else {
        valid
    };

    context
        .fill_text(valid_final, 75., 260.)
        .map_err(|e| anyhow::anyhow!("Could not draw text: {:?}", e))?;

    let adjectives = include_str!("../assets/adjectives.txt")
        .lines()
        .collect::<Vec<_>>();

    let codes = include_str!("../assets/lines.txt")
        .lines()
        .collect::<Vec<_>>();

    let code_full = if code.is_empty() {
        format!(
            "{}-{}-{:04}",
            adjectives.choose(&mut rand).unwrap(),
            codes.choose(&mut rand).unwrap(),
            rand.next_u32() % 10000
        )
    } else {
        code.to_owned()
    };

    context
        .fill_text(&code_full, 75.0, 340.0)
        .map_err(|e| anyhow::anyhow!("Could not draw text: {:?}", e))?;

    if attribution {
        context.set_font("italic 10px JetBrains Mono");
        context
            .fill_text("natty.sh/run-app/free-hugs", 112.0, 60.0)
            .map_err(|e| anyhow::anyhow!("Could not draw text: {:?}", e))?;
    }

    context
        .set_global_composite_operation("source-over")
        .map_err(|e| anyhow::anyhow!("Could not set global composite operation: {:?}", e))?;

    Ok(())
}
