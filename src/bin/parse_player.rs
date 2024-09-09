use bitvec::field::BitField;
use d2_itemsorter::{
    bitsy::{
        bitsy_to_bits, compare_bitslices, structs::BitsyInt, BitReader, BitVecReader, MyBitVec,
    },
    item::info::MapItemDb,
    player::{calculate_checksum, Player},
};

fn main() {
    let path = std::env::args().nth(1).expect("Missing player path");
    let output_path = std::env::args().nth(2);
    let bytes = std::fs::read(path).unwrap();
    let bits = MyBitVec::from_vec(bytes);
    let mut reader = BitVecReader::new(bits.clone(), MapItemDb::from_data_dir("data/items"));
    let mut player: Player = reader.read().unwrap();
    println!("Finished reading player @ {}", reader.index());
    reader.report_next_bytes(32);
    let tail = reader.read_tail().unwrap();
    println!("Tail was {} bits long: {}", tail.len(), tail);

    let mut roundtrip = bitsy_to_bits(&player, player.version);
    compare_bitslices(&bits, &roundtrip).unwrap();

    let new_checksum = calculate_checksum(&roundtrip);
    println!("Original checksum: {}", player.checksum);
    println!("New checksum: {}", new_checksum);
    roundtrip[96..128].store(new_checksum);
    compare_bitslices(&bits, &roundtrip).unwrap();
    println!(
        "Playter was roundtripped successfully! Number of bits: {}",
        roundtrip.len()
    );

    for item in player.items.iter_mut().filter(|i| !i.simple) {
        let props = item.item_properties.as_mut().unwrap();
        println!(
            "Item: {:?}. First unk prop: {:?}. Tail: {}",
            item.item_info,
            props.first_unknown_id(),
            props.tail,
        );
        for prop in props.properties.iter_mut() {
            if prop.definition.id == 80 || prop.definition.id == 74 {
                prop.values[0] = (1 << (prop.definition.values[0].size)) - 1
            }
            println!(" * {}", prop);
        }
    }

    let mut bits = bitsy_to_bits(&player, player.version);
    let new_checksum = calculate_checksum(&bits);
    println!("Original checksum: {}", player.checksum);
    println!("New checksum: {}", new_checksum);
    bits[96..128].store(new_checksum);

    if let Some(path) = output_path {
        println!("Writing player to {}", path);
        std::fs::write(path, bits.into_vec()).unwrap();
    };
}
