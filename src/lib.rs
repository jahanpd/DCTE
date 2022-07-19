use std::f32::consts::E;
use std::iter::zip;
use rand::prelude::*;
use yew::{Properties};

#[derive(Clone, PartialEq, Properties)]
pub struct Settings {
    pub length: usize,
    pub genome: String,
    pub mutation_rate: f32,
    pub growth_rate: f32,
    pub seed: u32
}

#[derive(Clone, PartialEq, Properties)]
pub struct Organism {
    pub coordinates: Vec<Location>,
    pub ages: Vec<f32>,
    pub senescent: Vec<bool>,
    pub genomes: Vec<String>,
    pub settings: Settings,
    pub size: u32,
    pub samplesize: u32
}

#[derive(Debug, Clone)]
pub struct Location {
    pub x: i32,
    pub y: i32
}

impl Location {
    pub fn get_neighbours(&self) -> Vec<Location> {
        let diffs: Vec<(i32, i32)> = [
            (0, 1), (1, 0), (1, 1), (1, -1),
            (0, -1), (-1, 0), (-1, -1), (-1, 1)
        ].to_vec();
        diffs.iter().map(
        |l| Location { x: self.x + l.0, y: self.y + l.1}
        ).filter(
        |l| (l.x >= 0) & (l.y >= 0)
        ).collect()
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        (self.x == other.x) & (self.y == other.y)
    }
}
impl Eq for Location {}

impl Settings {
    pub fn init_organism(&self) -> Organism {
        let coords = [Location {
            x:(self.length / 2) as i32, 
            y:(self.length / 2) as i32
        }].to_vec();
        println!("{:?}", coords);
        // build base plot
        Organism {
            coordinates: coords.clone(),
            ages: vec![0.0; coords.len()],
            senescent: vec![false; coords.len()],
            genomes: vec![self.genome.clone(); coords.len()],
            settings: self.clone(),
            size: 1,
            samplesize: 10
        }
    }
}

// calculate the euclidean distance between two points in the grid as a flat array
fn distance_calc(j: &Location, i: &Location) -> f32 {
    ((j.x - i.x).pow(2) as f32 + 
    (j.y - i.y).pow(2) as f32 ).powf(0.5)
}

// calculate the difference between rna signals
fn difference_rna(g1: &String, g2: &String) -> f32 {
        zip(g1.chars().collect::<Vec<char>>(), g2.chars().collect::<Vec<char>>()).map(
        |(x, y)| if x == y { 0 } else { 1 }
        ).collect::<Vec<u32>>().iter().sum::<u32>() as f32
}

// routine to mutate genes
fn gene_mutation(gene: String, mutation_rate: f32) -> String {
    let mut rng = thread_rng();
    let thresh = rng.gen::<f32>();
    gene.chars().collect::<Vec<char>>().iter().map(
        |l| {
            if mutation_rate > thresh { 
                let bases = "GCAT".chars();
                bases.choose(&mut rng).unwrap() 
            } 
            else { 
                l.clone() 
            }
        }
    ).collect::<String>().to_string()
}

impl Organism {
    pub fn mean_age(&self) -> f32 {
        self.ages.iter().sum::<f32>() / self.settings.length.pow(2) as f32
    }

    pub fn entropy(&self) -> Vec<(String, f32)> {
        let mut entropies: Vec<(String, f32)> = vec![];
        for (i, bp) in self.settings.genome.chars().enumerate() {
            let mut gcta: Vec<f32> = vec![0.0, 0.0, 0.0, 0.0];
            for genome in &self.genomes {
                match genome.chars().nth(i).expect("index value not found") {
                    'G' => gcta[0] += 1.0,
                    'C' => gcta[1] += 1.0,
                    'T' => gcta[2] += 1.0,
                    'A' => gcta[3] += 1.0,
                    _ => panic!("base pair not found")
                }
            }
            gcta = gcta.iter().map(|x| x / (self.size as f32)).collect();
            let entropy = gcta.iter().fold(0f32, |acc, x| acc - (x * (x + 0.000000000000000001).ln()));
            entropies.push((bp.to_string(), entropy));
        }
        return entropies
    }

    // TODO implement multithreading
    pub fn growstep(self) -> Organism {
        let mut rng = thread_rng();
        let mut new_coords = self.coordinates.clone();
        let mut new_age = self.ages.clone();
        let mut new_genes: Vec<String> = self.genomes.clone();
        let mut total_cells_sampled = 0;
        for (i, coordi) in self.coordinates.iter().enumerate() {
            let mut age = 0.0;
            let threshsplit = rng.gen::<f32>();
            let probsplit = 0.02 * E.powf(-1f32 * (new_age[i]));
            let split = probsplit > threshsplit;
            let mut countcells = 0;  // this is a counter for cells sampled
            let mut neighbours: Vec<Location> = coordi.get_neighbours().into_iter().filter(
                |l| (l.x < self.settings.length as i32) & (l.y < self.settings.length as i32)
            ).collect();
            for (j, coordj) in new_coords.iter().enumerate() {
                neighbours.retain(|x| x != coordj);
                let prob = E.powf(-0.2 * distance_calc(coordj, coordi));
                let thresh = rng.gen::<f32>();
                if prob > thresh {
                    countcells += 1;
                    age = age + difference_rna(&new_genes[i], &new_genes[j])
                } else {
                    age = age + 0.0
                }
            };
            if split & (neighbours.len() > 0) {
                new_coords.push(
                    neighbours.choose(&mut rng).unwrap().clone()
                );
                new_age.push(new_age[i].clone());
                new_genes.push(
                    gene_mutation(new_genes[i].clone(), self.settings.growth_rate)
                );
                new_genes[i] = gene_mutation(new_genes[i].clone(), self.settings.growth_rate);
            }
            new_age[i] = age;
            new_genes[i] = gene_mutation(new_genes[i].clone(), self.settings.mutation_rate);
            total_cells_sampled += countcells;
        }
        Organism {
            coordinates: new_coords.clone(),
            ages: new_age,
            senescent: self.senescent,
            genomes: new_genes,
            settings: self.settings,
            size: new_coords.len() as u32,
            samplesize: total_cells_sampled / self.coordinates.len() as u32
        }
    }
}

