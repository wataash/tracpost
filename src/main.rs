// SPDX-License-Identifier: Apache-2.0

// https://github.com/wataash/tracpost
const ABOUT :&str = "(WIP) File-based management for Edgewall Software's Trac tickets -- create, update tickets by editing local files";

fn main() {
    // TODO: PR: --help:
    //     tracpost [OPTIONS] <FILE> --trac-pass <TRAC_PASS> --trac-user <TRAC_USER>
    //   is strange; expect:
    //     tracpost [OPTIONS] --trac-pass <TRAC_PASS> --trac-user <TRAC_USER> <FILE>
    //   clap::app::usage::create_help_usage()
    //   -> clap::app::usage::get_required_usage_from()
    let arg_matches = clap::App::new("tracpost")
            .version("0.1.0")
            .about(ABOUT)
            .arg(
                clap::Arg::with_name("trac_user")
                    .long("trac-user")
                    .value_name("TRAC_USER")
                    .takes_value(true)
                    .env("TRAC_USER")
                    .required(true)
                    .help("Username on Trac"),
            )
            .arg(
                clap::Arg::with_name("trac_pass")
                    .long("trac-pass")
                    .value_name("TRAC_PASS")
                    .takes_value(true)
                    .env("TRAC_PASS")
                    .required(true)
                    .help("Password for Trac"),
            )
            .arg(
                clap::Arg::with_name("url_rpc")
                    .long("url-rpc")
                    .value_name("URL_RPC")
                    .takes_value(true)
                    .help("http://<trac url>/login/rpc; You can also set it as \"url_rpc\" in the Wiki header"),
            )
            .arg(
                clap::Arg::with_name("comment")
                    .long("comment")
                    .value_name("COMMENT")
                    .takes_value(true)
                    .help("Comment on update (ignored on create)"),
            )
            // TODO: -q -vv
            .arg(
                clap::Arg::with_name("FILE")
                    .index(1)
                    .required(true)
                    .help("Path to TracWiki (MoinMoin) file"),
            )
            .get_matches();

    // println!("{:?}", arg_matches);
    // println!("{:#?}", arg_matches);

    let mut config = tracpost::Config {
        ..Default::default()
    };

    config.trac_user = arg_matches
        .value_of_os("trac_user")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    config.trac_pass = arg_matches
        .value_of_os("trac_pass")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    if let Some(x) = arg_matches.value_of_os("url_rpc") {
        config.url_rpc = Some(x.to_str().unwrap().to_string());
    }
    if let Some(x) = arg_matches.value_of_os("comment") {
        config.update_comment = Some(x.to_str().unwrap().to_string());
    }
    let path = arg_matches
        .value_of_os("FILE")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    // TODO
    // debug!("config: {:#?}", config);
    // eprintln!("config: {:#?}", config);

    let txt = match std::fs::read_to_string(&path) {
        Ok(x) => x,
        Err(error) => {
            // TODO
            // error!("failed to read {}: {}", path, error);
            eprintln!("failed to read {}: {}", path, error);
            return; // TODO: return error?
        }
    };
    let _tmp = tracpost::post(config, &txt);
    // TODO: panic only on my errors
    let _breakpoint = 1;
}
