use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let output = Command::new("curl").arg("-h").output().unwrap();
    if !output.status.success() {
        panic!("curl -h was not successful");
    }
    let usage = String::from_utf8(output.stdout).unwrap();
    let re_arg =
        Regex::new(r#"^\s*(-(.),)?\s*(--([^ ]+))\s*((<[^>]+>)|(\[[^ ]+\]))?\s*(.+)$"#).unwrap();
    let struct_fields = usage
        .lines()
        .skip(1)
        .map(|line| {
            let caps = re_arg.captures(line).unwrap();
            let short = caps.get(2);
            let long = caps.get(4).unwrap().as_str();
            let value = caps.get(5);
            let desc = caps.get(8).unwrap().as_str();

            let mut params = vec![format!("long = \"{}\"", long)];
            if let Some(short) = short {
                params.push(format!("short = \"{}\"", short.as_str()));
            }
            format!(
                r#"/// {}
                #[structopt({})]
                {}: {},"#,
                desc,
                params.join(", "),
                long.replace(&['-', '.'][..], "_"),
                if value.is_some() { "Option<String>" } else { "bool" }
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("curl_opts.rs");
    fs::write(
        &dest_path,
        format!(
            r#"#[derive(Debug, structopt::StructOpt)]
            #[structopt(settings = &[
                structopt::clap::AppSettings::NoBinaryName,
                structopt::clap::AppSettings::DisableVersion,
                structopt::clap::AppSettings::DisableHelpFlags,
                structopt::clap::AppSettings::DisableHelpSubcommand,
            ])]
            struct Opts {{
                {}

                #[structopt(name = "url_arg")]
                url_arg: Vec<String>
            }}"#,
            struct_fields
        ),
    )
    .unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
