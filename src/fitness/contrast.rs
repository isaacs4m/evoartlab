use image::{RgbaImage, imageops};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use crate::draw::draw_concur;
use crate::vector::Environment;

pub fn calculate_fitness(env: &mut Environment, idx: usize) -> () {
    let images: Vec<RgbaImage> = draw_concur(
        &env.pool,
        env.target_width,
        env.target_height,
        env.num_threads,
    );
    let fitness_values: Vec<f64> = calculate_contrast_concur(&images);
    for (i, fitness) in fitness_values.into_iter().enumerate() {
        env.pool[i].fitness[idx] = fitness;
    }
}

fn calculate_contrast_concur(images: &Vec<RgbaImage>) -> Vec<f64> {
    let fitness_values: Arc<Mutex<Vec<Option<f64>>>> = Arc::new(Mutex::new(vec![None; images.len()]));

    images.par_iter().enumerate().for_each(|(i, image)| {
        let fitness = calculate_contrast(image);
        let mut fitness_output = fitness_values.lock().unwrap();
        fitness_output[i] = Some(fitness);
    });

    Arc::try_unwrap(fitness_values)
        .unwrap()
        .into_inner()
        .unwrap()
        .into_iter()
        .map(|fitness| fitness.unwrap())
        .collect()
}

fn calculate_contrast(image: &RgbaImage) -> f64 {
    let mut total_contrast = 0.0;
    let mut weight = 1.0;

    let mut current_image = image.clone();
    for _ in 0..5 {
        // Downscale the image
        let scaled_image = imageops::resize(
            &current_image,
            current_image.width() / 2,
            current_image.height() / 2,
            imageops::FilterType::Triangle,
        );

        // Compute contrast for the current resolution
        let contrast = compute_luminance_contrast(&scaled_image);
        total_contrast += weight * contrast;

        // Prepare for the next iteration
        current_image = scaled_image;
        weight *= 0.5; // Decrease weight for smaller scales
    }

    total_contrast
}

fn compute_luminance_contrast(image: &RgbaImage) -> f64 {
    let luminances: Vec<f64> = image
        .pixels()
        .map(|pixel| {
            let [r, g, b, _] = pixel.0;
            0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64
        })
        .collect();

    let mut contrast_sum = 0.0;
    for y in 0..image.height() {
        for x in 0..image.width() {
            if x > 0 {
                let idx = (y * image.width() + x) as usize;
                let left_idx = (y * image.width() + x - 1) as usize;
                contrast_sum += (luminances[idx] - luminances[left_idx]).abs();
            }
            if y > 0 {
                let idx = (y * image.width() + x) as usize;
                let top_idx = ((y - 1) * image.width() + x) as usize;
                contrast_sum += (luminances[idx] - luminances[top_idx]).abs();
            }
        }
    }

    -(contrast_sum / luminances.len() as f64)
}
