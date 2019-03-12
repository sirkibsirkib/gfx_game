use specs::ReadStorage;
use crate::resources::GlobalTrans;
use crate::resources::InputControlling;
use crate::resources::MetaGameState;
use enum_map::enum_map as enum_map_make;
use enum_map::Enum as EnumMapDerive;
use enum_map::EnumMap;
use gfx_pp::high_level::poll_events_simple;
use gfx_pp::{glutin, glutin::EventsLoop, high_level::SimpleEvent};
use specs::{Write, Read, join::Join, System, WriteStorage};
use gfx_pp::low_level::Trans;

use crate::components::*;

mod render_system;
pub use render_system::RenderSystem;
// use crate::resources::*;

#[derive(Debug, Default)]
pub struct MovementSystem;
impl<'a> System<'a> for MovementSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (WriteStorage<'a, Position>, WriteStorage<'a, Velocity>);

    fn run(&mut self, (mut pos_store, mut vel_store): Self::SystemData) {
        // println!("MovementSystem");
        for (p, v) in (&mut pos_store, &mut vel_store).join() {
            p.0[0] += v.0[0];
            p.0[1] += v.0[1];
            v.0[0] = 0.;
            v.0[1] = 0.;
        }
    }
}
/////////////////////////////////////////////////

/////////////////////////////////////////////////
pub struct UserInputSystem {
    e: EventsLoop,
    holding_key: EnumMap<MoveDir, bool>,
}
impl UserInputSystem {
    pub fn new(e: EventsLoop) -> Self {
        let holding_key = enum_map_make! {
            MoveDir::Up => false,
            MoveDir::Down => false,
            MoveDir::Left => false,
            MoveDir::Right => false,
        };
        Self { e, holding_key }
    }
}
impl<'a> System<'a> for UserInputSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Option<Read<'a, InputControlling>>,
        WriteStorage<'a, Velocity>,
        Write<'a, MetaGameState>,
    );

    fn run(&mut self, (con, mut vel, mut meta): Self::SystemData) {
        let (e, holding_key) = (&mut self.e, &mut self.holding_key);
        for event in poll_events_simple(e) {
            match event {
                SimpleEvent::KeyPress(code) => {
                    if let Some(h) = Self::keycode_map(code) {
                        holding_key[h] = true;
                        // println!("holding_key[{:?}]={}", h, true);
                    } else if let glutin::VirtualKeyCode::Escape = code {
                        meta.running = false;
                    }
                }
                SimpleEvent::KeyRelease(code) => {
                    if let Some(h) = Self::keycode_map(code) {
                        holding_key[h] = false;
                        // println!("holding_key[{:?}]={}", h, false);
                    }
                }
            }
        }
        if let Some(e) = con {
            if let Some(v) = vel.get_mut(e.0) {
                use MoveDir::*;
                let speed = if (holding_key[Left] ^ holding_key[Right])
                    && (holding_key[Up] ^ holding_key[Down])
                {
                    1.0 / 2.0_f32.sqrt()
                } else {
                    1.0
                };
                if holding_key[Left] ^ holding_key[Right] {
                    if holding_key[Left] {
                        v.0[0] = -speed;
                    } else {
                        v.0[0] = speed;
                    }
                } else {
                    v.0[0] = 0.;
                }
                if holding_key[Up] ^ holding_key[Down] {
                    if holding_key[Up] {
                        v.0[1] = speed;
                    } else {
                        v.0[1] = -speed;
                    }
                } else {
                    v.0[1] = 0.;
                }
            }
        }
    }
}
impl UserInputSystem {
    fn keycode_map(code: glutin::VirtualKeyCode) -> Option<MoveDir> {
        match code {
            glutin::VirtualKeyCode::A => Some(MoveDir::Left),
            glutin::VirtualKeyCode::D => Some(MoveDir::Right),
            glutin::VirtualKeyCode::W => Some(MoveDir::Up),
            glutin::VirtualKeyCode::S => Some(MoveDir::Down),
            _ => None,
        }
    }
}

#[derive(EnumMapDerive, Debug, Copy, Clone)]
enum MoveDir {
    Up,
    Down,
    Left,
    Right,
}


pub struct CameraSystem;
impl<'a> System<'a> for CameraSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Option<Read<'a, InputControlling>>,
        ReadStorage<'a, Position>,
        Write<'a, GlobalTrans>,
    );

    fn run(&mut self, (con, pos, mut glo): Self::SystemData) {
        if let Some(some_con) = con {
            let e = some_con.0;
            if let Some(p) = pos.get(e) {
                let [x,y] = [p.0[0], p.0[1]];
                let gt = Trans::identity().scaled([2. / 512.; 2]).translated([-x,-y,0.]);
                glo.set_and_dirty(gt);
            }
        }
    }
}