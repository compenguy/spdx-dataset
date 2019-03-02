use std::env;
use std::fs::read_dir;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const SPDX_JSON: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/modules/license-list-data/json/details/"
);

fn main() {
    let out_dir = env::var("OUT_DIR")
        .expect("OUT_DIR environment variable not set. This component requires the use of a compatible cargo version for build.");
    let dest_path = Path::new(&out_dir).join("spdx_json.rs");
    let mut f = File::create(&dest_path).expect(&format!(
        "Could not open codegen file {} for writing.",
        dest_path.to_string_lossy()
    ));

    // A collection of all the spdx license file paths
    let paths: Vec<PathBuf> = read_dir(SPDX_JSON)
        .expect(&format!(
            "Failed while listing files in the spdx json directory {}",
            SPDX_JSON
        ))
        .map(|entry| entry.expect("Problem enumerating file in spdx json directory."))
        .map(|entry| entry.path())
        .collect();

    writeln!(f, "use std::collections::HashMap;").expect("Write failed.");
    writeln!(f, "").expect("Write failed.");
    for path in &paths {
        if let Some(var_name) = convert_path_to_variable_name(path) {
            writeln!(
                f,
                "const {}: &str = include_str!(\"{}\");",
                var_name,
                path.to_str()
                    .expect("Unable to convert spdx json path string to utf-8")
            )
            .expect("Write failed.");
        }
    }

    writeln!(f, "").expect("Write failed.");
    writeln!(f, "lazy_static! {{").expect("Write failed.");
    writeln!(
        f,
        "\tpub static ref SPDX_NAMES: HashMap<&'static str, &'static str> = {{"
    )
    .expect("Write failed.");
    writeln!(f, "\t\tlet mut m = HashMap::new();").expect("Write failed.");
    for path in &paths {
        if let Some(name) = path.file_stem() {
            let utf_name = name
                .to_str()
                .expect("Unable to convert spdx json path string to utf-8");
            let var_name = convert_path_to_variable_name(path)
                .expect("Unable to convert spdx json path string to utf-8");
            writeln!(f, "\t\tm.insert(\"{}\", {});", utf_name, var_name).expect("Write failed.");
        }
    }
    writeln!(f, "\t\tm").expect("Write failed.");
    writeln!(f, "\t}};").expect("Write failed.");
    writeln!(f, "}}").expect("Write failed.");
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
