pub trait Split {
    type Output;
    fn lo(&self) -> Self::Output;
    fn hi(&self) -> Self::Output;
    fn split(&self) -> (Self::Output, Self::Output);
}

impl Split for u16 {
    type Output = u8;

    fn lo(&self) -> Self::Output {
        *self as u8
    }
    fn hi(&self) -> Self::Output {
        (*self >> 8) as u8
    }
    fn split(&self) -> (Self::Output, Self::Output) {
        (self.hi(), self.lo())
    }
}

impl Split for u32 {
    type Output = u16;

    fn lo(&self) -> Self::Output {
        *self as u16
    }
    fn hi(&self) -> Self::Output {
        (*self >> 16) as u16
    }
    fn split(&self) -> (Self::Output, Self::Output) {
        (self.hi(), self.lo())
    }
}

impl Split for u64 {
    type Output = u32;

    fn lo(&self) -> Self::Output {
        *self as u32
    }
    fn hi(&self) -> Self::Output {
        (*self >> 32) as u32
    }
    fn split(&self) -> (Self::Output, Self::Output) {
        (self.hi(), self.lo())
    }
}

impl Split for i16 {
    type Output = u8;

    fn lo(&self) -> Self::Output {
        *self as u8
    }
    fn hi(&self) -> Self::Output {
        (*self >> 8) as u8
    }
    fn split(&self) -> (Self::Output, Self::Output) {
        (self.hi(), self.lo())
    }
}

impl Split for i32 {
    type Output = u16;

    fn lo(&self) -> Self::Output {
        *self as u16
    }
    fn hi(&self) -> Self::Output {
        (*self >> 16) as u16
    }
    fn split(&self) -> (Self::Output, Self::Output) {
        (self.hi(), self.lo())
    }
}

impl Split for i64 {
    type Output = u32;

    fn lo(&self) -> Self::Output {
        *self as u32
    }
    fn hi(&self) -> Self::Output {
        (*self >> 32) as u32
    }
    fn split(&self) -> (Self::Output, Self::Output) {
        (self.hi(), self.lo())
    }
}