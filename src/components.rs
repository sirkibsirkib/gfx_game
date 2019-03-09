use specs::{Component, NullStorage, DenseVecStorage, FlaggedStorage};

use specs_derive::Component as DeriveComponent;


#[derive(DeriveComponent, Default, Debug)]
#[storage(NullStorage)]
pub struct TransDirty;


#[derive(Default, Debug)]
pub struct Position(pub [f32;2]);
impl Component for Position {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}


#[derive(DeriveComponent, Default)]
#[storage(DenseVecStorage)]
pub struct Velocity(pub [f32;2]);



#[derive(DeriveComponent)]
#[storage(DenseVecStorage)]
pub struct TreeBatchKey(pub usize);



#[derive(DeriveComponent, Default)]
#[storage(NullStorage)]
pub struct IsTree;