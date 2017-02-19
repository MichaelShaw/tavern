use std::fmt;
use std;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Player(pub u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Slot(pub u8);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Packed1(pub u32);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Position {
    pub x : u8,
    pub y : u8,
}

impl Position {
    pub fn from(x: f64, y: f64) -> Option<Position> {
        if 0.0 <= x && x < 255.0 && 0.0 <= y && y < 255.0 {
            Some(Position {
                x: x as u8,
                y: y as u8,
            })    
        } else {
            None
        }
    }
}

pub trait Packed where Self: std::marker::Sized {
    fn empty() -> Self;
    fn get(&self, slot: Slot) -> u8;
    fn set(&self, slot: Slot, value: u8) -> Self;
}

pub const ONE_MASK : u32 = 1;

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

pub const TWO_MASK : u64 = 3;
pub const ALL_MASK_64 : u64 = 0xffffffffffffffff;
pub const ALL_MASK_32 : u32 = 0xffffffff;


#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Packed2(pub u64);

impl Packed for Packed2 {
    fn empty() -> Packed2 {
        Packed2(0)
    }
 
    fn get(&self, slot: Slot) -> u8 {
        ((self.0 >> (slot.0 * 2)) & TWO_MASK) as u8
    }

    fn set(&self, slot:Slot, value: u8) -> Packed2 {
        let remove_mask : u64 = (3 << (slot.0 * 2)) ^ ALL_MASK_64;
        Packed2((self.0 & remove_mask) | ((value as u64) << (slot.0*2)))
    }
}

impl fmt::Debug for Packed2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Packed2(\n").unwrap();
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