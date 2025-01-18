use crate::{Vector, Triangle};
use image::{ImageBuffer, Rgba, RgbaImage};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

pub fn draw_concur(vectors: &Vec<Vector>, width: u32, height: u32, num_workers: usize) -> Vec<RgbaImage> {
    let chunk_size = (vectors.len() + num_workers - 1) / num_workers;
    let vectors_chunks: Vec<_> = vectors.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect();
    let output_images: Arc<Mutex<Vec<Option<RgbaImage>>>> = Arc::new(Mutex::new(vec![None; vectors.len()]));
    vectors_chunks.par_iter().enumerate().for_each(|(chunk_index, chunk)| {
        let images = process_chunk(chunk.to_vec(), width, height);
        let mut output = output_images.lock().unwrap();
        for (i, image) in images.into_iter().enumerate() {
            output[chunk_index * chunk_size + i] = Some(image);
        }
    });
    Arc::try_unwrap(output_images).unwrap().into_inner().unwrap().into_iter().map(|img| img.unwrap()).collect()
}

fn process_chunk(vectors_chunk: Vec<Vector>, width: u32, height: u32) -> Vec<RgbaImage> {
    vectors_chunk.iter().map(|vector| draw_vector(vector, width, height)).collect()
}

pub fn draw_vector(vector: &Vector, width: u32, height: u32) -> RgbaImage {
    let mut image = ImageBuffer::new(width, height);
    for triangle in &vector.triangles {
        draw_triangle(&mut image, triangle);
    }
    image
}

fn blend_pixel(dst: &Rgba<u8>, src: &Rgba<u8>) -> Rgba<u8> {
    let alpha = src[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    let r = (src[0] as f32 * alpha + dst[0] as f32 * inv_alpha) as u8;
    let g = (src[1] as f32 * alpha + dst[1] as f32 * inv_alpha) as u8;
    let b = (src[2] as f32 * alpha + dst[2] as f32 * inv_alpha) as u8;
    let a = (src[3] as f32 + dst[3] as f32 * inv_alpha) as u8;

    Rgba([r, g, b, a])
}

fn draw_triangle(image: &mut RgbaImage, triangle: &Triangle) {
    let color = triangle.color;

    let (mut x1, mut y1) = (triangle.vertex1.x as i32, triangle.vertex1.y as i32);
    let (mut x2, mut y2) = (triangle.vertex2.x as i32, triangle.vertex2.y as i32);
    let (mut x3, mut y3) = (triangle.vertex3.x as i32, triangle.vertex3.y as i32);

    if y1 > y2 {
        std::mem::swap(&mut y1, &mut y2);
        std::mem::swap(&mut x1, &mut x2);
    }
    if y1 > y3 {
        std::mem::swap(&mut y1, &mut y3);
        std::mem::swap(&mut x1, &mut x3);
    }
    if y2 > y3 {
        std::mem::swap(&mut y2, &mut y3);
        std::mem::swap(&mut x2, &mut x3);
    }

    let interpolate = |y, y1, y2, x1, x2| -> f32 {
        if y1 == y2 {
            return x1 as f32;
        }
        x1 as f32 + (x2 as f32 - x1 as f32) * (y as f32 - y1 as f32) / (y2 as f32 - y1 as f32)
    };

    let mut draw_scanline = |y: i32, x_start: f32, x_end: f32| {
        if y >= 0 && y < image.height() as i32 {
            let start = x_start.max(0.0) as u32;
            let end = x_end.min(image.width() as f32 - 1.0) as u32;
            for x in start..=end {
                let dst_pixel = image.get_pixel(x, y as u32);
                let blended_pixel = blend_pixel(dst_pixel, &color);
                image.put_pixel(x, y as u32, blended_pixel);
            }
        }
    };

    for y in y1..=y2 {
        let x_start = interpolate(y, y1, y3, x1, x3);
        let x_end = interpolate(y, y1, y2, x1, x2);
        if x_start <= x_end {
            draw_scanline(y, x_start, x_end);
        } else {
            draw_scanline(y, x_end, x_start);
        }
    }

    for y in y2..=y3 {
        let x_start = interpolate(y, y1, y3, x1, x3);
        let x_end = interpolate(y, y2, y3, x2, x3);
        if x_start <= x_end {
            draw_scanline(y, x_start, x_end);
        } else {
            draw_scanline(y, x_end, x_start);
        }
    }
}
