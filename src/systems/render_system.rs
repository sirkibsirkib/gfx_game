use crate::components::{Position, Sprite, TexBatched, TexKey, UsuallyStationary};
use crate::resources::GlobalTrans;
use gfx_pp::{
    high_level::{DrawArgs, GrowBehavior, InstanceBatcher, InstanceStorage},
    low_level::{Gfx, GpuTexture, InstanceDatum, LoadError, SamplerInfo, Trans},
};
use hashbrown::HashMap;
use specs::prelude::ReaderId;
use specs::storage::ComponentEvent;
use specs::{
    join::Join, BitSet, Entities, LazyUpdate, Read, ReadStorage, System, Write, WriteStorage,
};
use std::ops::Range;
use std::path::Path;

pub struct RenderSystem {
    g: Gfx,
    textures: HashMap<TexKey, GpuTexture>,
    sprite_storages: HashMap<TexKey, InstanceStorage>,
    sprite_batcher: InstanceBatcher<TexKey>,
    temp_bitset: BitSet,
    pos_reader: ReaderId<ComponentEvent>,
    sprite_reader: ReaderId<ComponentEvent>,
}

impl RenderSystem {
    pub fn new(g: Gfx, pos_reader: ReaderId<ComponentEvent>, sprite_reader: ReaderId<ComponentEvent>) -> Self {
        Self {
            g,
            sprite_storages: HashMap::default(),
            sprite_batcher: InstanceBatcher::new(),
            textures: HashMap::default(),
            temp_bitset: BitSet::new(),
            pos_reader,
            sprite_reader,
        }
    }
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
            TexKey::Tree => Path::new("./src/resources/liltrees.png"),
            TexKey::Adventurer => Path::new("./src/resources/adventurer.png"),
        }
    }
    fn tex_sampler_info(key: TexKey) -> SamplerInfo {
        match key {
            TexKey::Grass => gfx_pp::high_level::new_sampler_info(true, false),
            TexKey::Tree => gfx_pp::high_level::new_sampler_info(true, true),
            TexKey::Adventurer => gfx_pp::high_level::new_sampler_info(true, true),
        }
    }
}
impl<'a> System<'a> for RenderSystem {
    #[allow(clippy::type_complexity)]
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
                .sprite_storages
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
                .sprite_storages
                .get_mut(&b.tex_key)
                .expect("none for key?");
            store.remove(b.store_key).expect("bad store key removal");
            upd.remove::<TexBatched>(e);
        }

        // 3: update batched entries whose sprites OR positions have been modified
        // TODO additions and removals
        {
            self.temp_bitset.clear();
            for event in pos.channel().read(&mut self.pos_reader)
            .chain(spri.channel().read(&mut self.sprite_reader)) {
                match event {
                    ComponentEvent::Modified(id) => {
                        println!("id={:?} modified", id);
                        self.temp_bitset.add(*id);
                    }
                    _ => {} // TODO
                };
            }
            for (b, p, sp, _) in (&bat, &pos, &spri, &self.temp_bitset).join() {
                let trans = Self::trans_from(p, sp);
                self.sprite_storages
                    .get_mut(&b.tex_key)
                    .expect("no key for modify")
                    .overwrite_trans_as_trans(b.store_key, trans)
                    .expect("hey");
            }
        }

        // 4: update global trans
        if let Some(trans) = glo.get_if_dirty_then_clean() {
            self.g.set_global_trans(&trans);
        }

        // 5: clear window
        self.g.clear_screen(gfx_pp::high_level::colors::BLACK);
        self.g.clear_depth(1.0);

        // 6: draw batch elements
        let mut avail: Range<u32> = 0..self.g.max_instances();
        {
            let RenderSystem {
                sprite_storages,
                textures,
                g,
                ..
            } = self;
            for (&tex_key, store) in sprite_storages.iter_mut() {
                let rng: Range<u32> = store.commit(g, avail.clone()).expect("NO SPACE");
                let texture = get_tex_for(g, textures, tex_key).expect("BAD batch TEX?");
                g.draw(texture, rng.clone(), None).expect("draw 1 failed");
                avail.start = avail.start.max(rng.end + 5);
            }
        };

        // 7: draw non-batched elements
        {
            for (p, sp, _, _) in (&pos, &spri, !&bat, !&stat).join() {
                let datum = InstanceDatum {
                    trans: Self::trans_from(p, sp),
                    tex_rect: sp.tex_rect,
                };
                self.sprite_batcher.add(sp.tex_key, datum);
            }
            let RenderSystem {
                sprite_batcher,
                textures,
                g,
                ..
            } = self;
            for (tex_key, slice) in sprite_batcher.iter_batches() {
                g.prepare_instances(slice, avail.start).expect("UPD8");
                let texture = get_tex_for(g, textures, tex_key).expect("BAD standalone TEX?");
                let draw_rng = avail.start..(avail.start + slice.len() as u32);
                g.draw(texture, draw_rng, None).expect("draw 2 failed");
            }
            sprite_batcher.clear(false);
        }

        // 8: finish
        self.g.finish_frame().unwrap();
    }
}

fn get_tex_for<'a, 'b>(
    g: &'a mut Gfx,
    textures: &'b mut HashMap<TexKey, GpuTexture>,
    tex_key: TexKey,
) -> Result<&'b GpuTexture, LoadError> {
    if !textures.contains_key(&tex_key) {
        let path = RenderSystem::tex_path(tex_key);
        let sampler_info = RenderSystem::tex_sampler_info(tex_key);
        let tex = g.load_gpu_tex(path, sampler_info)?;
        textures.insert(tex_key, tex);
    }
    Ok(textures.get(&tex_key).unwrap())
}
