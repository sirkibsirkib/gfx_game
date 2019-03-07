use gfx_pp::{glutin, high_level, low_level};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::path::Path;

fn catch_close(event: glutin::Event, running: &mut bool) {
    if let glutin::Event::WindowEvent { event, .. } = event {
        match event {
            glutin::WindowEvent::Closed
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

    const NUM_TREES: u32 = 300;

    let mut g = Gfx::new(512, 512, "transtest".to_string(), 300);
    let tree_tex = g
        .load_gpu_tex(Path::new("src/resources/liltrees.png"))
        .expect("whoops");
    let mut stage = TransformStage::new(NUM_TREES as usize, 0);
    let _trees: Vec<_> = (0..NUM_TREES)
        .map(|i| {
            let x = rng.gen::<f32>() * 2.0 - 1.0;
            let z = rng.gen::<f32>();
            let y = z * 2.0 - 1.0;
            let z = z * 0.0001;
            let args = DrawArgs::default()
                .with_scale([0.128, 0.108])
                .with_pos([x, y, z]);
            // .with_pos([]);
            let tr = TexRect::from_grid_texture_sizes(tree_tex.get_size(), [32,28],[i%5,0]);
            let key = stage.add((args, tr)).unwrap();
            (args, key)
        })
        .collect();
    let _args = DrawArgs::default().with_scale([0.32, 0.28]);

    // begin main game loop
    let mut sleeper = Sleeper::default();
    let mut running = true;
    while running {
        // handle glutin events. takes arbitrary time
        g.events_loop.poll_events(|e| catch_close(e, &mut running));

        // update and render
        g.clear_screen(BLACK);
        g.clear_depth(1.0);

        stage.commit(&mut g, 0).unwrap();
        g.draw(&tree_tex, 0..NUM_TREES, None).unwrap();

        g.finish_frame().unwrap();

        // sleep if its been less than target UPS
        sleeper.mark_measure_sleep();
    }
}
