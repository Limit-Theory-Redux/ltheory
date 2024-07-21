use parley::layout::Glyph;
use swash::scale::{image::Content, Render, Scaler, Source, StrikeWith};
use swash::zeno::{Format, Vector};

use crate::render::Color;

pub(super) fn render_glyph(
    buffer: &mut [Color],
    scaler: &mut Scaler,
    color: &Color,
    glyph: &Glyph,
    glyph_x: f32,
    glyph_y: f32,
    image_width: u32,
) {
    // Compute the fractional offset
    // You'll likely want to quantize this in a real renderer
    // TODO: swash for some reason shifts horizontal offset by 1 pixel to the left so we have to correct it here
    let offset = Vector::new(glyph_x.fract() + 1.0, glyph_y.fract());

    // Render the glyph using swash
    let glyph_image = Render::new(
        // Select our source order
        &[
            Source::ColorOutline(0),
            Source::ColorBitmap(StrikeWith::BestFit),
            Source::Outline,
        ],
    )
    // Select the simple alpha (non-subpixel) format
    .format(Format::Alpha)
    // Apply the fractional offset
    .offset(offset)
    // Render the image
    .render(scaler, glyph.id)
    .unwrap();

    let glyph_width = glyph_image.placement.width;
    let glyph_height = glyph_image.placement.height;
    let glyph_x = (glyph_x.floor() as i32 + glyph_image.placement.left) as u32;
    let glyph_y = (glyph_y.floor() as i32 - glyph_image.placement.top) as u32;

    match glyph_image.content {
        Content::Mask => {
            // TODO: check if a single loop over i: [0..glyph_height*glyph_width] will be possible and more efficient
            let mut i = 0;
            for pixel_y in 0..glyph_height {
                for pixel_x in 0..glyph_width {
                    if glyph_image.data[i] > 0 {
                        let x = glyph_x + pixel_x;
                        let y = glyph_y + pixel_y;
                        let idx = y * image_width + x;

                        let alpha = color_u8_to_f32(glyph_image.data[i]);
                        let color = color.with_alpha(alpha);

                        buffer[idx as usize].blend_with(&color);
                    }

                    i += 1;
                }
            }
        }
        Content::SubpixelMask => {
            unimplemented!("Subpixel mask format is not implemented for the text rendering")
        }
        Content::Color => {
            let row_size = glyph_width as usize * 4;
            for (pixel_y, row) in glyph_image.data.chunks_exact(row_size).enumerate() {
                for (pixel_x, pixel) in row.chunks_exact(4).enumerate() {
                    if pixel[3] > 0 {
                        let x = glyph_x + pixel_x as u32;
                        let y = glyph_y + pixel_y as u32;
                        let idx = y * glyph_width + x;

                        let color = Color::new(
                            color_u8_to_f32(pixel[0]),
                            color_u8_to_f32(pixel[1]),
                            color_u8_to_f32(pixel[2]),
                            color_u8_to_f32(pixel[3]),
                        );

                        buffer[idx as usize].blend_with(&color);
                    }
                }
            }
        }
    };
}

#[inline]
fn color_u8_to_f32(v: u8) -> f32 {
    v as f32 / 255.0
}
