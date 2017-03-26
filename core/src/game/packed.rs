
use game::util::*;
// use std;
use std::fmt;

use std::ops::{BitOr, BitOrAssign, BitAnd, BitAndAssign, BitXor, BitXorAssign, Not};
  
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Packed1(pub u32);

pub const PACKED1_EMPTY : Packed1 = Packed1(0);

pub const ONE_MASK : u32 = 1;
pub const ALL_MASK_32 : u32 = 0xffffffff;


impl Packed for Packed1 {
    fn empty() -> Packed1 {
        Packed1(0)
    }

    fn get(&self, slot : Slot) -> u8 {
        ((self.0 >> slot.0) & ONE_MASK) as u8
    }

    fn set(&self, slot : Slot, value: u8) -> Packed1 {
        let remove_mask : u32 = (1 << slot.0) ^ ALL_MASK_32;
        Packed1((self.0 & remove_mask) | ((value as u32) << slot.0))
    }
}

impl Packed1 {
     #[inline]
    pub fn bitscan(&self) -> Slot {
        Slot(self.0.trailing_zeros() as i8)
    }

    #[inline]
    fn lsb(&self) -> Packed1 {
        Packed1(self.0 & 0u32.wrapping_sub(self.0))
    }

    #[inline]
    pub fn iter(self) -> Packed1Iterator {
        Packed1Iterator(self)
    }

    fn setb(&mut self, slot : Slot, value: u8) {
        let remove_mask : u32 = (1 << slot.0) ^ ALL_MASK_32;
        self.0 = (self.0 & remove_mask) | ((value as u32) << slot.0);
    }
}

impl fmt::Debug for Packed1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Packed1(\n").unwrap();
        for y in 0..5 {
            f.write_str("  ").unwrap();
            for x in 0..5 {
                let on = self.get(Slot(y * 5 + x ));
                write!(f, "{}", on).unwrap();
            }
            f.write_str("\n").unwrap();
        }
        f.write_str(")\n")
    }
}

impl BitAnd for Packed1 {
    type Output = Packed1;

    fn bitand(self, other: Packed1) -> Packed1 {
        Packed1(self.0 & other.0)
    }
}

impl BitOr for Packed1 {
    type Output = Packed1;

    fn bitor(self, other: Packed1) -> Packed1 {
        Packed1(self.0 | other.0)
    }
}

impl BitXor for Packed1 {
    type Output = Packed1;

    fn bitxor(self, other: Packed1) -> Packed1 {
        Packed1(self.0 ^ other.0)
    }
}

impl BitAndAssign for Packed1 {
    fn bitand_assign(&mut self, other: Packed1) {
        self.0 &= other.0;
    }
}

impl BitOrAssign for Packed1 {
    fn bitor_assign(&mut self, other: Packed1) {
        self.0 |= other.0;
    }
}

impl Not for Packed1 {
    type Output = Packed1;

    fn not(self) -> Packed1 {
        Packed1(!self.0)
    }
}

impl BitXorAssign for Packed1 {
    fn bitxor_assign(&mut self, other: Packed1) {
        self.0 ^= other.0;
    }
}

pub struct Packed1Iterator(Packed1);

impl Iterator for Packed1Iterator {
    type Item = Slot;

    #[inline]
    fn next(&mut self) -> Option<Slot> {
        if (self.0).0 == PACKED1_EMPTY.0 {
            return None;
        }

        let sq = self.0.bitscan();
        let lsb = self.0.lsb();
        self.0 ^= lsb;
        Some(sq)
    }
}

#[cfg(test)]
mod tests {
    
    use super::*;

    // #[test]
    fn test_iter() {
        let slots = vec![Slot(2), Slot(4), Slot(13), Slot(15)];
        let mut p = PACKED1_EMPTY;
        for sl in slots {
            p.setb(sl, 1);
        }

        println!("ok, p -> {:?}", p);

        for sl in p.iter() {
            println!("slot -> {:?}", sl);
        }

    }
}
