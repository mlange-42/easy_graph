fn main() {}

/*
use easy_graph::ui::window::{BufferWindow, WindowBuilder};
use legion::prelude::*;
use legion::schedule::Builder;
use legion::system::SystemQuery;
use plotters::drawing::bitmap_pixel::RGBPixel;
use plotters::prelude::*;
use rand::distributions::{Distribution, Poisson};
use rand::{Rng, ThreadRng};
use std::collections::HashMap;
use std::time::Instant;

const SIZE: i32 = 500;
const MARGIN: i32 = 25;

const S: usize = 0;
const I: usize = 1;
const R: usize = 2;

#[derive(Clone, Debug, PartialEq)]
struct TownId {
    id: usize,
}
#[derive(Clone, Debug, PartialEq)]
struct TownPos {
    x: f64,
    y: f64,
}
#[derive(Clone, Debug, PartialEq)]
struct TownEpi {
    epi: [i32; 3],
}
impl TownPos {
    #[allow(dead_code)]
    pub fn distance(&self, other: &TownPos) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt() as f64
    }
    pub fn distance_xy(&self, xy: &(f64, f64)) -> f64 {
        let dx = self.x - xy.0;
        let dy = self.y - xy.1;
        (dx * dx + dy * dy).sqrt() as f64
    }
}
#[derive(Clone, Debug, PartialEq)]
struct TownNeighbors {
    neighbors: Vec<(usize, f64)>,
}

fn main() {
    let mut world = create_world(50, 1_000f64.log10(), 1_000_000f64.log10());

    world.resources.insert(
        WindowBuilder::new()
            .with_dimensions(SIZE as usize, SIZE as usize)
            .with_title("Test")
            //.with_fps_limit(30.0)
            .build(),
    );

    let mut schedule = Builder::default()
        .add_system(town_connector_system(100.0))
        .add_system(sir_system(0.25, 7.0, 360.0))
        .add_thread_local(draw_system(1))
        .build();

    let now = Instant::now();
    let max_steps = 10000;
    for tick in 0..max_steps {
        if tick % 100 == 0 {
            println!("TICK {}", tick);
        }
        schedule.execute(&mut world);
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:?}", elapsed);
}

fn create_world(num_towns: usize, min_log_pop: f64, max_log_pop: f64) -> World {
    let mut rng: ThreadRng = rand::thread_rng();
    let universe = Universe::new();
    let mut world = universe.create_world();
    let mut entities: Vec<_> = (0..num_towns)
        .map(|i| {
            (
                TownId { id: i },
                TownPos {
                    x: rng.gen_range(MARGIN as f64, (SIZE - MARGIN) as f64),
                    y: rng.gen_range(MARGIN as f64, (SIZE - MARGIN) as f64),
                },
                TownEpi {
                    epi: [
                        10f64.powf(rng.gen_range(min_log_pop, max_log_pop)) as i32,
                        0,
                        0,
                    ],
                },
                TownNeighbors {
                    neighbors: Vec::new(),
                },
            )
        })
        .collect();
    entities[rng.gen_range(0, num_towns)].2.epi[I] = 5;

    world.insert((), entities);
    world
}

fn town_connector_system(max_distance: f64) -> Box<dyn Schedulable> {
    let mut initialized = false;
    let sys = SystemBuilder::<()>::new("Creator")
        .with_query(<(Read<TownId>, Read<TownPos>, Write<TownNeighbors>)>::query())
        .build(move |_commands, world, _resource, queries| {
            if !initialized {
                let queries: &mut SystemQuery<_, _> = queries;
                let coll: Vec<_> = {
                    queries
                        .iter(world)
                        .map(|(id, pos, _)| (id.id, (pos.x, pos.y)))
                        .collect()
                };
                for (id2, pos2, mut neigh2) in queries.iter(world) {
                    for (id1, pos1) in coll.iter().filter(|(id, _)| *id != id2.id) {
                        let dist = pos2.distance_xy(&pos1);
                        if dist <= max_distance {
                            &neigh2.neighbors.push((*id1, dist));
                        }
                    }
                }
                initialized = true;
            }
        });
    sys
}

fn sir_system(beta: f64, mean_inf_time: f64, mean_immune_time: f64) -> Box<dyn Schedulable> {
    let recovery_rate = 1.0 / mean_inf_time;
    let imm_loss_rate = 1.0 / mean_immune_time;
    let sys = SystemBuilder::<()>::new("Creator")
        .with_query(<Write<TownEpi>>::query())
        .build(move |_commands, world, _resource, queries| {
            let mut rng: ThreadRng = rand::thread_rng();
            let queries: &mut SystemQuery<_, _> = queries;

            for mut epi in queries.iter(world) {
                if epi.epi[I] > 0 || epi.epi[R] > 0 {
                    let n = epi.epi.iter().sum::<i32>() as f64;
                    let inf_exp = beta * epi.epi[S] as f64 * epi.epi[I] as f64 / n;
                    let rec_exp = recovery_rate * epi.epi[I] as f64;
                    let loss_exp = imm_loss_rate * epi.epi[R] as f64;
                    let inf = if inf_exp > 0.0 {
                        Poisson::new(inf_exp).sample(&mut rng) as i32
                    } else {
                        0
                    };
                    let rec = if rec_exp > 0.0 {
                        Poisson::new(rec_exp).sample(&mut rng) as i32
                    } else {
                        0
                    };
                    let loss = if loss_exp > 0.0 {
                        Poisson::new(loss_exp).sample(&mut rng) as i32
                    } else {
                        0
                    };

                    epi.epi[S] += loss - inf;
                    epi.epi[I] += inf - rec;
                    epi.epi[R] += rec - loss;

                    if epi.epi[I] > 0 {
                        println!("{:?}", epi.epi);
                    }
                }
            }
        });
    sys
}

fn migration_system() -> Box<dyn Schedulable> {
    // TODO: find faster hasher
    let mut moves: HashMap<(usize, usize), (i32, i32, i32)> = HashMap::new();
    let sys = SystemBuilder::<()>::new("Creator")
        .with_query(<(Read<TownId>, Write<TownEpi>)>::query())
        .build(move |_commands, world, _resource, queries| {
            let queries: &mut SystemQuery<_, _> = queries;
            let max_id = queries.iter(world).map(|(id, epi)| id.id).max().unwrap();

            println!("{}", max_id);
        });
    sys
}

fn draw_system(step: u32) -> Box<dyn Runnable> {
    let mut steps = 0;
    let sys = SystemBuilder::<()>::new("Drawer")
        .with_query(<(Read<TownPos>, Read<TownEpi>)>::query())
        .write_resource::<BufferWindow>()
        .build_thread_local(move |_commands, world, window, queries| {
            if window.is_open() && (step == 0 || steps % step == 0) {
                //println!("{}", queries.iter_immutable(world).count());
                window.draw(|b: BitMapBackend<RGBPixel>| {
                    let root = b.into_drawing_area();
                    root.fill(&WHITE).unwrap();

                    let colors = [BLUE, GREEN, RED];
                    for (_entity, (pos, epi)) in queries.iter_entities(world) {
                        let rad = [
                            epi.epi[I] + epi.epi[R] + epi.epi[S],
                            epi.epi[I] + epi.epi[R],
                            epi.epi[I],
                        ];
                        ((epi.epi.iter().sum::<i32>() as f64).sqrt() * 0.02).round() as i32;
                        //println!("{:?}", rad);
                        for (r, c) in rad.iter().zip(colors.iter()) {
                            let rr = ((*r as f64).sqrt() * 0.02).round() as i32;
                            if rr > 0 {
                                let circle =
                                    Circle::new((pos.x as i32, pos.y as i32), rr, c.filled());
                                root.draw(&circle).unwrap();
                            }
                        }
                    }
                });
            }
            steps += 1;
        });
    sys
}

//#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use std::collections::HashMap;

    #[test]
    fn hash_map_test() {}
}
*/
