use d2_itemsorter::{
    bitsy::{bitsy_to_bits, compare_bitslices, BitReader, BitVecReader, MyBitVec},
    player::Player,
};

fn main() {
    let path = std::env::args().nth(1).expect("Missing player path");
    let bytes = std::fs::read(path).unwrap();
    let bits = MyBitVec::from_vec(bytes);
    let mut reader = BitVecReader::new(bits.clone());
    let player: Player = reader.read().unwrap();
    println!("Finished reading player @ {}", reader.index());
    reader.report_next_bytes(32);
    let tail = reader.read_tail().unwrap();
    println!("Tail was {} bits long: {}", tail.len(), tail);

    let roundtrip = bitsy_to_bits(&player, player.version);
    compare_bitslices(&bits, &roundtrip).unwrap();
    println!(
        "Playter was roundtripped successfully! Number of bits: {}",
        roundtrip.len()
    );
}
