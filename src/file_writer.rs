use std::io::{BufWriter, Write};
use std::{io};
use std::fs::OpenOptions;

pub enum FileWriter {
    Error(std::io::Error),
    Ready(FileWriterHandle),
}

pub struct FileWriterHandle {
    file: std::fs::File,
}

impl FileWriterHandle {
    pub fn write_buffer(&self, buffer: &str) -> io::Result<()> {
        let mut f = BufWriter::new(&self.file);
        match Write::write_all(&mut f, format!("{}\n", buffer).as_bytes()) {
            Err(error) => {println!("Write error! {}",error); Err(error)},
            Ok(_) => Ok(()),
        }
    }
}

pub fn new(file_name: &str) -> FileWriter {
    let file_result = OpenOptions::new().create(true).write(true).append(true).open(file_name);

    match file_result {
        Ok(file) => FileWriter::Ready(FileWriterHandle{ file }),
        Err(error) => FileWriter::Error(error),
    }
}