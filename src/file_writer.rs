use std::fs::File;
use std::io::{BufWriter, Write};

fn main() {
    let config = get_config();

    let data = "Some data!";
    let f = File::create("/tmp/foo").expect("Unable to create file");
    let mut f = BufWriter::new(f);
    f.write_all(data.as_bytes()).expect("Unable to write data");
}

struct SetupConfiguration {
    path: String,
    file_name: String,
}

impl SetupConfiguration {
    fn get_full_path_filename(&self) -> String {
        let full_array = [self.path.clone(), "/".to_string(), self.file_name.clone()];
        full_array.join("")
    }
}
fn get_config() -> SetupConfiguration {
    SetupConfiguration {
        path: "./output".to_string(),
        file_name: "test.out".to_string(),
    }
}

#[test]
fn get_full_path_filename_test() {
    let config = SetupConfiguration {
        path: "./output".to_string(),
        file_name: "test.out".to_string(),
    };

    let result = config.get_full_path_filename();
    assert_eq!(result, "./output/test.out");
}
