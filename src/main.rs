pub mod draw;
pub mod vector;
pub mod fitness {
    pub mod pixel_compare;
    pub mod benford;
    pub mod contrast;
}
pub use vector::{Environment, FitnessFunction, Vector, Triangle};

use eframe::egui;
use rfd::FileDialog;
use std::time::Instant;
use std::sync::Arc;
use std::collections::HashMap;

// INFO: Add new fitness functions here and in the function below
use crate::fitness::pixel_compare;
use crate::fitness::contrast;
use crate::fitness::benford;

fn define_fitness_functions<'a>() -> HashMap<String, FitnessFunction<'a>> {
    let mut fitness_registry: HashMap<String, FitnessFunction> = HashMap::new();
    fitness_registry.insert("Pixel Compare".to_string(), Arc::new(pixel_compare::calculate_fitness) as FitnessFunction);
    fitness_registry.insert("Contrast".to_string(), Arc::new(contrast::calculate_fitness) as FitnessFunction);
    fitness_registry.insert("Benford".to_string(), Arc::new(benford::calculate_fitness) as FitnessFunction);
    fitness_registry
}

struct EnvParams {
    pool_size: usize,
    scaling_factor: f64,
    crossover_probability: f64,
    num_triangles: usize,
    num_threads: usize,
    tournament_size: usize,
}

impl EnvParams {
    fn new() -> Self {
        Self {
            pool_size: 100,
            scaling_factor: 1.7,
            crossover_probability: 0.03,
            num_triangles: 250,
            num_threads: 16,
            tournament_size: 3,
        }
    }
}

struct EvoArtLab<'a> {
    running: bool,
    destination_folder: String,
    parameters: EnvParams,
    fitness_functions: HashMap<String, FitnessFunction<'a>>,
    fitness_functions_checkbox: HashMap<String, bool>,
    selected_functions: Vec<FitnessFunction<'a>>,
    time_elapsed: Option<Instant>,
    stop_condition: u16,
    environment: Option<Environment<'a>>,
    generation: usize,
    target_img: Option<image::RgbaImage>,
    img_path: String,
}

impl<'a> Default for EvoArtLab<'a> {
    fn default() -> Self {
        let fitness_functions = define_fitness_functions();
        let fitness_functions_checkbox = fitness_functions.keys().map(|name| (name.clone(), false)).collect();
        let env_params = EnvParams::new();

        Self {
            running: false,
            destination_folder: String::from("No folder selected"),
            parameters: env_params,
            fitness_functions,
            fitness_functions_checkbox,
            selected_functions: Vec::with_capacity(2),
            time_elapsed: None,
            stop_condition: 30,
            environment: None,
            generation: 0,
            target_img: None,
            img_path: String::from("No image selected"),
        }
    }
}

impl<'a> eframe::App for EvoArtLab<'a> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Run").clicked() {
                    if self.running { return; }
                    self.running = true;
                    self.time_elapsed = Some(Instant::now());

                    self.selected_functions = self
                        .fitness_functions_checkbox
                        .iter()
                        .filter_map(|(name, &is_selected)| {
                            if is_selected {
                                self.fitness_functions.get(name).cloned()
                            } else {
                                None
                            }
                        })
                        .collect();

                    let final_img: Option<image::RgbaImage>;
                    if self.target_img.is_none() {
                        final_img = Some(image::open("test.jpg").unwrap().to_rgba8());
                    } else {
                        final_img = self.target_img.clone();
                    }

                    self.environment = Some(Environment::new(
                        self.parameters.pool_size,
                        self.parameters.scaling_factor,
                        self.parameters.crossover_probability,
                        final_img.unwrap(),
                        self.parameters.num_triangles,
                        self.parameters.num_threads,
                        self.selected_functions.clone(),
                        self.parameters.tournament_size,
                    ));
                    self.environment.as_mut().unwrap().generate_initial_pool();
                    self.generation = 1;
                }

                if ui.button("Stop").clicked() {
                    if self.running == false { return; }
                    self.running = false;
                    self.time_elapsed = None;
                    self.environment = None;
                    self.generation = 0;
                }

                if ui.button("Choose Destination Folder").clicked() {
                    if let Some(folder) = FileDialog::new().pick_folder() {
                        self.destination_folder = folder.display().to_string();
                    } else {
                        self.destination_folder = "No folder selected".to_string();
                    }
                }

                if ui.button("Choose Template Image").clicked() {
                    if let Some(img) = FileDialog::new().pick_file() {
                        self.img_path = img.to_str().unwrap().to_string();
                        self.target_img = Some(image::open(img).unwrap().to_rgba8());
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Parameters");
                    ui.add(
                        egui::DragValue::new(&mut self.parameters.pool_size)
                           .prefix("Pool Size: ")
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.parameters.scaling_factor)
                           .prefix("Scaling Factor: ")
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.parameters.crossover_probability)
                        .prefix("Crossover Probability: ")
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.parameters.num_triangles)
                        .prefix("Number of Triangles: ")
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.parameters.num_threads)
                        .prefix("Number of Threads: ")
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.parameters.tournament_size)
                        .prefix("Tournament Size: ")
                    );

                });

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("Stop Condition");
                            ui.add(egui::DragValue::new(&mut self.stop_condition).prefix(format!("Time Elapsed in Seconds:")));
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("Fitness Functions");
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (key, toggled) in self.fitness_functions_checkbox.iter_mut() {
                                    ui.checkbox(toggled, key);
                                }
                            });
                        });
                    });
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label(format!("Template Image: {}", self.img_path));
                ui.label(format!("Destination Folder: {}", self.destination_folder));
                ui.label(format!("Generation: {}", self.generation));

                if let Some(start_time) = self.time_elapsed {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    ui.label(format!("Time Elapsed: {:.2} seconds", elapsed));
                } else {
                    ui.label("Time Elapsed: N/A");
                }

                ui.horizontal(|ui| {
                    let mean_fitness;
                    let std_deviation;

                    if let Some(env) = &self.environment {
                        let mean = env.fitness_mean();
                        let std = env.fitness_std_dev();
                        let mean_str = mean.iter().map(|f| format!("{:.1}", f)).collect::<Vec<String>>().join(", ");
                        let std_str = std.iter().map(|f| format!("{:.1}", f)).collect::<Vec<String>>().join(", ");
                        mean_fitness = format!("Mean Fitness: ({})", mean_str);
                        std_deviation = format!("Std Deviation: ({})", std_str);
                    } else {
                        mean_fitness = "Mean Fitness: N/A".to_string();
                        std_deviation = "Std Deviation: N/A".to_string();
                    }

                    ui.label(mean_fitness);
                    ui.label(std_deviation);
                });
            });
        });

        if self.running {
            if let Some(mut env) = self.environment.take() {
                self.generation += 1;

                // limit execution by time elapsed
                if self.time_elapsed.unwrap().elapsed().as_millis() > (self.stop_condition * 1000) as u128 {
                    // get first front
                    let front = env.get_first_front();
                    // save all vectors in the first front if at least one fitness was selected
                    for (_, vector) in front.iter().enumerate() {
                        if vector.fitness.len() != 0 {
                            let img = draw::draw_vector(&vector, env.target_width, env.target_height);
                            let fitnesses = vector.fitness.iter().map(|f| f.to_string()).collect::<Vec<String>>().join("_");
                            let file_name;
                            if self.destination_folder != "No folder selected" {
                                file_name = format!("{}/front_{}.png", self.destination_folder, fitnesses);
                            } else {
                                file_name = format!("front_{}.png", fitnesses);
                            }
                            img.save(file_name).unwrap();
                        }
                    }
                    self.running = false;
                    self.time_elapsed = None;
                    self.environment = None;
                    self.generation = 0;
                } else {
                    env.iterate();
                    self.environment = Some(env);
                }
            }

        }

        ctx.request_repaint();
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder {
            title: Some("EvoArtLab".to_string()),
            min_inner_size: Some(egui::vec2(420.0, 350.0)),
            max_inner_size: Some(egui::vec2(420.0, 350.0)),
            ..Default::default()
        },
        centered: true,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "EvoArtLab", // "Evolutionary Art Fitness Functions Tester"
        options,
        Box::new(|_cc| Ok(Box::new(EvoArtLab::default())))
    );
}
