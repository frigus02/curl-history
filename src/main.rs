mod capturing_writer;

use capturing_writer::CapturingWriter;
use std::env;
use std::process::{Command, Stdio};
use structopt::StructOpt;

const STATUS_ERR_SIGNAL: i32 = -1;
const STATUS_ERR_SPAWN: i32 = -2;

type BoxError = Box<dyn std::error::Error>;

include!(concat!(env!("OUT_DIR"), "/curl_opts.rs"));

#[derive(Debug)]
struct Request {
    method: String,
    url: String,
}

fn try_parse_request() -> Result<Request, BoxError> {
    let mut opts = Opts::from_args_safe()?;
    Ok(Request {
        method: opts.request.take().unwrap_or_else(|| "GET".into()),
        url: opts
            .url
            .take()
            .or_else(|| opts.url_arg.first().cloned())
            .take()
            .unwrap(),
    })
}

#[derive(Debug)]
struct CurlResult {
    exit_code: i32,
    output: String,
}

fn try_run_curl() -> Result<CurlResult, BoxError> {
    let mut child = Command::new("curl")
        .args(env::args_os().skip(1))
        .stdout(Stdio::piped())
        .spawn()?;

    let mut child_stdout = child.stdout.take().expect("stdout is piped");
    let stdout = std::io::stdout();
    let mut stdout_capture = CapturingWriter::new(stdout.lock());
    let _ = std::io::copy(&mut child_stdout, &mut stdout_capture);

    let status = child.wait().expect("command wasn't running");
    let output = stdout_capture.into_string();

    Ok(CurlResult {
        exit_code: status.code().unwrap_or(STATUS_ERR_SIGNAL),
        output,
    })
}

fn save_request(req: Request, res: CurlResult) -> Result<(), BoxError> {
    println!("REQ: {:#?}", req);
    println!("RES: {:#?}", res);
    todo!()
}

fn main() {
    let req = try_parse_request();
    if let Err(ref err) = req {
        eprintln!("[curl-history] failed to parse request from args: {}", err);
    }

    let res = try_run_curl();
    let exit_code = res
        .as_ref()
        .map(|x| x.exit_code)
        .unwrap_or(STATUS_ERR_SPAWN);
    if let Err(ref err) = res {
        eprintln!("[curl-history] failed to run curl: {}", err);
    }

    if let (Ok(req), Ok(res)) = (req, res) {
        if let Err(err) = save_request(req, res) {
            eprintln!("[curl-history] failed to save request: {}", err);
        }
    }

    std::process::exit(exit_code);
}
