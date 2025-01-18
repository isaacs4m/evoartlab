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
        env.num_threads
    );
    let fitness_values: Vec<f64> = calculate_mse_concur(env, &images);
    for (i, fitness) in fitness_values.into_iter().enumerate() {
        env.pool[i].fitness[idx] = fitness;
    }
}

fn calculate_mse_concur(env: &Environment, images: &Vec<RgbaImage>) -> Vec<f64> {
    let chunk_size = (images.len() + env.num_threads - 1) / env.num_threads;
    let images_chunks: Vec<_> = images.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect();

    let fitness_values: Arc<Mutex<Vec<Option<f64>>>> = Arc::new(Mutex::new(vec![None; images.len()]));

    images_chunks.par_iter().enumerate().for_each(|(chunk_index, images_chunk)| {
        let fitness_chunk = process_mse_chunk(env, images_chunk.to_vec());
        let mut fitness_output = fitness_values.lock().unwrap();
        for (i, fitness) in fitness_chunk.into_iter().enumerate() {
            fitness_output[chunk_index * chunk_size + i] = Some(fitness);
        }
    });

    Arc::try_unwrap(fitness_values).unwrap().into_inner().unwrap().into_iter().map(|fitness| fitness.unwrap()).collect()
}

fn process_mse_chunk(env: &Environment, images_chunk: Vec<RgbaImage>) -> Vec<f64> {
    images_chunk.iter().map(|image| calculate_mse(&env.target_img, image)).collect()
}

fn calculate_mse(img1: &RgbaImage, img2: &RgbaImage) -> f64 {
    let img1 = img1.clone().into_raw();
    let img2 = img2.clone().into_raw();
    let mse: f64 = img1.iter().zip(img2.iter()).map(|(a, b)| {
        let diff = *a as f64 - *b as f64;
        diff * diff
    }).sum();
    mse/ (img1.len() as f64)
}
