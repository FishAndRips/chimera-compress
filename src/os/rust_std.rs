use crate::do_it;

pub fn read_file(file: &str) -> Result<Vec<u8>, String> {
    std::fs::read(file).map_err(|e| e.to_string())
}

pub fn write_file(file: &str, data: &[u8]) -> Result<(), String> {
    std::fs::write(file, data).map_err(|e| e.to_string())
}

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    if (args.len() != 3 && args.len() != 4) || (args[0] != "compress" && args[0] != "decompress") {
        println!("Usage: {} <compress|decompress> <path/to/map.map> [output.map]", args[0]);
        std::process::exit(1);
    }

    let compressing = args[1] == "compress";
    let input_path_utf8 = &args[2];
    let output_path_utf8 = args.get(3).unwrap_or(input_path_utf8);

    let code = do_it(compressing, input_path_utf8.as_str(), output_path_utf8.as_str());
    std::process::exit(code);
}
