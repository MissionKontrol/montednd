use std::io::{BufWriter, Write};
use std::fs::OpenOptions;

pub enum FileWriter {
    Error(std::io::Error),
    Ready(std::fs::File),
}

impl FileWriter {
    pub fn write_buffer(&self, buffer: &str) -> Result<std::io::Result<()>,String> {
        match self {
            FileWriter::Ready(file) => {
                let mut f = BufWriter::new(file);
                Ok(f.write_all(format!("{}\n", buffer).as_bytes()))
            }
            FileWriter::Error(_) => {
                Err("Fucked".to_string())
            }
        }
    }
}

pub fn new(file_name: &str) -> FileWriter {
    let file_result = OpenOptions::new().create(true).write(true).append(true).open(file_name);

    match file_result {
        Ok(file) => FileWriter::Ready(file),
        Err(error) => FileWriter::Error(error),
    }
}