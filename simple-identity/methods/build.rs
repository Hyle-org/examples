fn main() {
    let results = risc0_build::embed_methods();

    results.iter().for_each(|data| {
        std::fs::write(format!("{}/{}.img", data.name, data.name), &data.elf)
            .expect("failed to write img");
        // Convert u32 slice to hex
        let hex_image_id = data
            .image_id
            .iter()
            .map(|x| format!("{:08x}", x.to_be()))
            .collect::<Vec<_>>()
            .join("");
        std::fs::write(format!("{}/{}.txt", data.name, data.name), &hex_image_id)
            .expect("failed to write program ID");
    });
}
