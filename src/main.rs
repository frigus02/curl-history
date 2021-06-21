use std::env;
use std::io::Write;
use std::process::{Command, Stdio};
use structopt::StructOpt;

const STATUS_ERR_SIGNAL: i32 = -1;
const STATUS_ERR_SPAWN: i32 = -2;

include!(concat!(env!("OUT_DIR"), "/curl_opts.rs"));

#[derive(Debug)]
struct CapturingWriter<T> {
    data: Vec<u8>,
    writer: T,
}

impl<T> CapturingWriter<T> {
    fn new(writer: T) -> Self {
        Self {
            data: Vec::new(),
            writer,
        }
    }

    fn into_string(self) -> String {
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

#[derive(Debug)]
struct Output {
    out: String,
    err: String,
}

impl Output {
    fn new(out: String, err: String) -> Self {
        Self { out, err }
    }
}

fn main() {
    let opts = match Opts::from_args_safe() {
        Ok(opts) => Some(opts),
        Err(err) => {
            eprintln!("[curl-history] failed to parse args: {}", err);
            None
        }
    };

    let (exit_code, output) = match Command::new("curl")
        .args(env::args_os().skip(1))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            let mut child_stdout = child.stdout.take().expect("stdout is piped");
            let stdout_handle = std::thread::spawn(move || {
                let stdout = std::io::stdout();
                let mut stdout_capture = CapturingWriter::new(stdout.lock());
                let _ = std::io::copy(&mut child_stdout, &mut stdout_capture);
                stdout_capture.into_string()
            });

            let mut child_stderr = child.stderr.take().expect("stderr is piped");
            let stderr_handle = std::thread::spawn(move || {
                let stderr = std::io::stderr();
                let mut stderr_capture = CapturingWriter::new(stderr.lock());
                let _ = std::io::copy(&mut child_stderr, &mut stderr_capture);
                stderr_capture.into_string()
            });

            let status = child.wait().expect("command wasn't running");
            let output = Output::new(stdout_handle.join().unwrap(), stderr_handle.join().unwrap());
            (status.code().unwrap_or(STATUS_ERR_SIGNAL), Some(output))
        }
        Err(err) => {
            eprintln!("{}", err);
            (STATUS_ERR_SPAWN, None)
        }
    };

    println!("REQUEST: {:#?}", opts);
    println!("RESPONSE: {:#?}", output);

    std::process::exit(exit_code);
}
