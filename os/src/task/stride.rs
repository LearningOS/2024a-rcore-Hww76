// use core::cmp::Ordering;

#[derive(Clone, Copy)]
/// stride
pub struct Stride(usize);

// impl PartialOrd for Stride {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         // ...
//     }
// }

// impl PartialEq for Stride {
//     fn eq(&self, other: &Self) -> bool {
//         false
//     }
// }

impl Stride {
    /// new a stride , start of 0
    pub fn new() -> Self{
        Self(0)
    }

    /// update stride
    pub fn update(&mut self,pass: usize) {
        self.0 += pass;
    }

    /// get stride
    pub fn get_stride(&self) -> usize{
        self.0
    }
}

pub const BIG_STRIDE:usize = 255;