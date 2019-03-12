use crate::resources::MetaGameState;
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
    world.register::<components::Sprite>();

    // entangling things
    let pos_reader_for_render = world
        .write_storage::<components::Position>()
        .channel_mut()
        .register_reader();
    let sprite_reader_for_render = world
        .write_storage::<components::Sprite>()
        .channel_mut()
        .register_reader();

    // provide resources
    world.add_resource({
        let mut x = resources::GlobalTrans::default();
        x.set_and_dirty(gfx_pp::low_level::Trans::identity().scaled([2. / 512.; 2]));
        x
    });
    world.add_resource(resources::MetaGameState::default());

    let (g, e) = gfx_pp::low_level::build_window([512.0; 2], "game!".into(), true, 500);
    let builder = DispatcherBuilder::new()
        .with_thread_local(systems::UserInputSystem::new(e))
        .with(systems::MovementSystem, "MovementSystem", &[])
        .with(systems::CameraSystem, "CameraSystem", &[])
        .with_thread_local(systems::RenderSystem::new(g, pos_reader_for_render, sprite_reader_for_render));
    builder.print_par_seq();
    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world.res);

    let mut rng = SmallRng::from_seed([0; 16]);

    // add grass
    world
        .create_entity()
        .with(components::Position([0., 0., 0.5]))
        .with(components::Sprite {
            scale: [100.0 * 30.0; 2],
            rot: 0.,
            tex_key: components::TexKey::Grass,
            tex_rect: TexRect {
                scale: [30.; 2],
                offset: [0.; 2],
            },
        })
        .with(components::UsuallyStationary)
        .build();


    let e = world
        .create_entity()
        .with(components::Position::from_xy([50.0, 50.0]))
        .with(components::Velocity::default())
        .with(components::Sprite {
            scale: [32.0, 37.0],
            rot: 0.,
            tex_key: components::TexKey::Adventurer,
            tex_rect: TexRect::from_grid_texture_sizes([385,592], [32,37], [0,0]),
        })
        .build();
    world.add_resource(resources::InputControlling(e));

    // add trees
    for i in 0..200 {
        use crate::components::*;
        let [x, y] = [rng.gen::<f32>() * 2000.0, rng.gen::<f32>() * 2000.0];
        world
            .create_entity()
            .with(Position::from_xy([x, y]))
            .with(Sprite {
                scale: [32., 28.],
                rot: 0.,
                tex_key: TexKey::Tree,
                tex_rect: TexRect::from_grid_texture_sizes([160,28], [32,28], [i%5,0]),
            })
            .with(UsuallyStationary)
            .build();
    }

    // begin the main loop
    let mut sleeper = gfx_pp::high_level::Sleeper::default();
    sleeper.min_sleep_time = std::time::Duration::from_millis(16);
    while world.read_resource::<MetaGameState>().running {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
        sleeper.mark_measure_sleep();
    }
}
