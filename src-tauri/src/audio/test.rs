use metaflac::Tag;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let flac_file_path = "C:/Users/bnpco/Music/Ado/[A]  Kura Kura [318697446] [2023]/01 - Ado - Kura Kura (Dolby Atmos).flac"; // Replace with your FLAC file path
    let output_image_path = "C:/Users/bnpco/Music/Ado/[A]  Kura Kura [318697446] [2023]/extracted_cover.jpg"; // Desired output image path

    let tag = Tag::read_from_path(flac_file_path)?;

    if let Some(picture) = tag.pictures().next() {
        // Assuming you want the first picture found
        let mut file = File::create(output_image_path)?;
        file.write_all(&picture.data)?;
        println!("Successfully extracted cover art to {}", output_image_path);
    } else {
        println!("No embedded picture found in {}", flac_file_path);
    }

    Ok(())
}

#[test]
fn test_extract_flac_cover_art() -> Result<(), Box<dyn std::error::Error>> {
    main()
}