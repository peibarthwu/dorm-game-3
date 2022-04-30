use frenderer::assets::TextureRef;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32,
}

impl std::ops::Add<Vec2i> for Vec2i {
    type Output = Self;

    fn add(self, other: Vec2i) -> <Self as std::ops::Add<Vec2i>>::Output {
        Vec2i {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
#[derive(Clone)]
pub enum GameObject {
    Key,
    Chair,
    LockedChest,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Clone)]
pub struct Room {
    pub doors: Vec<Door>,
    // pub floor: TextureRef,        //figure out the type for a texture
    // pub objects: Vec<GameObject>, //vec of game objects, perhaps including a key
}

impl Room {
    pub fn new(doors: Vec<Door>) -> Self {
        return Room {
            doors,
            // floor,
            // objects,
        };
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Door {
    pub direction: Direction,
    pub target: usize, //where it goes, Room
    pub spawn_pos: Direction, //which door you come from
}
impl Door {
    pub fn new(direction: Direction, target: usize, spawn_pos: Direction) -> Self {
        return Door { direction, target , spawn_pos};
    }
    // pub fn new(direction: Direction, target: usize) -> Self {
    //     Door::new(direction, target);
    // }
}
