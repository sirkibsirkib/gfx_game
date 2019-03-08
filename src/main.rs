use gfx_pp::{glutin, high_level, low_level};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::path::Path;

fn catch_close(event: glutin::Event, running: &mut bool) {
    if let glutin::Event::WindowEvent { event, .. } = event {
        match event {
            glutin::WindowEvent::CloseRequested
            | glutin::WindowEvent::KeyboardInput {
                input:
                    glutin::KeyboardInput {
                        virtual_keycode: Some(glutin::VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *running = false,
            _ => {}
        }
    }
}

fn main() {
    // prepare whatever persistent game state I need
    use high_level::colors::BLACK;
    use high_level::*;
    use low_level::*;
    let mut rng = SmallRng::from_seed([0; 16]);

    const NUM_TREES: u32 = 1000;

    let (mut g, mut e) = build_window([512.0;2], "transtest".into(), true, NUM_TREES+1);
    let tree_tex = g
        .load_gpu_tex(Path::new("src/resources/liltrees.png"))
        .expect("whoops");
    let grass_tex = {
    	let mut tex = g
        .load_gpu_tex(Path::new("src/resources/grass.png"))
        .expect("whoops");
        let sampler_info = Gfx::_tiley();
        let sampler = g.new_sampler(sampler_info);
        tex.set_sampler(sampler);
        tex
    };
    let mut store = InstanceStorage::new(NUM_TREES as usize, 0);
    let mut _batch = InstanceBatcher::new();
    let _trees: Vec<_> = (0..NUM_TREES)
        .map(|i| {
            let x = rng.gen::<f32>() * 2000.0;
            let y = rng.gen::<f32>() * 2000.0;
            let z = 1. - y / 99999.0;
            let mut args = DrawArgs::default()
                .with_scale([32., 28.])
                .with_pos([x, y, z]);
            if i%2==0 {
            	args.scale[0] *= -1.0;
            }
            let tr = TexRect::from_grid_texture_sizes(tree_tex.get_size(), [32,28],[i%5,0]);
            let key = store.add((args, tr)).unwrap();
            (args, key)
        })
        .collect();
    let grass_param = {
    	let mut args = DrawArgs::default();
    	let mut tex_rect = TexRect::default();
    	tex_rect.scale = [1_00.;2];
    	args.scale = [100_00.;2];
    	args.pos[2] = 1.0;
    	InstanceDatum {
    		trans: args.into(),
    		tex_rect,
    	}
    };

    // begin main game loop
    let mut sleeper = Sleeper::default();
    let mut running = true;
    let mut glob = Trans::identity().scaled([2. / 512. ; 2] ).translated([-512.0, -512.0, 0.0]);
    let mut dir = None;
    while running {
        // handle glutin events. takes arbitrary time
        e.poll_events(|event| foo(event, &mut running, &mut dir));

        // update and render
        if let Some(d) = dir {
        	let [x,y] = match d {
        		Dir::Left => 	[ 3.0,  0.0],
        		Dir::Right => 	[-3.0,  0.0],
        		Dir::Up => 		[ 0.0, -3.0],
        		Dir::Down => 	[ 0.0,  3.0],
        	};
        	glob *= Trans::translator([x, y, 0.0]);
        }

        let start = std::time::Instant::now();
        g.clear_screen(BLACK);

        g.clear_depth(1.0);
        draw_singleton(&mut g, &grass_tex, &[grass_param.into()], NUM_TREES).unwrap();
        g.set_global_trans(&glob);

        store.commit(&mut g, 0).unwrap();
        g.draw(&tree_tex, 0..NUM_TREES, None).unwrap();
        dbg!(start.elapsed());

        g.finish_frame().unwrap();

        // sleep if its been less than target UPS
        sleeper.mark_measure_sleep();
    }
}

#[derive(Debug, Copy, Clone)]
enum Dir {
	Left, Right, Up, Down,
}

fn foo(event: glutin::Event, running: &mut bool, dir: &mut Option<Dir>) {
	use glutin::ElementState;
	if let glutin::Event::WindowEvent { event, .. } = event {
        match event {
            glutin::WindowEvent::CloseRequested
            | glutin::WindowEvent::KeyboardInput {
                input:
                    glutin::KeyboardInput {
                        virtual_keycode: Some(glutin::VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *running = false,
            glutin::WindowEvent::KeyboardInput {
                input:
                    glutin::KeyboardInput {
                        virtual_keycode: Some(glutin::VirtualKeyCode::A),
                        state: whee,
                        ..
                    },
                ..
            } => *dir = if whee==ElementState::Pressed {Some(Dir::Left)} else {None},
            glutin::WindowEvent::KeyboardInput {
                input:
                    glutin::KeyboardInput {
                        virtual_keycode: Some(glutin::VirtualKeyCode::D),
                        state: whee,
                        ..
                    },
                ..
            } => *dir = if whee==ElementState::Pressed {Some(Dir::Right)} else {None},
            glutin::WindowEvent::KeyboardInput {
                input:
                    glutin::KeyboardInput {
                        virtual_keycode: Some(glutin::VirtualKeyCode::W),
                        state: whee,
                        ..
                    },
                ..
            } => *dir = if whee==ElementState::Pressed {Some(Dir::Up)} else {None},
            glutin::WindowEvent::KeyboardInput {
                input:
                    glutin::KeyboardInput {
                        virtual_keycode: Some(glutin::VirtualKeyCode::S),
                        state: whee,
                        ..
                    },
                ..
            } => *dir = if whee==ElementState::Pressed {Some(Dir::Down)} else {None},
            _ => (),
        }
    }
}