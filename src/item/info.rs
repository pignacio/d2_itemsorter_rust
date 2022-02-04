use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ItemInfo {
    pub id: String,
    pub name: String,
    pub width: Option<u8>,
    pub height: Option<u8>,
    pub has_durability: bool,
    pub has_defense: bool,
    pub has_quantity: bool,
}

impl ItemInfo {
    fn default(id: &str) -> Self {
        return ItemInfo {
            id: id.to_string(),
            name: "?????????".to_string(),
            height: None,
            width: None,
            has_durability: false,
            has_defense: false,
            has_quantity: false,
        };
    }
}

impl Default for ItemInfo {
    fn default() -> Self {
        return ItemInfo::default("????");
    }
}

pub trait ItemDb {
    fn get_info(&self, id: &str) -> ItemInfo;
}

pub struct MapItemDb {
    item_infos: HashMap<String, ItemInfo>,
}

impl MapItemDb {
    pub fn new() -> MapItemDb {
        return MapItemDb {
            item_infos: HashMap::new(),
        };
    }

    pub fn from_data_dir<P: AsRef<Path>>(path: P) -> MapItemDb {
        let mut item_db = MapItemDb::new();
        let path_ref = path.as_ref();
        item_db.add_items_from_csv(path_ref.join("armors.csv"), true, true, false);
        item_db.add_items_from_csv(path_ref.join("belts.csv"), true, true, false);
        item_db.add_items_from_csv(path_ref.join("boots.csv"), true, true, false);
        item_db.add_items_from_csv(path_ref.join("gems.csv"), false, false, false);
        item_db.add_items_from_csv(path_ref.join("gloves.csv"), true, true, false);
        item_db.add_items_from_csv(path_ref.join("helmets.csv"), true, true, false);
        item_db.add_items_from_csv(path_ref.join("items.csv"), false, false, false);
        item_db.add_items_from_csv(path_ref.join("runes.csv"), false, false, false);
        item_db.add_items_from_csv(path_ref.join("shields.csv"), true, true, false);
        item_db.add_items_from_csv(path_ref.join("souls.csv"), false, false, false);
        item_db.add_items_from_csv(path_ref.join("stack.csv"), false, false, true);
        item_db.add_items_from_csv(path_ref.join("stack-weapons.csv"), true, false, true);
        item_db.add_items_from_csv(path_ref.join("weapons.csv"), true, false, false);
        return item_db;
    }

    fn add_items_from_csv<P: AsRef<Path>>(
        &mut self,
        path: P,
        has_durability: bool,
        has_defense: bool,
        has_quantity: bool,
    ) {
        let string_path = path.as_ref().to_string_lossy().to_string();
        let mut reader = csv::Reader::from_path(path).unwrap();
        for result in reader.records() {
            let row = result.unwrap();
            let id = row.get(0).unwrap().to_string();
            self.item_infos.insert(
                id.to_string(),
                ItemInfo {
                    id,
                    name: row.get(1).unwrap().to_string(),
                    width: row.get(2).unwrap().parse::<u8>().ok(),
                    height: row.get(3).unwrap().parse::<u8>().ok(),
                    has_durability,
                    has_defense,
                    has_quantity,
                },
            );
        }
        println!(
            "Finished loading data from {}. Final size: {}",
            string_path,
            self.item_infos.len()
        )
    }
}

impl ItemDb for MapItemDb {
    fn get_info(&self, id: &str) -> ItemInfo {
        self.item_infos
            .get(id)
            .map(|x| x.clone())
            .unwrap_or_else(|| {
                return ItemInfo {
                    id: id.to_string(),
                    name: "?????????".to_string(),
                    height: None,
                    width: None,
                    has_durability: false,
                    has_defense: false,
                    has_quantity: false,
                };
            })
    }
}
