use crate::capturing_writer::CapturingWriter;
use crate::db::ensure_db;
use crate::BoxError;
use std::ffi::OsString;
use std::process::{Command, Stdio};
use structopt::StructOpt;

const STATUS_ERR_SIGNAL: i32 = -1;
const STATUS_ERR_SPAWN: i32 = -2;

include!(concat!(env!("OUT_DIR"), "/curl_opts.rs"));

#[derive(Debug)]
struct Request {
    method: String,
    url: String,
}

fn try_parse_request(args: &[OsString]) -> Result<Request, BoxError> {
    let mut opts = Opts::from_iter_safe(args)?;
    Ok(Request {
        method: opts.request.take().unwrap_or_else(|| "GET".into()),
        url: opts
            .url
            .take()
            .or_else(|| opts.url_arg.first().cloned())
            .ok_or("missing url")?,
    })
}

#[derive(Debug)]
struct CurlResult {
    exit_code: i32,
    output: String,
}

fn try_run_curl(args: &[OsString]) -> Result<CurlResult, BoxError> {
    let mut child = Command::new("curl")
        .args(args)
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

async fn save_history(req: Request, res: CurlResult) -> Result<(), BoxError> {
    let pool = ensure_db().await?;
    let _res = sqlx::query!(
        "INSERT INTO history (method, url, output)
        VALUES (?, ?, ?)",
        req.method,
        req.url,
        res.output
    )
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn run_curl_and_save_history(args: Vec<OsString>) {
    let req = try_parse_request(&args);
    if let Err(ref err) = req {
        eprintln!("[curl-history] failed to parse request from args: {}", err);
    }

    let res = try_run_curl(&args);
    let exit_code = res
        .as_ref()
        .map(|x| x.exit_code)
        .unwrap_or(STATUS_ERR_SPAWN);
    if let Err(ref err) = res {
        eprintln!("[curl-history] failed to run curl: {}", err);
    }

    if let (Ok(req), Ok(res)) = (req, res) {
        if let Err(err) = save_history(req, res).await {
            eprintln!("[curl-history] failed to save request: {}", err);
        }
    }

    std::process::exit(exit_code);
}
