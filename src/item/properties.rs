use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;

use crate::bitsy::*;
use crate::item::reader::ItemReader;

const TERMINATOR_ID: u16 = 0b111111111;
const PROPERTY_ID_SIZE: usize = 9;

pub struct PropertyList {
    pub properties: Vec<Property>,
    pub tail: MyBitVec,
}

impl Deref for PropertyList {
    type Target = Vec<Property>;

    fn deref(&self) -> &Self::Target {
        &self.properties
    }
}

impl PropertyList {
    pub fn parse(reader: &mut ItemReader) -> PropertyList {
        let mut properties: Vec<Property> = Vec::new();
        let mut tail = MyBitVec::new();

        loop {
            let id = reader.read_int::<u16>(PROPERTY_ID_SIZE);
            if id == TERMINATOR_ID {
                break;
            }

            if let Some(definition) = reader.property_db().get_definition(id) {
                let values = definition.parse_values(reader);
                // println!(
                //     "Found property {}: {}. Values: {:?}",
                //     id, definition.text, values
                // );
                properties.push(Property { values, definition })
            } else {
                // println!("Unknown property id: {}", id);
                tail.append_int(id, PROPERTY_ID_SIZE);
                tail.extend_from_bitslice(
                    reader
                        .read_until_bits(&bitvec_init(true, PROPERTY_ID_SIZE))
                        .as_bitslice(),
                );
                // Read the terminator ID and discard.
                reader.read_int::<u16>(PROPERTY_ID_SIZE);
                // println!("Tail has size {}: {}", tail.len(), tail);
                // println!("Next bits are: {}", reader.peek_bits(32));
                break;
            }
        }
        return PropertyList { properties, tail };
    }

    pub fn append_to(&self, bitvec: &mut MyBitVec) {
        for property in &self.properties {
            bitvec.append_int(property.definition.id, PROPERTY_ID_SIZE);
            property.definition.append_values(property.values, bitvec);
        }
        bitvec.extend_from_bitslice(&self.tail);
        bitvec.append_int(TERMINATOR_ID, PROPERTY_ID_SIZE);
    }

    pub fn tail_is_padding(&self) -> bool {
        return self.tail.len() < 8 && self.tail.not_any();
    }
}

pub struct Property {
    definition: PropertyDef,
    values: Values,
}

impl Property {
    pub fn definition(&self) -> &PropertyDef {
        &self.definition
    }
}

impl Display for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Property: {}, values:{}",
            self.definition.text,
            self.values.map(|x| x.to_string()).join(", ")
        )?;
        Ok(())
    }
}

const MAX_PROPERTY_VALUES: usize = 4;

type Values = [i32; MAX_PROPERTY_VALUES];

#[derive(Clone, Debug)]
pub struct PropertyDef {
    id: u16,
    text: String,
    values: [ValueDef; MAX_PROPERTY_VALUES],
}

impl PropertyDef {
    pub fn id(&self) -> u16 {
        self.id
    }

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
    ( $( $size:literal $(($offset:literal))? ),* ) => {
        {
            let mut value_defs = [ValueDef::default(); MAX_PROPERTY_VALUES];
            let mut index = 0;
            $(
                value_defs[index].size = $size;
                $(
                value_defs[index].offset = $offset;
                )?
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
    #[allow(unused_assignments)]
    #[rustfmt::skip]
    pub fn new() -> Self {
        let mut db = MapPropertyDb {
            properties: HashMap::new(),
        };

        db.add(PropertyDef::new(0, "{:+d} to Strength", defs![10(32)]));
        db.add(PropertyDef::new(1, "{:+d} to Energy", defs![10(32)]));
        db.add(PropertyDef::new(2, "{:+d} to Dexterity", defs![10(32)]));
        db.add(PropertyDef::new(3, "{:+d} to Vitality", defs![10(32)]));
        db.add(PropertyDef::new(7, "{:+d} to Life", defs![10(32)]));
        db.add(PropertyDef::new(9, "{:+d} to Mana", defs![10(32)]));
        db.add(PropertyDef::new(11, "{:+d} Maximum Stamina", defs![10(32)]));
        db.add(PropertyDef::new(16, "{:+d}% Enhanced Defense", defs![9]));
        db.add(PropertyDef::new(17, "{:+d}% Enhanced Damage", defs![9, 9]));
        db.add(PropertyDef::new(19, "{:+d} to Attack Rating", defs![10]));
        db.add(PropertyDef::new(20, "{:+d}% Increased Chance of Blocking", defs![6]));
        db.add(PropertyDef::new(21, "{:+d} to Minimum Damage", defs![8]));
        db.add(PropertyDef::new(22, "{:+d} to Maximum Damage", defs![9]));
        db.add(PropertyDef::new(23, "{:+d} to Minimum Damage", defs![8]));
        db.add(PropertyDef::new(24, "{:+d} to Maximum Damage", defs![9]));
        db.add(PropertyDef::new(27, "Regenerate Mana {:d}%", defs![8]));
        db.add(PropertyDef::new(28, "Heal Stamina Plus {:d}%", defs![8]));
        db.add(PropertyDef::new(31, "{:+d} Defense", defs![11(10)]));
        db.add(PropertyDef::new(32, "{:+d} Defense vs. Missile", defs![10]));
        db.add(PropertyDef::new(33, "{:+d} Defense vs. Melee", defs![10]));
        db.add(PropertyDef::new(34, "Damage Reduced by {:d}", defs![16]));
        db.add(PropertyDef::new(35, "Magic Damage Reduced by {:d}", defs![16]));
        db.add(PropertyDef::new(36, "Damage Reduced by {:+d}%", defs![8]));
        db.add(PropertyDef::new(37, "Magic Resist {:+d}%", defs![8(50)]));
        db.add(PropertyDef::new(38, "+{:d}% to Maximum Magic Resist", defs![5]));
        db.add(PropertyDef::new(39, "Fire Resist {:+d}%", defs![8(50)]));
        db.add(PropertyDef::new(40, "+{:d}% to max fire resist", defs![5]));
        db.add(PropertyDef::new(41, "Lightning Resist {:+d}%", defs![8(50)]));
        db.add(PropertyDef::new(42, "+{:d}% to max lightning resist", defs![5]));
        db.add(PropertyDef::new(43, "Cold Resist {:+d}%", defs![8(50)]));
        db.add(PropertyDef::new(44, "+{:d}% to max cold resist", defs![5]));
        db.add(PropertyDef::new(45, "Poison Resist {:+d}%", defs![8(50)]));
        db.add(PropertyDef::new(46, "{:+d} to max Poison Resist", defs![5]));
        db.add(PropertyDef::new(48, "Adds {:d}-{:d} fire damage", defs![10, 11]));
        db.add(PropertyDef::new(50, "Adds {:d}-{:d} lightning damage", defs![10, 11]));
        db.add(PropertyDef::new(52, "Adds {:d}-{:d} magic damage", defs![10, 11]));
        db.add(PropertyDef::new(54, "Adds {:d}-{:d} cold damage", defs![10, 11, 10]));
        db.add(PropertyDef::new(57, "+({:d}-{:d})/256 poison damage over {:d}/25 s", defs![13, 13, 16]));
        db.add(PropertyDef::new(60, "{:d}% Life Stolen per Hit", defs![8(50)]));
        db.add(PropertyDef::new(62, "{:d}% Mana Stolen per Hit", defs![8(50)]));
        db.add(PropertyDef::new(66, "Hit Stuns Enemies <{:d}>", defs![12]));
        db.add(PropertyDef::new(73, "[?][73] <{:d}>", defs![9]));
        db.add(PropertyDef::new(74, "+{:d} Replenish Life", defs![16(3000)]));
        db.add(PropertyDef::new(75, "Increased Maximum Durability {:d}%", defs![7(20)]));
        db.add(PropertyDef::new(76, "Increase Maximum Life {:d}%", defs![8(10)]));
        db.add(PropertyDef::new(77, "Increase Maximum Mana {:d}%", defs![8(10)]));
        db.add(PropertyDef::new(78, "Attacker takes damage of {:d}", defs![16]));
        db.add(PropertyDef::new(79, "{:d}% Extra Gold from Monsters", defs![13]));
        db.add(PropertyDef::new(80, "{:d}% Better Chance of Getting Magic Items", defs![13]));
        db.add(PropertyDef::new(81, "Knockback", defs![7]));
        db.add(PropertyDef::new(83, "+{1:d} to Class<{0:d}> Skill Levels", defs![3, 5]));
        db.add(PropertyDef::new(85, "{:d}% to Experience Gained", defs![12(50)]));
        db.add(PropertyDef::new(86, "{:+d} Life after each Kill", defs![7]));
        db.add(PropertyDef::new(87, "Reduces all Vendor Prices {:d}%", defs![7]));
        db.add(PropertyDef::new(89, "{:+d} to Light Radius", defs![5(12)]));
        db.add(PropertyDef::new(91, "Requirements {:+d}%", defs![12(100)]));
        db.add(PropertyDef::new(92, "Unknown<92>: {:+d}", defs![12]));
        db.add(PropertyDef::new(93, "{:+d}% Increased Attack Speed", defs![9(20)]));
        db.add(PropertyDef::new(96, "{:+d}% Faster Run/Walk", defs![9(100)]));
        db.add(PropertyDef::new(97, "+{1:d} to Skill<{0:d}> (All) [97]", defs![10, 7]));
        db.add(PropertyDef::new(98, "ConvertTo[?]<98>: {:d}", defs![10]));
        db.add(PropertyDef::new(99, "{:+d}% Faster Hit Recovery", defs![8(20)]));
        db.add(PropertyDef::new(102, "{:+d}% Faster Block Rate", defs![8(20)]));
        db.add(PropertyDef::new(105, "{:+d}% Faster Cast Rate", defs![9(50)]));
        db.add(PropertyDef::new(107, "+{1:d} to Skill<{0:d}> (Class Only) [107]", defs![10, 7]));
        db.add(PropertyDef::new(108, "Slain Monster Rest in Peace <{:+d}>%", defs![1]));
        db.add(PropertyDef::new(109, "Shorter Curse Duration {:+d}%", defs![9(100)]));
        db.add(PropertyDef::new(110, "Poison Length Reduced by {:d}%", defs![8(20)]));
        db.add(PropertyDef::new(112, "Hit Causes Monster to Flee {:d}%", defs![7(10)]));
        db.add(PropertyDef::new(113, "Hit Blinds Target ({:d})", defs![7]));
        db.add(PropertyDef::new(114, "{:d}% Damage Taken Goes To Mana", defs![7]));
        db.add(PropertyDef::new(115, "Ignore Target's Defense", defs![1]));
        db.add(PropertyDef::new(116, "-{:d}% Target Defense", defs![7]));
        db.add(PropertyDef::new(117, "Prevent Monster Heal", defs![7]));
        db.add(PropertyDef::new(118, "Half Freeze Duration", defs![1]));
        db.add(PropertyDef::new(119, "{:+d}% Bonus to Attack Rating", defs![12(20)]));
        db.add(PropertyDef::new(120, "{:+d} to Monster Defense Per Hit", defs![7(128)]));
        db.add(PropertyDef::new(121, "{:+d}% Damage to Demons", defs![12(20)]));
        db.add(PropertyDef::new(122, "{:+d}% Damage to Undead", defs![12(20)]));
        db.add(PropertyDef::new(123, "{:+d} to Attack Rating against Demons", defs![13(128)]));
        db.add(PropertyDef::new(124, "{:+d} to Attack Rating against Undead", defs![13(128)]));
        db.add(PropertyDef::new(126, "+{1:d} to SkillTree<{0:d}>", defs![3, 6]));
        db.add(PropertyDef::new(127, "+{:d} to All Skills", defs![5]));
        db.add(PropertyDef::new(128, "Attacker Takes Lightning Damage of {:+d}", defs![16]));
        db.add(PropertyDef::new(134, "Freezes Target <{:d}>", defs![5]));
        db.add(PropertyDef::new(135, "{:d}% Chance of Open Wounds", defs![9]));
        db.add(PropertyDef::new(136, "{:d}% Chance of Crushing Blow", defs![9]));
        db.add(PropertyDef::new(138, "{:+d} to Mana after each Kill", defs![7]));
        db.add(PropertyDef::new(139, "{:+d} to Life after each Kill", defs![7]));
        db.add(PropertyDef::new(140, "Unknown<140>: {:d}", defs![7]));
        db.add(PropertyDef::new(141, "{:d}% Deadly Strke", defs![8]));
        db.add(PropertyDef::new(142, "Fire Absorb {:d}%", defs![8]));
        db.add(PropertyDef::new(143, "{:d} Fire Absorb", defs![16]));
        db.add(PropertyDef::new(144, "Lightning Absorb {:d}%", defs![8]));
        db.add(PropertyDef::new(145, "{:d} Lightning Absorb", defs![16]));
        db.add(PropertyDef::new(146, "Magic Absorb {:d}%", defs![8]));
        db.add(PropertyDef::new(147, "{:d} Magic Absorb", defs![16]));
        db.add(PropertyDef::new(148, "Cold Absorb {:d}%", defs![8]));
        db.add(PropertyDef::new(149, "{:d} Cold Absorb", defs![16]));
        db.add(PropertyDef::new(150, "Slows Target by {:d}%", defs![7]));
        db.add(PropertyDef::new(151, "Level {1:d} Skill<{0:d}> When Equipped", defs![10, 8]));
        db.add(PropertyDef::new(152, "Indestructible", defs![1]));
        db.add(PropertyDef::new(153, "Cannot Be Frozen", defs![1]));
        db.add(PropertyDef::new(154, "{:+d}% Slower Stamina Drain", defs![8(90)]));
        db.add(PropertyDef::new(155, "{1:d}% reanimate as: Mob<{0:d}>", defs![10, 7]));
        db.add(PropertyDef::new(156, "Piercing Attack <{:d}>", defs![7]));
        db.add(PropertyDef::new(157, "Fires Magic Arrows <{:d}>", defs![7]));
        db.add(PropertyDef::new(158, "Fires Explosive Arrows or Bolds <{:d}>", defs![7]));
        db.add(PropertyDef::new(159, "{:+d} to Minimum Damage", defs![9]));
        db.add(PropertyDef::new(160, "{:+d} to Maximum Damage", defs![10]));
        db.add(PropertyDef::new(179, "{1:+d}% to Damage/AR against EnemyClass<{0:d}>", defs![10, 12]));
        db.add(PropertyDef::new(180, "{1:+d}% to AR/Damage against EnemyClass<{0:d}>", defs![10, 12]));
        db.add(PropertyDef::new(181, "[?][181] ??? <{:d}>", defs![9]));
        db.add(PropertyDef::new(188, "+{1:d} to Skill<{0:d}> [188][?]", defs![16, 3]));
        db.add(PropertyDef::new(195, "{2:d}% Chance to cast Level {0:d} Skill<{1:d}> on attack", defs![6, 10, 7]));
        db.add(PropertyDef::new(196, "{2:d}% Chance to cast Level {0:d} Skill<{1:d}> when you Kill an Enemy", defs![6, 10, 7]));
        db.add(PropertyDef::new(197, "{2:d}% Chance to cast Level {0:d} Skill<{1:d}> when you Die", defs![6, 10, 7]));
        db.add(PropertyDef::new(198, "{2:d}% Chance to cast Level {0:d} Skill<{1:d}> on striking", defs![6, 10, 7]));
        db.add(PropertyDef::new(201, "{2:d}% Chance to cast Level {0:d} Skill<{1:d}> when struck", defs![6, 10, 7]));
        db.add(PropertyDef::new(204, "Level {:d} Skill<{:d}> ({:d}/{:d} charges)", defs![6, 10, 8, 8]));
        db.add(PropertyDef::new(214, "{:+d}/8 to Defense (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(215, "{:+d}/16% Enhanced Defense (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(217, "{:+d}/16 to Mana (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(218, "{:+d}/16 to Maximum Damage (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(220, "{:+d}/16 to Strength (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(221, "{:+d}/16 to Dexterity (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(222, "{:+d}/16 to Energy (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(224, "{:+d}/2 to Attack Rating (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(225, "{:+d}/8% Bonus to Attack Rating (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(228, "Indestructible [?]", defs![6]));
        db.add(PropertyDef::new(230, "Cold Resist {:d}/16 (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(231, "Fire Resist {:d}/16 (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(232, "{:+d}/16 to Lightning Resist (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(233, "{:+d}/16 to Poison Resist (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(239, "{:+d}/16 Extra Gold form Monsters (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(240, "{:+d}/16 Better Chance of Getting Magic Items (Based on Character Level)", defs![6]));
        db.add(PropertyDef::new(252, "Repairs 1 durability in 100/{:d} seconds", defs![6]));
        db.add(PropertyDef::new(253, "Replenishes Quantity ({:+d}/??)[?]", defs![8]));
        db.add(PropertyDef::new(254, "Increaed Stack Size ({:+d})", defs![8]));
        db.add(PropertyDef::new(329, "{:+d}% to Fire Skill Damage", defs![12(50)]));
        db.add(PropertyDef::new(330, "{:+d}% to Lightning Skill Damage", defs![12(50)]));
        db.add(PropertyDef::new(331, "{:+d}% to Cold Skill Damage", defs![12(50)]));
        db.add(PropertyDef::new(332, "{:+d}% to Poison Skill Damage", defs![12(50)]));
        db.add(PropertyDef::new(333, "-{:d}% to Enemy Lightning Resistance", defs![9]));
        db.add(PropertyDef::new(334, "-{:d}% to Enemy Lightning Resistance", defs![9]));
        db.add(PropertyDef::new(335, "-{:d}% to Enemy Cold Resistance", defs![9]));
        db.add(PropertyDef::new(336, "-{:d}% to Enemy Poison Resistance", defs![9]));
        db.add(PropertyDef::new(338, "Chance to dodge melee attack when still +{:d}%", defs![7]));
        db.add(PropertyDef::new(339, "Chance to dodge missile attack when still +{:d}%", defs![7]));
        db.add(PropertyDef::new(340, "Chance to dodge attacks when moving +{:d}%", defs![7]));
        db.add(PropertyDef::new(349, "Elemental resistance of summons {:+d}%", defs![8]));
        db.add(PropertyDef::new(357, "{:+d}% to Magic Skill Damage", defs![12(50)]));
        db.add(PropertyDef::new(359, "Magic Affinity Bonus {:+d}%", defs![12(100)]));
        db.add(PropertyDef::new(362, "Extra Throwing Potion Damage +{:d}%", defs![12]));
        db.add(PropertyDef::new(365, "Strength bonus {:d}%", defs![8(10)]));
        db.add(PropertyDef::new(366, "Energy bonus {:d}%", defs![8(10)]));
        db.add(PropertyDef::new(367, "Dexterity bonus {:d}%", defs![8(10)]));
        db.add(PropertyDef::new(372, "[?][372] <{:d}>", defs![8]));
        db.add(PropertyDef::new(388, "{:d}% Extra Base Life to Summons", defs![9(50)]));
        db.add(PropertyDef::new(407, "{2:d}% Chance to cast Level {0:d} Skill<{1:d}> when struck", defs![6, 10, 7]));
        db.add(PropertyDef::new(441, "Extra resistance from temporary resistance potions +{:d}%", defs![7]));
        db.add(PropertyDef::new(443, "+{:d} Extra duration (in frames) to all resistance potions", defs![15]));
        db.add(PropertyDef::new(444, "+{:d} Extra duration (in frames) to stamina potions", defs![15]));
        db.add(PropertyDef::new(446, "Stamina Bonus {:d}%", defs![9(60)]));
        db.add(PropertyDef::new(449, "bonus healing from normal rejuvination potions {:d}%", defs![7]));
        db.add(PropertyDef::new(451, "Boosts the effectiveness of mana potions by x {:d}", defs![4]));
        db.add(PropertyDef::new(465, "Boosts Double Throw Damage by {:d}%", defs![9]));
        db.add(PropertyDef::new(471, "Boosts damage of Hireling Skills by {:d}%", defs![9]));
        db.add(PropertyDef::new(479, "+{:d} extra Potions launched from Potion Launcher skill", defs![5]));
        db.add(PropertyDef::new(495, "+{:d}/?? Min/Max Fire Damage (Increases with kills)[?]", defs![6]));
        db.add(PropertyDef::new(502, "+{:d} Extra duration (in frames) to RIP Potions", defs![15]));
        db.add(PropertyDef::new(505, "+{:d} Extra duration (in frames) to portable shrines", defs![15]));
        db.add(PropertyDef::new(508, "Boosts Summon Damage by {:d}%", defs![12]));

        return db;
    }

    fn add(&mut self, def: PropertyDef) {
        self.properties.insert(def.id, def);
    }
}
