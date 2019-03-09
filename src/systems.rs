use specs::prelude::ReaderId;
use specs::BitSet;
use crate::components::TreeBatchKey;
use specs::LazyUpdate;
use specs::Entities;
use specs::Read;
use crate::resources::InputControlling;
use gfx_pp::high_level::poll_events_simple;
use crate::resources::GlobalTrans;
use specs::Write;
use gfx_pp::high_level::GrowBehavior;
use specs::{System, WriteStorage, join::Join, ReadStorage};
use gfx_pp::{
	glutin,
	low_level::{Gfx, GpuTexture, InstanceDatum, TexRect, Trans},
	high_level::{InstanceStorage, DrawArgs, SimpleEvent},
	glutin::EventsLoop,
};
use std::path::Path;
use enum_map::Enum as EnumMapDerive;
use enum_map::enum_map as enum_map_make;
use enum_map::EnumMap;
use specs::storage::ComponentEvent;

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
	trees: InstanceStorage,
	grass_tex: GpuTexture, 
	tree_tex: GpuTexture, 
    dirty_trees: BitSet,
    reader_id: Option<ReaderId<ComponentEvent>>,
	grass_datum: InstanceDatum,
	g: Gfx,
}
impl RenderSystem {
	const TREE_CAPACITY: usize = 300;

	pub fn new(mut g: Gfx) -> Self {
		let grass_tex = {
			let mut tex = g
		        .load_gpu_tex(TexItem::Grass.path())
		        .expect("whoops");
	        let sampler_info = Gfx::_tiley();
	        let sampler = g.new_sampler(sampler_info);
	        tex.set_sampler(sampler);
	        tex
	    };
		let tree_tex = g
		        .load_gpu_tex(TexItem::Tree.path())
		        .expect("whoops");
	    let grass_datum = {
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
		Self {
			g,
			grass_tex,
			tree_tex,
			dirty_trees: BitSet::new(),
			grass_datum,
			reader_id: None,
			trees: InstanceStorage::new(Self::TREE_CAPACITY, GrowBehavior::Doubling, 0),
		}
	}
}
impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        Entities<'a>,
    	Write<'a, GlobalTrans>,
    	WriteStorage<'a, Position>,
    	ReadStorage<'a, TreeBatchKey>,
        Read<'a, LazyUpdate>,
    );
    fn run(&mut self, (ent, mut glo, mut pos, key, upd): Self::SystemData) {
    	// [grass][trees]
    	//  0      1.. 

    	// add new keys for trees
    	for (e, p, _) in (&ent, &pos, !&key).join() {
    		println!("ADDING TREE");
    		let args = DrawArgs {
    			pos: [p.0[0], p.0[1], 0.5],
    			scale: [32.0, 28.0],
    			..Default::default()
    		};
    		let key = self.trees.add((args, TexRect::default())).expect("NO SPACE FOR TREE");
    		upd.insert(e, TreeBatchKey(key));
    	}

    	// update trees that have been modified
    	self.dirty_trees.clear();
    	if let Some(ref mut r) = self.reader_id {
	        for event in pos.channel().read(r) {
	            match event {
	                ComponentEvent::Modified(id) => {
	                    self.dirty_trees.add(*id);
	                }
	                _ => {}, // TODO
	            }
	        }
	    	for (k, _p, _) in (&key, &pos, &self.dirty_trees).join() {
	    		let trans = Trans::identity(); // TODO
	    		self.trees.overwrite_trans_as_trans(k.0, trans).expect("hey");
			}
    	} else {
    		println!("REGISTERED");
    		self.reader_id = Some(pos.channel_mut().register_reader());
    	}
    	

    	use gfx_pp::high_level::*;
    	use gfx_pp::high_level::colors::BLACK;

    	if let Some(trans) = glo.get_if_dirty_then_clean() {
    		self.g.set_global_trans(&trans);
    	}

        self.g.clear_screen(BLACK);
        self.g.clear_depth(1.0);
        draw_singleton(&mut self.g, &self.grass_tex, &[self.grass_datum], 0).expect("FAM");
        
        let mi = self.g.get_max_instances();
        let dun = self.trees.commit(&mut self.g, 1..mi).unwrap();
        println!("DUN {:?}", dun);
        self.g.draw(&self.tree_tex, 1..(self.trees.len() as u32 + 1), None).unwrap();
        println!("self.trees.len() {:?}", self.trees.len());
        self.g.finish_frame().unwrap();

    }
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
enum TexItem {
	Grass, Tree,
}
impl TexItem {
	fn path(&self) -> &'static Path {
		match self {
			TexItem::Grass => Path::new("./src/resources/grass.png"),
			TexItem::Tree => Path::new("./src/resources/liltree.png"),
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
		Self {
			e,
			holding_key,
		}
	}
}
impl<'a> System<'a> for UserInputSystem {
    type SystemData = (Option<Read<'a, InputControlling>>, WriteStorage<'a, Velocity>);

    fn run(&mut self, (con, mut vel): Self::SystemData) {
    	// println!("UserInputSystem");
    	let (e, holding_key) = (&mut self.e, &mut self.holding_key);
    	for event in poll_events_simple(e) {
    		match event {
    			SimpleEvent::KeyPress(code) => {
    				if let Some(h) = Self::keycode_map(code) {
    					holding_key[h] = true;
    				} else if let glutin::VirtualKeyCode::A = code {
    					println!("ESCAPE PRESSED");
    				}
    			},
    			SimpleEvent::KeyRelease(code) => {
    				if let Some(h) = Self::keycode_map(code) {
    					holding_key[h] = false;
    				}
    			},
    		}
    	}
    	if let Some(e) = con {
    		if let Some(v) = vel.get_mut(e.0) {
    			use MoveDir::*;
    			let speed = if (holding_key[Left] ^ holding_key[Right]) && (holding_key[Up] ^ holding_key[Down]) {
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
	Up, Down, Left, Right,
}