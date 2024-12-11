use std::num::TryFromIntError;
use glam::{IVec2, UVec2};

pub trait IntoIVec2 {
    fn into_ivec2(&self) -> IVec2;
}

impl IntoIVec2 for UVec2 {
    // TODO: this should realistically return Result,
    //   but since it's likely none of the AOC problems will overflow i32
    //   we can probably get away with this for keeping it simple
    fn into_ivec2(&self) -> IVec2 {
        let error = |item: u32| move |err: TryFromIntError| panic!("failed to cast u32({item}) in {self:?} to i32: {err:?}");
        let x = i32::try_from(self.x).unwrap_or_else(error(self.x));
        let y = i32::try_from(self.y).unwrap_or_else(error(self.y));
        IVec2::new(x, y)
    }
}

pub trait TryIntoUVec2 {
    type Error;
    fn try_into_uvec2(&self) -> Result<UVec2, Self::Error>;
}

impl TryIntoUVec2 for IVec2 {
    type Error = TryFromIntError;
    fn try_into_uvec2(&self) -> Result<UVec2, Self::Error> {
        let x = u32::try_from(self.x)?;
        let y = u32::try_from(self.y)?;
        Ok(UVec2::new(x, y))
    }
}

pub trait IntoUsizeTuple {
    fn into_usize_tuple(&self) -> (usize, usize);
}

impl IntoUsizeTuple for UVec2 {
    fn into_usize_tuple(&self) -> (usize, usize) {
        (self.x as usize, self.y as usize)
    }
}
