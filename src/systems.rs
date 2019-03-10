use crate::resources::MetaGameState;
use crate::resources::GlobalTrans;
use crate::resources::InputControlling;
use enum_map::enum_map as enum_map_make;
use enum_map::Enum as EnumMapDerive;
use enum_map::EnumMap;
use hashbrown::HashMap;
use gfx_pp::high_level::poll_events_simple;
use gfx_pp::high_level::GrowBehavior;
use gfx_pp::low_level::SamplerInfo;
use gfx_pp::{
    glutin,
    glutin::EventsLoop,
    high_level::{DrawArgs, InstanceStorage, SimpleEvent},
    low_level::{Gfx, GpuTexture, InstanceDatum, Trans},
};
use specs::prelude::ReaderId;
use specs::storage::ComponentEvent;
use specs::BitSet;
use specs::Entities;
use specs::LazyUpdate;
use specs::Read;
use specs::Write;
use specs::{join::Join, ReadStorage, System, WriteStorage};
use std::ops::Range;
use std::path::Path;

use crate::components::*;
// use crate::resources::*;

#[derive(Debug, Default)]
pub struct MovementSystem;
impl<'a> System<'a> for MovementSystem {
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
pub struct RenderSystem {
    g: Gfx,
    textures: HashMap<TexKey, GpuTexture>,
    sprite_batches: HashMap<TexKey, InstanceStorage>,
    temp_bitset: BitSet,
    pos_reader: ReaderId<ComponentEvent>,
}
impl RenderSystem {
    pub fn new(g: Gfx, pos_reader: ReaderId<ComponentEvent>) -> Self {
        Self {
            g,
            sprite_batches: HashMap::default(),
            textures: HashMap::default(),
            temp_bitset: BitSet::new(),
            pos_reader,
        }
    }
}
impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, GlobalTrans>,
        WriteStorage<'a, TexBatched>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Sprite>,
        ReadStorage<'a, UsuallyStationary>,
        Read<'a, LazyUpdate>,
    );
    fn run(&mut self, (ent, mut glo, bat, pos, spri, stat, upd): Self::SystemData) {
        // 1: create batch elements for everything STATIONARY but not BATCHED
        for (e, p, sp, _, _) in (&ent, &pos, &spri, &stat, !&bat).join() {
            let tex_key = sp.tex_key;
            let store = &mut self
                .sprite_batches
                .entry(tex_key)
                .or_insert_with(|| InstanceStorage::new(8, GrowBehavior::Doubling, 3));
            let datum = InstanceDatum {
                trans: Self::trans_from(p, sp),
                tex_rect: sp.tex_rect,
            };
            let store_key = store.add_datum(datum).expect("NO SPACE???");
            // println!("adding new thng to batch {:?} {:?}", tex_key, store_key);
            upd.insert(e, TexBatched { tex_key, store_key });
        }

        // 2: remove batch elements that are not STATIONARY
        for (e, b, _) in (&ent, &bat, !&stat).join() {
            let store = self
                .sprite_batches
                .get_mut(&b.tex_key)
                .expect("none for key?");
            store.remove(b.store_key).expect("bad store key removal");
            upd.remove::<TexBatched>(e);
        }

        // 3: update batche entries whose sprites OR positions have changed
        {
            self.temp_bitset.clear();
            for event in pos.channel().read(&mut self.pos_reader) {
                match event {
                    ComponentEvent::Modified(id) => {
                        self.temp_bitset.add(*id);
                    }
                    _ => {} // TODO
                };
            }
            for (b, p, sp, _) in (&bat, &pos, &spri, &self.temp_bitset).join() {
                let trans = Self::trans_from(p, sp);
                self.sprite_batches
                    .get_mut(&b.tex_key)
                    .expect("no key for modify")
                    .overwrite_trans_as_trans(b.store_key, trans)
                    .expect("hey");
            }
        }

        // TODO

        // 4: update global trans
        if let Some(trans) = glo.get_if_dirty_then_clean() {
            self.g.set_global_trans(&trans);
        }

        // 5: clear window
        self.g.clear_screen(gfx_pp::high_level::colors::BLACK);
        self.g.clear_depth(1.0);

        // 6: draw batch elements
        {
            let mut avail: Range<u32> = 0..self.g.max_instances();
            for (&tex_key, store) in self.sprite_batches.iter_mut() {
                let rng: Range<u32> = store.commit(&mut self.g, avail.clone()).expect("NO SPACE");
                let texture = {
                    let RenderSystem {
                        ref mut textures,
                        ref mut g,
                        ..
                    } = self;
                    &textures.entry(tex_key).or_insert_with(|| {
                        let path = Self::tex_path(tex_key);
                        let sampler_info = Self::tex_sampler_info(tex_key);
                        g.load_gpu_tex(path, sampler_info).expect("BAD LOAD?")
                    })
                };
                self.g.draw(texture, rng.clone(), None).unwrap();
                avail.start = avail.start.max(rng.end + 5);
            }
        }
        self.g.finish_frame().unwrap();
    }
}

impl RenderSystem {
    fn trans_from(pos: &Position, sprite: &Sprite) -> Trans {
        let args = DrawArgs {
            pos: pos.0,
            scale: sprite.scale,
            rot: sprite.rot,
            origin: [0.; 2],
        };
        args.into()
    }
    fn tex_path(key: TexKey) -> &'static Path {
        match key {
            TexKey::Grass => Path::new("./src/resources/grass.png"),
            TexKey::Tree => Path::new("./src/resources/liltree.png"),
        }
    }
    fn tex_sampler_info(key: TexKey) -> SamplerInfo {
        match key {
            TexKey::Grass => gfx_pp::high_level::new_sampler_info(true, false),
            TexKey::Tree => gfx_pp::high_level::new_sampler_info(true, true),
        }
    }
}

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
                    } else if let glutin::VirtualKeyCode::Escape = code {
                        meta.running = false;
                    }
                }
                SimpleEvent::KeyRelease(code) => {
                    if let Some(h) = Self::keycode_map(code) {
                        holding_key[h] = false;
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
                }
                if holding_key[Up] ^ holding_key[Down] {
                    if holding_key[Up] {
                        v.0[1] = -speed;
                    } else {
                        v.0[1] = speed;
                    }
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

#[derive(EnumMapDerive)]
enum MoveDir {
    Up,
    Down,
    Left,
    Right,
}
