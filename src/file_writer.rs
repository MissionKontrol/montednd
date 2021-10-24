use std::fs::File;
use std::io::{BufWriter, Write};
use std::fs::OpenOptions;

fn main() {
    let config = get_config();

    let data = "Some data!";
    let f = File::create("/tmp/foo").expect("Unable to create file");
    let mut f = BufWriter::new(f);
    f.write_all(data.as_bytes()).expect("Unable to write data");
}

struct SetupConfiguration {
    file_name: String,
}

struct FileWriterHandle {
    config: SetupConfiguration,
    file_handle: std::fs::File,
}

pub enum FileWriter {
    Error(std::io::Error),
    Ready(std::fs::File),
}

impl FileWriter {
    pub fn new(file_name: &str) -> FileWriter {
        let file_result = OpenOptions::new().write(true).append(true).open(file_name);

        match file_result {
            Ok(file) => FileWriter::Ready(file),
            Err(error) => FileWriter::Error(error),
        }
    }

    pub fn write_buffer(self, buffer: &str) -> Result<std::io::Result<()>,FileWriter> {
        if let FileWriter::Ready(file) = self{
            let mut f = BufWriter::new(file);
            Ok(f.write_all(buffer.as_bytes()))
        }
        else { Err(self) }
    }
}

fn get_config() -> SetupConfiguration {
    SetupConfiguration {
        file_name: "test.out".to_string(),
    }
}