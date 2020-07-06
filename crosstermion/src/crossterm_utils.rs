pub fn into_io_error(err: crossterm::ErrorKind) -> std::io::Error {
    if let crossterm::ErrorKind::IoError(err) = err {
        return err;
    }
    unimplemented!("we cannot currently handle non-io errors reported by crossterm")
}
