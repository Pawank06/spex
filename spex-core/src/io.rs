use std::fs;
use std::io;
use std::path::Path;

pub fn read_file(path: &Path) -> io::Result<Vec<u8>> {
    fs::read(path)
}

pub fn write_file(path: &Path, data: &[u8]) -> io::Result<()> {
    fs::write(path, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_bytes() {
        let dir = std::env::temp_dir();
        let path = dir.join("spex_io_test.bin");
        let payload = b"streaming protocol experiment";

        write_file(&path, payload).unwrap();
        let read = read_file(&path).unwrap();

        assert_eq!(read, payload);
        let _ = fs::remove_file(&path);
    }
}
