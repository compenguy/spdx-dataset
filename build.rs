use std::collections::BTreeSet;
use std::env;
use std::fs::read_dir;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

extern crate sha2;

use sha2::{Digest, Sha256};

pub const SPDX_JSON: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/modules/license-list-data/json/details/"
);

pub const SPDX_TEXT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/modules/license-list-data/text/"
);

fn write_spdx(input_dir: &Path, out_file: &Path) {
    let mut f = File::create(&out_file).expect(&format!(
        "Could not open codegen file {} for writing.",
        out_file.to_string_lossy()
    ));

    // A collection of all the spdx license file paths
    // it's sorted so that we can generate a stable fingerprint of the data
    let paths: BTreeSet<PathBuf> = read_dir(input_dir)
        .expect(&format!(
            "Failed while listing files in the spdx directory {}",
            input_dir.to_string_lossy()
        ))
        .map(|entry| entry.expect("Problem enumerating file in spdx directory."))
        .map(|entry| entry.path())
        .collect();

    writeln!(f, "use std::collections::HashMap;").expect("Write failed.");
    writeln!(f, "").expect("Write failed.");

    // Calculate the "fingerprint" of the license collection
    // this can be used by clients to detect when cached forms of this data should be invalidated
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    for path in &paths {
        let input = File::open(path).expect("Failed opening spdx input file.");
        let mut reader = BufReader::new(input);
        loop {
            let count = reader
                .read(&mut buffer)
                .expect("Failed reading spdx input file.");
            if count == 0 {
                break;
            }
            hasher.input(&buffer[..count]);
        }
    }
    let fingerprint: Vec<u8> = hasher.result()[..].to_vec();
    writeln!(
        f,
        "pub const SPDX_FINGERPRINT: &[u8] = &{:#X?};",
        &fingerprint
    )
    .expect("Write failed.");

    writeln!(f, "").expect("Write failed.");
    for path in &paths {
        if let Some(var_name) = convert_path_to_variable_name(path) {
            writeln!(
                f,
                "const {}: &str = include_str!(\"{}\");",
                var_name,
                path.to_str()
                    .expect("Unable to convert spdx input path string to utf-8")
            )
            .expect("Write failed.");
        }
    }

    writeln!(f, "").expect("Write failed.");
    writeln!(f, "lazy_static! {{").expect("Write failed.");
    writeln!(
        f,
        "\tpub static ref SPDX_LICENSES: HashMap<&'static str, &'static str> = {{"
    )
    .expect("Write failed.");
    writeln!(f, "\t\tlet mut m = HashMap::new();").expect("Write failed.");
    for path in &paths {
        if let Some(name) = path.file_stem() {
            let utf_name = name
                .to_str()
                .expect("Unable to convert spdx input path string to utf-8");
            let var_name = convert_path_to_variable_name(path)
                .expect("Unable to convert spdx input path string to utf-8");
            writeln!(f, "\t\tm.insert(\"{}\", {});", utf_name, var_name).expect("Write failed.");
        }
    }
    writeln!(f, "\t\tm").expect("Write failed.");
    writeln!(f, "\t}};").expect("Write failed.");
    writeln!(f, "}}").expect("Write failed.");
}

fn main() {
    let out_dir = env::var("OUT_DIR")
        .expect("OUT_DIR environment variable not set. This component requires the use of a compatible cargo version for build.");
    let dest_dir = Path::new(&out_dir);

    let json_input = Path::new(SPDX_JSON);
    let json_output = dest_dir.join("spdx_json.rs");
    write_spdx(&json_input, &json_output);

    let text_input = Path::new(SPDX_TEXT);
    let text_output = dest_dir.join("text_json.rs");
    write_spdx(&text_input, &text_output);
}

fn convert_path_to_variable_name(path: &PathBuf) -> Option<String> {
    path.file_name()
        .and_then(|file| file.to_str())
        .map(|file| {
            file.replace(".", "_")
                .replace("-", "_")
                .replace("+", "PLUS")
                .to_uppercase()
        })
        .map(|file| format!("SPDX_{}", file))
}
