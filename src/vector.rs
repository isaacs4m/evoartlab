use rand::Rng;
use image::{Rgba, RgbaImage};
use std::sync::Arc;

pub type FitnessFunction<'a> = Arc<dyn Fn(&mut Environment<'a>, usize) + Send + Sync + 'a>;

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub struct Vertex {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(Copy)]
pub struct Triangle {
    pub vertex1: Vertex,
    pub vertex2: Vertex,
    pub vertex3: Vertex,
    pub color: Rgba<u8>
}

impl Triangle {
    pub fn generate_random_triangle(width: u32, height: u32) -> Self {
        let mut rng = rand::thread_rng();

        let vertex1 = Vertex {
            x: rng.gen_range(0..width),
            y: rng.gen_range(0..height),
        };

        let mut vertex2 = vertex1;
        while vertex2.x == vertex1.x && vertex2.y == vertex1.y {
            vertex2.x = rng.gen_range(0..width);
            vertex2.y = rng.gen_range(0..height);
        }

        let mut vertex3 = vertex2;
        while (vertex3.x == vertex1.x && vertex3.y == vertex1.y) || (vertex3.x == vertex2.x && vertex3.y == vertex2.y) {
            vertex3.x = rng.gen_range(0..width);
            vertex3.y = rng.gen_range(0..height);
        }

        let color: Rgba<u8> = Rgba::from([rng.gen(), rng.gen(), rng.gen(), rng.gen_range(50..130)]);

        Triangle {
            vertex1,
            vertex2,
            vertex3,
            color
        }
    }

    fn difference_between_triangles(t1: &Triangle, t2: &Triangle) -> Triangle {
        let vertex1 = Vertex {
            x: t1.vertex1.x - t2.vertex1.x,
            y: t1.vertex1.y - t2.vertex1.y
        };

        let vertex2 = Vertex {
            x: t1.vertex2.x - t2.vertex2.x,
            y: t1.vertex2.y - t2.vertex2.y
        };

        let vertex3 = Vertex {
            x: t1.vertex3.x - t2.vertex3.x,
            y: t1.vertex3.y - t2.vertex3.y
        };

        let color = Rgba::from([
            t1.color[0] - t2.color[0],
            t1.color[1] - t2.color[1],
            t1.color[2] - t2.color[2],
            t1.color[3] - t2.color[3]
        ]);

        Triangle {
            vertex1,
            vertex2,
            vertex3,
            color
        }
    }

    fn sum_triangles(t1: &Triangle, t2: &Triangle) -> Triangle {
        let vertex1 = Vertex {
            x: t1.vertex1.x + t2.vertex1.x,
            y: t1.vertex1.y + t2.vertex1.y
        };

        let vertex2 = Vertex {
            x: t1.vertex2.x + t2.vertex2.x,
            y: t1.vertex2.y + t2.vertex2.y
        };

        let vertex3 = Vertex {
            x: t1.vertex3.x + t2.vertex3.x,
            y: t1.vertex3.y + t2.vertex3.y
        };

        let color = Rgba::from([
            t1.color[0] + t2.color[0],
            t1.color[1] + t2.color[1],
            t1.color[2] + t2.color[2],
            t1.color[3] + t2.color[3]
        ]);

        Triangle {
            vertex1,
            vertex2,
            vertex3,
            color
        }
    }

    fn apply_scaling_factor_in_triangle(t: &mut Triangle, scaling_factor: f64) -> () {
        t.vertex1.x = (t.vertex1.x as f64 * scaling_factor) as u32;
        t.vertex1.y = (t.vertex1.y as f64 * scaling_factor) as u32;
        t.vertex2.x = (t.vertex2.x as f64 * scaling_factor) as u32;
        t.vertex2.y = (t.vertex2.y as f64 * scaling_factor) as u32;
        t.vertex3.x = (t.vertex3.x as f64 * scaling_factor) as u32;
        t.vertex3.y = (t.vertex3.y as f64 * scaling_factor) as u32;
        t.color[0] = (t.color[0] as f64 * scaling_factor) as u8;
        t.color[1] = (t.color[1] as f64 * scaling_factor) as u8;
        t.color[2] = (t.color[2] as f64 * scaling_factor) as u8;
        t.color[3] = (t.color[3] as f64 * scaling_factor) as u8;
    }

    fn clamp_triangle(t: &mut Triangle, width: u32, height: u32) -> () {
        t.vertex1.x = t.vertex1.x.clamp(0, width-1);
        t.vertex1.y = t.vertex1.y.clamp(0, height-1);
        t.vertex2.x = t.vertex2.x.clamp(0, width-1);
        t.vertex2.y = t.vertex2.y.clamp(0, height-1);
        t.vertex3.x = t.vertex3.x.clamp(0, width-1);
        t.vertex3.y = t.vertex3.y.clamp(0, height-1);
        t.color[0] = t.color[0].clamp(0, 255);
        t.color[1] = t.color[1].clamp(0, 255);
        t.color[2] = t.color[2].clamp(0, 255);
        t.color[3] = t.color[3].clamp(50, 130);
    }
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Vector {
    pub triangles: Vec<Triangle>,
    pub fitness: Vec<f64>,
    pub rank: usize,
    pub crowding_distance: f64
}

impl Vector {
    fn generate_random_vector(width: u32, height: u32, num_triangles: usize, num_objectives: usize) -> Self {
        let mut triangles = Vec::with_capacity(num_triangles);
        for _ in 0..num_triangles {
            triangles.push(Triangle::generate_random_triangle(width, height));
        }

        Vector {
            triangles,
            fitness: vec![0.0; num_objectives],
            rank: 0,
            crowding_distance: 0.0
        }
    }

    fn generate_mutant_vector(xr1: &Vector, xr2: &Vector, xr3: &Vector, scaling_factor: f64, width: u32, height: u32, num_objectives: usize) -> Vector {
        // xr2 - xr3
        let mut difference_vector: Vector = Vector::difference_between_vectors(xr2, xr3, num_objectives);
        // F * (xr2 - xr3)
        for triangle in difference_vector.triangles.iter_mut() {
            Triangle::apply_scaling_factor_in_triangle(triangle, scaling_factor);
        }
        // xr1 + F * (xr2 - xr3)
        let mut mutant_vector: Vector = Vector::sum_vectors(xr1, &difference_vector, num_objectives);

        // clamp final triangles
        for triangle in mutant_vector.triangles.iter_mut() {
            Triangle::clamp_triangle(triangle, width, height);
        }

        mutant_vector
    }

    fn difference_between_vectors(vector1: &Vector, vector2: &Vector, num_objectives: usize) -> Vector {
        let mut triangles = Vec::with_capacity(vector1.triangles.len());
        for i in 0..vector1.triangles.len() {
            triangles.push(Triangle::difference_between_triangles(&vector1.triangles[i], &vector2.triangles[i]));
        }

        Vector {
            triangles,
            fitness: vec![0.0; num_objectives],
            rank: 0,
            crowding_distance: 0.0
        }
    }

    fn sum_vectors(vector1: &Vector, vector2: &Vector, num_objectives: usize) -> Vector {
        let mut triangles = Vec::with_capacity(vector1.triangles.len());
        for i in 0..vector1.triangles.len() {
            triangles.push(Triangle::sum_triangles(&vector1.triangles[i], &vector2.triangles[i]));
        }

        Vector {
            triangles,
            fitness: vec![0.0; num_objectives],
            rank: 0,
            crowding_distance: 0.0
        }
    }

    fn crossover(vector1: &Vector, vector2: &Vector, crossover_probability: f64, num_objectives: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut new_triangles = Vec::with_capacity(vector1.triangles.len());

        for i in 0..vector1.triangles.len() {
            if rng.gen_bool(crossover_probability) {
                new_triangles.push(vector1.triangles[i]);
            } else {
                new_triangles.push(vector2.triangles[i]);
            }
        }

        Vector {
            triangles: new_triangles,
            fitness: vec![0.0; num_objectives],
            rank: 0,
            crowding_distance: 0.0
        }
    }
}

pub struct Environment<'a> {
    pub pool: Vec<Vector>,
    pub pool_size: usize,
    pub scaling_factor: f64,
    pub crossover_probability: f64,
    pub target_img: RgbaImage,
    pub target_width: u32,
    pub target_height: u32,
    pub num_triangles: usize,
    pub num_threads: usize,
    pub num_objectives: usize,
    pub fitness_functions: Vec<FitnessFunction<'a>>,
    pub tournament_size: usize
}

impl<'a> Environment<'a> {
    pub fn new(
        pool_size: usize,
        scaling_factor: f64,
        crossover_probability: f64,
        target_img: RgbaImage,
        num_triangles: usize,
        num_threads: usize,
        fitness_functions: Vec<FitnessFunction<'a>>,
        tournament_size: usize
    ) -> Self {
        if pool_size < 4 {
            panic!("Pool size must be at least 4");
        }
        let target_width = target_img.width();
        let target_height = target_img.height();

        let pool = Vec::with_capacity(pool_size);
        let num_objectives = fitness_functions.len();

        Environment {
            pool,
            pool_size,
            scaling_factor,
            crossover_probability,
            target_img,
            target_width,
            target_height,
            num_triangles,
            num_threads,
            num_objectives,
            fitness_functions,
            tournament_size
        }
    }

    pub fn generate_initial_pool(&mut self) -> () {
        for _ in 0..self.pool_size {
            self.pool.push(
                Vector::generate_random_vector(
                    self.target_width,
                    self.target_height,
                    self.num_triangles,
                    self.num_objectives
                )
            );
        }
        self.calculate_fitness_for_population();
    }

    pub fn calculate_fitness_for_population(&mut self) {
        let fitness_functions = self.fitness_functions.clone(); // Clone the fitness functions
        for (idx, fitness_fn) in fitness_functions.iter().enumerate() {
            fitness_fn(self, idx); // Mutable borrow of `self` is now safe
        }
    }

    fn tournament_selection(&self, tournament_size: usize) -> &Vector {
        let mut rng = rand::thread_rng();

        let mut best: Option<&Vector> = None;
        for _ in 0..tournament_size {
            let index = rng.gen_range(0..self.pool_size);
            let candidate = &self.pool[index];

            if let Some(current_best) = best {
                if candidate.fitness < current_best.fitness {
                    best = Some(candidate);
                }
            } else {
                best = Some(candidate);
            }
        }
        best.unwrap()
    }

    pub fn get_first_front(&self) -> Vec<Vector> {
        let fronts = self.non_dominated_sort(&self.pool);
        fronts.get(0).cloned().unwrap_or_default()
    }

    pub fn iterate(&mut self) {
        let old_pool = self.pool.clone();

        // mutate the pool via formula
        // xr1 + scaling_factor * (xr2 - xr3)
        for i in 0..self.pool_size {
            let xr1 = self.tournament_selection(self.tournament_size);
            let xr2 = self.tournament_selection(self.tournament_size);
            let xr3 = self.tournament_selection(self.tournament_size);

            let mutant_vector = Vector::generate_mutant_vector(
                xr1,
                xr2,
                xr3,
                self.scaling_factor,
                self.target_width,
                self.target_height,
                self.num_objectives
            );

            let trial_vector = Vector::crossover(&mutant_vector, &self.pool[i], self.crossover_probability, self.num_objectives);
            self.pool[i] = trial_vector;
        }

        // calculate the fitness of the current pool
        self.calculate_fitness_for_population();

        // select the best vectors from the old and new pool
        for i in 0..self.pool_size {
            if self.pool[i].fitness > old_pool[i].fitness {
                self.pool[i] = old_pool[i].clone();
            }
        }


        if self.num_objectives == 1 {
            self.single_objective_selection();
        } else {
            self.nsga_selection(&old_pool);
        }
    }

    fn calculate_crowding_distance(&mut self, front: &mut Vec<Vector>) {
        let num_objectives = self.num_objectives;
        for vector in front.iter_mut() {
            vector.crowding_distance = 0.0;
        }

        for obj_idx in 0..num_objectives {
            front.sort_by(|a, b| a.fitness[obj_idx].partial_cmp(&b.fitness[obj_idx]).unwrap());

            let last_idx = front.len() - 1;
            front[0].crowding_distance = f64::INFINITY;
            front[last_idx].crowding_distance = f64::INFINITY;

            let min_value = front[0].fitness[obj_idx];
            let max_value = front[front.len() - 1].fitness[obj_idx];
            let range = max_value - min_value;

            if range == 0.0 {continue;}
            for i in 1..front.len() - 1 {
                front[i].crowding_distance +=
                    (front[i + 1].fitness[obj_idx] - front[i - 1].fitness[obj_idx]) / range;
            }
        }
    }

    fn single_objective_selection(&mut self) {
        let mut combined_pool = self.pool.clone();
        combined_pool.sort_by(|a, b| a.fitness[0].partial_cmp(&b.fitness[0]).unwrap());
        self.pool = combined_pool.into_iter().take(self.pool_size).collect();
    }

    fn nsga_selection(&mut self, old_pool: &[Vector]) {
        let mut combined_pool = self.pool.clone();
        combined_pool.extend_from_slice(old_pool);
        let mut fronts = self.non_dominated_sort(&combined_pool);

        let mut new_pool = Vec::new();
        for front in &mut fronts {
            self.calculate_crowding_distance(front);

            if new_pool.len() + front.len() > self.pool_size {
                let mut sorted_front = front.clone();
                sorted_front.sort_by(|a, b| b.crowding_distance.partial_cmp(&a.crowding_distance).unwrap());
                let remaining_slots = self.pool_size - new_pool.len();
                new_pool.extend(sorted_front.into_iter().take(remaining_slots));
                break;
            } else {
                new_pool.extend(front.clone());
            }
        }

        self.pool = new_pool;
    }

    fn non_dominated_sort(&self, vectors: &[Vector]) -> Vec<Vec<Vector>> {
        let mut fronts: Vec<Vec<Vector>> = Vec::new();
        let mut domination_count: Vec<usize> = vec![0; vectors.len()];
        let mut dominated_solutions: Vec<Vec<usize>> = vec![Vec::new(); vectors.len()];
        let mut current_front: Vec<usize> = Vec::new();

        for i in 0..vectors.len() {
            for j in 0..vectors.len() {
                if i == j {
                    continue;
                }

                if dominates(&vectors[i], &vectors[j]) {
                    dominated_solutions[i].push(j);
                } else if dominates(&vectors[j], &vectors[i]) {
                    domination_count[i] += 1;
                }
            }

            if domination_count[i] == 0 {current_front.push(i);}
        }

        while !current_front.is_empty() {
            let mut next_front: Vec<usize> = Vec::new();
            for &i in &current_front {
                for &j in &dominated_solutions[i] {
                    domination_count[j] -= 1;
                    if domination_count[j] == 0 {
                        next_front.push(j);
                    }
                }
            }

            fronts.push(current_front.iter().map(|&i| vectors[i].clone()).collect());
            current_front = next_front;
        }

        fronts
    }

    pub fn fitness_mean(&self) -> Vec<f64> {
        if self.pool.is_empty() {
            return vec![0.0; self.num_objectives];
        }

        let mut sums = vec![0.0; self.num_objectives];

        for vector in self.pool.iter() {
            for (i, fitness) in vector.fitness.iter().enumerate() {
                sums[i] += fitness;
            }
        }

        sums.iter().map(|sum| sum / self.pool_size as f64).collect()
    }

    pub fn fitness_std_dev(&self) -> Vec<f64> {
        if self.pool.is_empty() {
            return vec![0.0; self.num_objectives];
        }

        let means = self.fitness_mean();
        let mut sum_of_squares = vec![0.0; self.num_objectives];
        for vector in self.pool.iter() {
            for (i, fitness) in vector.fitness.iter().enumerate() {
                sum_of_squares[i] += (fitness - means[i]).powi(2);
            }
        }

        sum_of_squares
            .iter()
            .map(|sum| (sum / self.pool_size as f64).sqrt())
            .collect()
    }
}

// checks if `a` dominates `b`, intially assumes that it does
fn dominates(a: &Vector, b: &Vector) -> bool {
    let mut better_in_all = true;
    let mut strictly_better_in_one = false;

    for (f_a, f_b) in a.fitness.iter().zip(&b.fitness) {
        if f_a > f_b {
            better_in_all = false;
        }
        if f_a < f_b {
            strictly_better_in_one = true;
        }
    }

    better_in_all && strictly_better_in_one
}

