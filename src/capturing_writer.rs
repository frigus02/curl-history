use std::io::Write;

#[derive(Debug)]
pub struct CapturingWriter<T> {
    data: Vec<u8>,
    writer: T,
}

impl<T> CapturingWriter<T> {
    pub fn new(writer: T) -> Self {
        Self {
            data: Vec::new(),
            writer,
        }
    }

    pub fn into_string(self) -> String {
        String::from_utf8(self.data).unwrap_or_else(|_| "".into())
    }
}

impl<T> Write for CapturingWriter<T>
where
    T: Write,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.data.extend(buf);
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
