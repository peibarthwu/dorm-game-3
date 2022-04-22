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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rect {
    pub pos: Vec2i,
    pub sz: Vec2i,
}

impl Rect {
    pub fn contains(&self, other: Rect) -> bool {
        let br = self.pos + self.sz;
        let obr = other.pos + other.sz;
        self.pos.x <= other.pos.x && self.pos.y <= other.pos.y && obr.x <= br.x && obr.y <= br.y
    }

    pub fn touches(&self, other: Rect) -> bool {
        // r1 left is left of r2 right
        self.pos.x <= other.pos.x+other.sz.x as i32 &&
            // r2 left is left of r1 right
            other.pos.x <= self.pos.x+self.sz.x as i32 &&
            // those two conditions handle the x axis overlap;
            // the next two do the same for the y axis:
            self.pos.y <= other.pos.y+other.sz.y as i32 &&
            other.pos.y <= self.pos.y+self.sz.y as i32
    }

    pub fn rect_displacement(&self, r2:Rect) -> Option<(i32,i32)> {
        // Draw this out on paper to double check, but these quantities
        // will both be positive exactly when the conditions in rect_touching are true.
        let x_overlap = (self.pos.x+self.sz.x as i32).min(r2.pos.x+r2.sz.x as i32) - self.pos.x.max(r2.pos.y);
        let y_overlap = (self.pos.y+self.sz.y as i32).min(r2.pos.y+r2.sz.y as i32) - self.pos.y.max(r2.pos.y);
        if x_overlap >= 0 && y_overlap >= 0 {
            // This will return the magnitude of overlap in each axis.
            Some((x_overlap, y_overlap))
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);

unsafe impl vulkano::format::Pixel for Color {
    fn ensure_accepts(
        format: vulkano::format::Format,
    ) -> std::result::Result<(), vulkano::format::IncompatiblePixelsType> {
        // TODO: Be more strict: accept only if the format has a matching AcceptsPixels impl.
        if format.size().map_or(false, |x| {
            x % std::mem::size_of::<Self>() as vulkano::DeviceSize == 0
        }) {
            Ok(())
        } else {
            Err(vulkano::format::IncompatiblePixelsType)
        }
    }
    fn rate(format: vulkano::format::Format) -> u32 {
        (format.size().expect("this format cannot accept pixels")
            / std::mem::size_of::<Self>() as vulkano::DeviceSize) as u32
    }
}
