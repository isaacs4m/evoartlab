use image::RgbaImage;
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
    let fitness_values: Vec<f64> = calculate_benford_concur(&images);
    for (i, fitness) in fitness_values.into_iter().enumerate() {
        env.pool[i].fitness[idx] = fitness;
    }
}

fn calculate_benford_concur(images: &Vec<RgbaImage>) -> Vec<f64> {
    let fitness_values: Arc<Mutex<Vec<Option<f64>>>> = Arc::new(Mutex::new(vec![None; images.len()]));

    images.par_iter().enumerate().for_each(|(i, image)| {
        let fitness = calculate_benford(image);
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

fn calculate_benford(image: &RgbaImage) -> f64 {
    let brightness_values: Vec<u8> = image
        .pixels()
        .map(|pixel| {
            let [r, g, b, _] = pixel.0;
            (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64) as u8
        })
        .collect();

    let mut digit_counts = vec![0; 10];
    for value in &brightness_values {
        if *value == 0 {
            continue;
        }
        let leading_digit = value.to_string().chars().next().unwrap().to_digit(10).unwrap();
        digit_counts[leading_digit as usize] += 1;
    }

    let total_values = brightness_values.len() as f64;
    let expected_distribution = vec![0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046];
    let mut mse = 0.0;

    for (i, &count) in digit_counts.iter().enumerate().take(9) {
        let observed = count as f64 / total_values;
        mse += (observed - expected_distribution[i]).powi(2);
    }

    100.0*-mse
}
