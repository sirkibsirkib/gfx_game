use gfx_pp::low_level::TexRect;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use specs::world::Builder;
use specs::{DispatcherBuilder, World};

mod components;
mod resources;
mod systems;

fn main() {
    // create the world state
    let mut world = World::new();
    world.register::<components::Position>();

    // entangling things
    let pos_reader_for_render = world
        .write_storage::<components::Position>()
        .channel_mut()
        .register_reader();

    // provide resources
    world.add_resource({
        let mut x = resources::GlobalTrans::default();
        x.set_and_dirty(gfx_pp::low_level::Trans::identity().scaled([2. / 512.; 2]));
        x
    });

    let (g, e) = gfx_pp::low_level::build_window([512.0; 2], "game!".into(), true, 500);
    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::MovementSystem, "MovementSystem", &[])
        .with_thread_local(systems::RenderSystem::new(g, pos_reader_for_render))
        .with_thread_local(systems::UserInputSystem::new(e))
        .build();
    dispatcher.setup(&mut world.res);

    let mut rng = SmallRng::from_seed([0; 16]);


    // add grass
    world
        .create_entity()
        .with(components::Position([0., 0., 0.5]))
        .with(components::Sprite {
            scale: [100.0 * 3.0; 2],
            rot: 0.,
            tex_key: components::TexKey::Grass,
            tex_rect: TexRect {
                scale: [3.; 2],
                offset: [0.; 2],
            },
        })
        .with(components::UsuallyStationary)
        .build();

    // add trees
    for _ in 0..200 {
        use crate::components::*;
        let [x, y] = [rng.gen::<f32>() * 300.0, rng.gen::<f32>() * 300.0];
        let z = y / 9999.0;
        println!("tree z {:?}", z);
        let pos = [x, y, z];
        world
            .create_entity()
            .with(Position(pos))
            .with(Sprite {
                scale: [32., 28.],
                rot: 0.,
                tex_key: TexKey::Tree,
                tex_rect: TexRect::default(),
            })
            .with(UsuallyStationary)
            .build();
    }


    // begin the main loop
    let mut sleeper = gfx_pp::high_level::Sleeper::default();
    sleeper.min_sleep_time = std::time::Duration::from_millis(250);
    loop {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
        sleeper.mark_measure_sleep();
    }
}

// use gfx_pp::{glutin, high_level, low_level};
// use rand::{rngs::SmallRng, Rng, SeedableRng};
// use std::path::Path;
// use high_level::*;
// use low_level::*;

// #[derive(Debug, Copy, Clone, PartialEq, Eq)]
// enum Dir {
// 	Up, Down, Left, Right
// }
// struct State {
// 	running: bool,
// 	glob: Trans,
// 	dir: Option<Dir>
// }
// impl EventHandler for State {
// 	fn key_pressed(&mut self, code: glutin::VirtualKeyCode) {
// 		use glutin::VirtualKeyCode;
// 		 match code {
// 			VirtualKeyCode::Escape => {
// 				self.running = false
// 			},
//     		VirtualKeyCode::A => self.dir = Some(Dir::Left),
//     		VirtualKeyCode::D => self.dir = Some(Dir::Right),
//     		VirtualKeyCode::W => self.dir = Some(Dir::Up),
//     		VirtualKeyCode::S => self.dir = Some(Dir::Down),
//     		_ => {},
//     	}
// 	}

// 	fn key_released(&mut self, code: glutin::VirtualKeyCode) {
// 		use glutin::VirtualKeyCode;
// 		match code {
//     		VirtualKeyCode::W |
//     		VirtualKeyCode::A |
//     		VirtualKeyCode::S |
//     		VirtualKeyCode::D => self.dir = None,
//     		_ => {},
//     	}
// 	}
// 	fn close_requested(&mut self) {
// 		self.running = false;
// 	}
// 	fn mouse_wheel(&mut self, distance: f32) {
// 		let mul = if distance < 0.0 {0.98} else {1. / 0.98};
// 		self.glob *= Trans::scaler([mul, mul]);
// 	}
// }
// impl State {
// 	fn update(&mut self) {
// 		if let Some(d) = self.dir {
// 			let translation: Option<[f32;2]> = match d {
// 	    		Dir::Left  => Some([ 3.0,  0.0]),
// 	    		Dir::Right => Some([-3.0,  0.0]),
// 	    		Dir::Up    => Some([ 0.0, -3.0]),
// 	    		Dir::Down  => Some([ 0.0,  3.0]),
// 	    	};
// 	    	if let Some([x,y]) = translation {
// 	    		self.glob *= Trans::translator([x, y, 0.])
// 	    	}
// 		}
// 	}
// }

// fn main() {
//     // prepare whatever persistent game state I need
//     use high_level::colors::BLACK;
//     let mut rng = SmallRng::from_seed([0; 16]);

//     const NUM_TREES: u32 = 1000;

//     let (mut g, mut e) = build_window([512.0;2], "transtest".into(), true, NUM_TREES+1);
//     let tree_tex = g
//         .load_gpu_tex(Path::new("src/resources/liltrees.png"))
//         .expect("whoops");
//     let tree_tex = g
//         .load_gpu_tex(Path::new("src/resources/adventurer.png"))
//         .expect("whoops");

//     let mut
//     let grass_tex = {
//     	let mut tex = g
//         .load_gpu_tex(Path::new("src/resources/grass.png"))
//         .expect("whoops");
//         let sampler_info = Gfx::_tiley();
//         let sampler = g.new_sampler(sampler_info);
//         tex.set_sampler(sampler);
//         tex
//     };
//     let mut store = InstanceStorage::new(NUM_TREES as usize, 0);
//     let mut _batch = InstanceBatcher::new();
//     let _trees: Vec<_> = (0..NUM_TREES)
//         .map(|i| {
//             let x = rng.gen::<f32>() * 2000.0;
//             let y = rng.gen::<f32>() * 2000.0;
//             let z = y * 0.00001;
//             let mut args = DrawArgs::default()
//                 .with_scale([32., 28.])
//                 .with_pos([x, y, z]);
//             if i%2==0 {
//             	args.scale[0] *= -1.0;
//             }
//             let tr = TexRect::from_grid_texture_sizes(tree_tex.get_size(), [32,28],[i%5,0]);
//             let key = store.add((args, tr)).unwrap();
//             (args, key)
//         })
//         .collect();
//     let grass_param = {
//     	let mut args = DrawArgs::default();
//     	let mut tex_rect = TexRect::default();
//     	tex_rect.scale = [1_00.;2];
//     	args.scale = [100_00.;2];
//     	args.pos[2] = 1.0;
//     	InstanceDatum {
//     		trans: args.into(),
//     		tex_rect,
//     	}
//     };

//     let mut state = State {
//     	running: true,
//     	dir: None,
//     	glob: Trans::identity().scaled([2. / 512. ; 2] ),
//     };

//     let mut sleeper = Sleeper::default();
//     while state.running {
//         // handle glutin events. takes arbitrary time
//         e.poll_events(|event| handler_invoke(event, &mut state));

//         // update and render
//         state.update();

//         // let start = std::time::Instant::now();

//         g.clear_screen(BLACK);
//         g.clear_depth(1.0);
//         draw_singleton(&mut g, &grass_tex, &[grass_param.into()], NUM_TREES).unwrap();
//         g.set_global_trans(&state.glob);

//         store.commit(&mut g, 0).unwrap();
//         g.draw(&tree_tex, 0..NUM_TREES, None).unwrap();
//         // dbg!(start.elapsed());

//         g.finish_frame().unwrap();

//         // sleep if its been less than target UPS
//         sleeper.mark_measure_sleep();
//     }
// }
