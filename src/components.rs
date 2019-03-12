use gfx_pp::low_level::TexRect;
use specs::{Component, DenseVecStorage, FlaggedStorage, NullStorage, VecStorage};

use specs_derive::Component as DeriveComponent;

#[derive(DeriveComponent, Default, Debug)]
#[storage(NullStorage)]
pub struct TransDirty;

#[derive(Default, Debug)]
pub struct Position(pub [f32; 3]);
impl Component for Position {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}
impl Position {
	pub fn from_xy(xy: [f32; 2]) -> Self {
		Self([xy[0], xy[1], xy[1] * 0.000001])
	}
}

#[derive(DeriveComponent, Default)]
#[storage(DenseVecStorage)]
pub struct Velocity(pub [f32; 2]);


#[derive(Debug)]
pub struct Sprite {
    pub tex_key: TexKey,
    pub tex_rect: TexRect,
    pub scale: [f32; 2],
    pub rot: f32,
}
impl Component for Sprite {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum TexKey {
    Grass,
    Tree,
    Adventurer,
}

// CURRENTLY BATCHED by render system
#[derive(DeriveComponent)]
#[storage(DenseVecStorage)]
pub struct TexBatched {
    pub tex_key: TexKey,
    pub store_key: usize,
}

#[derive(DeriveComponent, Default)]
#[storage(NullStorage)]
pub struct UsuallyStationary;
