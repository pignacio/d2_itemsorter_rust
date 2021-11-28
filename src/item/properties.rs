use std::collections::HashMap;
use std::ops::Deref;

use bitvec::bitvec;
use bitvec::prelude::Lsb0;

use crate::bitsy::*;
use crate::item::reader::ItemReader;

const TERMINATOR_ID: u16 = 0b111111111;

pub struct PropertyList {
    properties: Vec<Property>,
    tail: MyBitVec,
}

impl Deref for PropertyList {
    type Target = Vec<Property>;

    fn deref(&self) -> &Self::Target {
        return &self.properties;
    }
}

impl PropertyList {
    fn parse(itemreader: &mut ItemReader) -> PropertyList {
        let properties: Vec<Property> = Vec::new();
        let mut tail = MyBitVec::new();

        while let id = itemreader.read_int::<u16>(9) {
            if id == TERMINATOR_ID {
                break;
            }

            tail.extend_from_bitslice(
                itemreader
                    .read_until_bits(&bitvec_init(true, 9))
                    .as_bitslice(),
            );
        }
        return PropertyList { properties, tail };
    }

    fn append_to(&self, bitvec: MyBitVec) {}
}

pub struct Property {
    id: u16,
}

const MAX_PROPERTY_VALUES: usize = 4;

type Values = [i32; MAX_PROPERTY_VALUES];

#[derive(Clone, Debug)]
struct PropertyDef {
    id: u16,
    text: String,
    values: [ValueDef; MAX_PROPERTY_VALUES],
}

impl PropertyDef {
    fn parse_values(&self, reader: &mut ItemReader) -> Values {
        let mut result = [0i32; MAX_PROPERTY_VALUES];
        for index in 0..MAX_PROPERTY_VALUES {
            let definition = self.values[index];
            if definition.size > 0 {
                let value: u32 = reader.read_int(definition.size);
                result[index] = value as i32 - definition.offset as i32;
            }
        }
        return result;
    }

    fn append_values(&self, values: Values, bits: &mut MyBitVec) {
        for index in 0..MAX_PROPERTY_VALUES {
            let definition = self.values[index];
            if definition.size > 0 {
                bits.append_int(
                    (values[index] + definition.offset as i32) as u32,
                    definition.size,
                );
            }
        }
    }

    fn new<S: AsRef<str>>(id: u16, text: S, values: [ValueDef; MAX_PROPERTY_VALUES]) -> Self {
        return PropertyDef {
            id,
            text: text.as_ref().to_string(),
            values,
        };
    }
}

#[derive(Copy, Clone, Debug)]
struct ValueDef {
    size: usize,
    offset: usize,
}

impl Default for ValueDef {
    fn default() -> Self {
        ValueDef { size: 0, offset: 0 }
    }
}

#[macro_export]
macro_rules! defs {
    ( $( $x:expr,$y:expr );* ) => {
        {
            let mut value_defs = [ValueDef::default(); MAX_PROPERTY_VALUES];
            let mut index = 0;
            $(
                value_defs[index].size = $x;
                value_defs[index].offset = $y;
                index += 1;
            )*
            value_defs
        }
    };
}

pub trait PropertyDb {
    fn get_definition(&self, id: u16) -> Option<PropertyDef>;
}

pub struct MapPropertyDb {
    properties: HashMap<u16, PropertyDef>,
}

impl PropertyDb for MapPropertyDb {
    fn get_definition(&self, id: u16) -> Option<PropertyDef> {
        return self.properties.get(&id).map(|x| x.clone());
    }
}

impl MapPropertyDb {
    pub fn new() -> Self {
        let mut db = MapPropertyDb {
            properties: HashMap::new(),
        };

        db.add(PropertyDef::new(0, "{:+d} to Strength", defs![10, 32]));
        db.add(PropertyDef::new(1, "{:+d} to Energy", defs![10, 32]));
        db.add(PropertyDef::new(2, "{:+d} to Dexterity", defs![10, 32]));
        db.add(PropertyDef::new(3, "{:+d} to Vitality", defs![10, 32]));
        db.add(PropertyDef::new(
            48,
            "Adds {:d}-{:d} fire damage",
            defs![10, 0; 11, 0],
        ));

        println!("{:?}", db.get_definition(0));
        println!("{:?}", db.get_definition(48));

        panic!();

        return db;
    }

    fn add(&mut self, def: PropertyDef) {
        self.properties.insert(def.id, def);
    }
}
