use easy_graph::ui::window::WindowBuilder;
use legion::prelude::*;
use legion::query::Read;
use legion::schedule::{Builder, Schedulable};
use legion::system::SystemBuilder;
use legion::world::Universe;
use plotters::drawing::bitmap_pixel::RGBPixel;
use plotters::element::Circle;
use plotters::prelude::*;
use plotters::style::{BLACK, WHITE};
use rand::{Rng, ThreadRng};
use std::time::Instant;

const SIZE: i32 = 500;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Pos {
    x: i32,
    y: i32,
}

fn main() {
    println!("ECS + BufferWindow example");
    let universe = Universe::new();
    let mut world = universe.create_world();
    //world.resources.insert( BufferWindow::new((SIZE as usize, SIZE as usize), None) );

    let mut schedule = Builder::default()
        .add_system(entity_creator_system(1_000))
        .add_system(entity_mover_system())
        .add_thread_local(draw_system(10))
        .build();

    let now = Instant::now();
    for tick in 0..100000 {
        if tick % 1000 == 0 {
            println!("TICK {}", tick);
        }
        schedule.execute(&mut world);
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:?}", elapsed);
}

fn entity_creator_system(num_entities: i32) -> Box<dyn Schedulable> {
    let mut initialized = false;
    let sys =
        SystemBuilder::<()>::new("Creator").build(move |commands, _world, _resource, _queries| {
            let mut rng: ThreadRng = rand::thread_rng();
            if !initialized {
                let entities: Vec<_> = (0..num_entities)
                    .map(|_| {
                        (Pos {
                            x: rng.gen_range(0, SIZE) as i32,
                            y: rng.gen_range(0, SIZE) as i32,
                        },)
                    })
                    .collect();
                commands.insert((), entities);
                initialized = true;
            }
        });
    sys
}

fn entity_mover_system() -> Box<dyn Schedulable> {
    let sys = SystemBuilder::<()>::new("Mover")
        .with_query(<Write<Pos>>::query())
        .build(move |_commands, world, _resource, queries| {
            let mut rng: ThreadRng = rand::thread_rng();
            for (_entity, mut pos) in queries.iter_entities(&mut *world) {
                let px = pos.x + if rng.gen_bool(0.5) { 1 } else { -1 };
                let py = pos.y + if rng.gen_bool(0.5) { 1 } else { -1 };
                if px >= 0 && px < SIZE && py >= 0 && py < SIZE {
                    pos.x = px;
                    pos.y = py;
                }
            }
        });
    sys
}

fn draw_system(step: u32) -> Box<dyn Runnable> {
    let mut steps = 0;
    let mut win = WindowBuilder::new()
        .with_dimensions(SIZE as usize, SIZE as usize)
        .with_title("Test")
        .build();
    let sys = SystemBuilder::<()>::new("Drawer")
        .with_query(<Read<Pos>>::query())
        //.write_resource::<BufferWindow>(  )
        .build_thread_local(move |_commands, world, _resource, queries| {
            if win.is_open() && (step == 0 || steps % step == 0) {
                win.draw(|b: BitMapBackend<RGBPixel>| {
                    let root = b.into_drawing_area();
                    root.fill(&WHITE).unwrap();
                    for (_entity, pos) in queries.iter_entities(&mut *world) {
                        let mut style = ShapeStyle::from(&BLACK);
                        style.filled = true;
                        let circle = Circle::new((pos.x, pos.y), 1, style);
                        root.draw(&circle).unwrap();
                    }
                });
            }
            steps += 1;
        });
    sys
}
