use anyhow::Result;
use image::imageops;
use image::{DynamicImage, ImageFormat, RgbaImage};
use ndarray::{Array, CowArray};
use ort::{Session, Value};
use std::io::Cursor;
use std::sync::OnceLock;

static THRESHOLD_BG: OnceLock<u8> = OnceLock::new();

pub struct ProcessedImage {
    pub data: Vec<u8>,
}

// Function to find the bounding box containing non-transparent pixels
pub(crate) fn find_alpha_bounds(image: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let mut min_x = u32::MAX;
    let mut max_x = 0;
    let mut min_y = u32::MAX;
    let mut max_y = 0;
    let threshold_bg = THRESHOLD_BG.get().unwrap_or(&10);

    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel[3] > *threshold_bg {
            // Non-transparent pixel
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }

    if min_x > max_x || min_y > max_y {
        log::warn!(
            "No non-transparent pixels found: {:?}",
            (min_x, min_y, max_x, max_y)
        );
        return None;
    }

    Some((min_x, min_y, max_x, max_y))
}

pub async fn process_image(session: &Session, image_data: &[u8]) -> Result<ProcessedImage> {
    // Create image from bytes
    let img = image::load_from_memory(image_data)?;

    // Process image using ONNX model
    let processed = process_dynamic_image(session, img)?;

    // Convert back to bytes
    let mut buffer = Cursor::new(Vec::new());
    processed.write_to(&mut buffer, ImageFormat::Png)?;

    Ok(ProcessedImage {
        data: buffer.into_inner(),
    })
}

fn process_dynamic_image(session: &Session, dynamic_img: DynamicImage) -> Result<DynamicImage> {
    let input_shape = session.inputs[0]
        .dimensions()
        .map(|dim| dim.unwrap())
        .collect::<Vec<usize>>();

    let input_img = dynamic_img.into_rgba8();
    let scaling_factor = f32::min(
        1., // Avoid upscaling
        f32::min(
            input_shape[3] as f32 / input_img.width() as f32, // Width ratio
            input_shape[2] as f32 / input_img.height() as f32, // Height ratio
        ),
    );

    let mut resized_img = imageops::resize(
        &input_img,
        input_shape[3] as u32,
        input_shape[2] as u32,
        imageops::FilterType::Triangle,
    );

    let input_tensor = CowArray::from(
        Array::from_shape_fn(input_shape, |indices| {
            let mean = 128.;
            let std = 256.;
            (resized_img[(indices[3] as u32, indices[2] as u32)][indices[1]] as f32 - mean) / std
        })
        .into_dyn(),
    );

    let inputs = vec![Value::from_array(session.allocator(), &input_tensor)?];
    let outputs = session.run(inputs)?;
    let output_tensor = outputs[0].try_extract::<f32>()?;

    for (indices, alpha) in output_tensor.view().indexed_iter() {
        resized_img[(indices[3] as u32, indices[2] as u32)][3] = (alpha * 255.) as u8;
    }

    let output_img = imageops::resize(
        &resized_img,
        (input_img.width() as f32 * scaling_factor) as u32,
        (input_img.height() as f32 * scaling_factor) as u32,
        imageops::FilterType::Triangle,
    );

    Ok(DynamicImage::ImageRgba8(output_img))
}
