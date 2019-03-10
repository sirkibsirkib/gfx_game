use gfx_pp::low_level::Trans;
use specs::Entity;

#[derive(Debug)]
pub struct GlobalTrans {
    trans: Trans,
    dirty: bool,
}
impl Default for GlobalTrans {
    fn default() -> Self {
        Self {
            trans: Trans::identity(),
            dirty: true,
        }
    }
}
impl GlobalTrans {
    pub fn peek(&self) -> Trans {
        self.trans
    }
    pub fn get_if_dirty_then_clean(&mut self) -> Option<&Trans> {
        if self.dirty {
            self.dirty = false;
            Some(&self.trans)
        } else {
            None
        }
    }
    pub fn set_and_dirty(&mut self, new_value: Trans) {
        self.trans = new_value;
        self.dirty = true;
    }
}

#[derive(Debug)]
pub struct InputControlling(pub Entity);
