use easy_graph::geom::grid::Grid;
use easy_graph::ui::chart::{Chart, ChartBuilder, Series};
use easy_graph::ui::window::{BufferWindow, WindowBuilder};
use legion::prelude::*;
use legion::schedule::{Builder, Schedulable};
use legion::system::SystemBuilder;
use legion::world::Universe;
use minifb::Scale;
use plotters::drawing::bitmap_pixel::RGBPixel;
use plotters::drawing::BitMapBackend;
use plotters::prelude::*;
use plotters::style::{BLUE, GREEN, RED, WHITE};
use rand::{Rng, ThreadRng};
use std::time::Instant;

#[derive(Clone, Debug, PartialEq)]
enum EpiStat {
    S,
    I,
    R,
}

#[derive(Clone, Debug, PartialEq)]
struct EpiStatComp {
    epi: EpiStat,
    tick_changed: i32,
}

fn main() {
    println!("Chart example");
    let universe = Universe::new();
    let mut world = universe.create_world();

    let size = 256_usize;
    let mut grid = Grid::new(
        size,
        size,
        EpiStatComp {
            epi: EpiStat::S,
            tick_changed: std::i32::MIN,
        },
    );
    let mut epi = grid.get_mut(size / 2, size / 2);
    epi.epi = EpiStat::I;
    epi.tick_changed = 0;

    world.resources.insert(grid);

    world.resources.insert(
        WindowBuilder::new()
            .with_dimensions(size, size)
            .with_title("Map")
            .with_scale(Scale::X2)
            .with_position((50, 50))
            .with_fps_skip(30.0)
            .build(),
    );

    world.resources.insert(
        ChartBuilder::new()
            .with_title("EpiStat")
            .with_dimensions(600, 400)
            .with_position(560, 50)
            .with_data_limit(500)
            .with_y_label("# Individuals x 1000")
            .with_y_scale(0.001)
            .with_ylim(Some(0.0), None)
            .with_fps_skip(30.0)
            .add_series(Series::line("S", &BLUE))
            .add_series(Series::line("I", &RED))
            .add_series(Series::line("R", &GREEN))
            .build(),
    );

    let mut schedule = Builder::default()
        .add_system(infection_system(0.02, 2, 5, 25))
        .add_thread_local(chart_system(1))
        .add_thread_local(draw_system(1))
        .build();

    let now = Instant::now();
    let max_tick = 10000;
    for tick in 0..max_tick {
        if tick % 1000 == 0 {
            println!("TICK {}", tick);
        }
        schedule.execute(&mut world);
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:?}", elapsed);
    println!("Avg. FPS:: {:?}", max_tick as f64 / elapsed.as_secs_f64());
}

fn infection_system(
    beta: f64,
    inf_radius: i32,
    ticks_infected: i32,
    ticks_immune: i32,
) -> Box<dyn Schedulable> {
    let mut tick = 0;
    let mut counter_grid: Option<Grid<i32>> = None;
    let sys = SystemBuilder::<()>::new("Infection")
        .with_query(<Write<EpiStatComp>>::query())
        .write_resource::<Grid<EpiStatComp>>()
        .build(move |_commands, _world, grid, _queries| {
            let mut rng: ThreadRng = rand::thread_rng();
            let grid: &mut Grid<EpiStatComp> = grid;

            let counter = counter_grid.get_or_insert(Grid::new(
                grid.width() as usize,
                grid.height() as usize,
                0,
            ));
            counter.fill(|| 0);

            for x in 0..grid.width() {
                for y in 0..grid.height() {
                    let epi = grid.get_mut(x as usize, y as usize);
                    if epi.epi == EpiStat::I {
                        if tick > epi.tick_changed + ticks_infected {
                            epi.epi = EpiStat::R;
                            epi.tick_changed = tick;
                        } else {
                            for xx in (x - inf_radius)..=(x + inf_radius) {
                                for yy in (y - inf_radius)..=(y + inf_radius) {
                                    if grid.contains(xx, yy) {
                                        *counter.get_mut(xx as usize, yy as usize) += 1;
                                    }
                                }
                            }
                        }
                    } else if epi.epi == EpiStat::R {
                        if tick > epi.tick_changed + ticks_immune {
                            epi.epi = EpiStat::S;
                            epi.tick_changed = std::i32::MIN;
                        }
                    }
                }
            }

            for x in 0..grid.width() {
                for y in 0..grid.height() {
                    let epi = grid.get_mut(x as usize, y as usize);
                    if epi.epi == EpiStat::S {
                        let cnt = *counter.get(x as usize, y as usize);
                        if cnt > 0 {
                            let prob = 1.0 - (1.0 - beta).powi(cnt);
                            if rng.gen_bool(prob) {
                                epi.epi = EpiStat::I;
                                epi.tick_changed = tick;
                            }
                        }
                    }
                }
            }
            tick += 1;
        });
    sys
}

fn chart_system(step: u32) -> Box<dyn Runnable> {
    let mut steps = 0;
    let sys = SystemBuilder::<()>::new("Chart")
        //.with_query(<Write<EpiStatComp>>::query())
        .write_resource::<Chart>()
        .write_resource::<Grid<EpiStatComp>>()
        .build_thread_local(move |_commands, _world, (chart, grid), _queries| {
            if chart.is_open() {
                let grid: &mut Grid<EpiStatComp> = grid;
                let (mut s, mut i, mut r) = (0, 0, 0);
                for epi in grid.iter() {
                    match epi.epi {
                        EpiStat::S => s += 1,
                        EpiStat::I => i += 1,
                        EpiStat::R => r += 1,
                    }
                }
                chart.push_time_series(steps as f64, &[s as f64, i as f64, r as f64]);
                if steps % step == 0 {
                    chart.update();
                }
            }
            steps += 1;
        });
    sys
}

fn draw_system(step: u32) -> Box<dyn Runnable> {
    let mut steps = 0;
    let sys = SystemBuilder::<()>::new("Drawer")
        .write_resource::<BufferWindow>()
        .write_resource::<Grid<EpiStatComp>>()
        .build_thread_local(move |_commands, _world, (win, grid), _queries| {
            let win: &mut BufferWindow = win;
            if win.is_open() && (step == 0 || steps % step == 0) {
                win.draw(|b: BitMapBackend<RGBPixel>| {
                    let root = b.into_drawing_area();
                    root.fill(&WHITE).unwrap();

                    let grid: &mut Grid<EpiStatComp> = grid;
                    for (i, epi) in grid.iter().enumerate() {
                        let (x, y) = grid.coord(i);
                        let x = x as i32;
                        let y = y as i32;

                        let col = match epi.epi {
                            EpiStat::S => &BLUE,
                            EpiStat::I => &RED,
                            EpiStat::R => &GREEN,
                        };
                        root.draw_pixel((x, y), col).unwrap();
                    }
                });
            }
            steps += 1;
        });
    sys
}
