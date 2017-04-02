use std::fmt;
use std;

use aphid::HashSet;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Player(pub i8);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Slot(pub i8);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SlotTransform {
    pub slots : [Slot; 25]
}

impl SlotTransform {
    pub fn check(&self) -> bool {
        let mut slots : HashSet<Slot> = HashSet::default();
        for sl in &self.slots {
            slots.insert(*sl);
        }
        slots.len() == 25
    }
}

pub trait Packed where Self: std::marker::Sized {
    fn empty() -> Self;
    fn get(&self, slot: Slot) -> u8;
    fn set(&mut self, slot: Slot, value: u8);
}

pub const EMPTY_SLOT_TRANSFORM : SlotTransform = SlotTransform { slots: [Slot(0) ; 25] };

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Position {
    pub x : i8,
    pub y : i8,
}

pub fn pos(x:i8, y: i8) -> Position {
    Position {
        x: x,
        y: y,
    }
}

impl Position {
    pub fn from(x: f64, y: f64) -> Option<Position> {
        if 0.0 <= x && x < 255.0 && 0.0 <= y && y < 255.0 {
            Some(Position {
                x: x as i8,
                y: y as i8,
            })    
        } else {
            None
        }
    }
}

impl Mul<i8> for Position {
    type Output = Position;

    fn mul(self, rhs:i8) -> Position {
        Position {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

use std::ops::Add;

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Position {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Transform {
    pub m00: i8,
    pub m01: i8,
    pub m10: i8,
    pub m11: i8,
}

use std::ops::Mul;

impl Mul<Position> for Transform {
    type Output = Position;

    fn mul(self, rhs: Position) -> Position {
        Position {
            x: self.m00*rhs.x + self.m10*rhs.y,
            y: self.m01*rhs.x + self.m11*rhs.y,
        }
    }
}

impl Mul<Transform> for Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Transform {
        Transform {
            m00: self.m00*rhs.m00 + self.m10*rhs.m01,
            m01: self.m01*rhs.m00 + self.m11*rhs.m01,
            m10: self.m00*rhs.m10 + self.m10*rhs.m11,
            m11: self.m01*rhs.m10 + self.m11*rhs.m11,
        }
    }
}

pub const ROTATE_90 : Transform = Transform { m00:0, m01:1, m10:-1, m11:0 };
pub const ROTATE_180 : Transform = Transform { m00: -1, m01: 0, m10: 0, m11: -1};
pub const ROTATE_270 : Transform = Transform { m00: 0, m01: -1, m10: 1, m11: 0};


pub const PACKED2_EMPTY : Packed2 = Packed2(0);

pub const TWO_MASK : u64 = 3;
pub const ALL_MASK_64 : u64 = 0xffffffffffffffff;


#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Packed2(pub u64);

impl Packed for Packed2 {
    fn empty() -> Packed2 {
        Packed2(0)
    }
 
    fn get(&self, slot: Slot) -> u8 {
        ((self.0 >> (slot.0 * 2)) & TWO_MASK) as u8
    }

    fn set(&mut self, slot:Slot, value: u8) {
        let remove_mask : u64 = (3 << (slot.0 * 2)) ^ ALL_MASK_64;
        self.0 = (self.0 & remove_mask) | ((value as u64) << (slot.0*2)) 
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