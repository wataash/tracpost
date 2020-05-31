// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

#[macro_use]
mod utils;

// TODO: --comment='update foo' --attr=foo:bar
//   when create: commment='update foo' is ignored on create

// -------------------------------------------------------------------------------------------------
// cli

// https://github.com/wataash/tracpost
const ABOUT :&str = "(WIP) File-based management for Edgewall Software's Trac tickets -- create, update tickets by editing local files";

#[derive(Debug, Default)]
pub struct Config {
    trac_user: String,
    trac_pass: String,
    url_rpc_arg: bool,
    url_rpc: String, // priority: arg (--url-rpc) > "url_rpc" in file
    interactive: bool,
}

// TODO: move to main.rs?
pub fn main() {
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
        // TODO: -q -vv
        // TODO: comment (effective only on update)
        .arg(
            clap::Arg::with_name("FILE")
                .index(1)
                .required(true)
                .help("Path to TracWiki (MoinMoin) file"),
        )
        .get_matches();

    // println!("{:?}", arg_matches);
    // println!("{:#?}", arg_matches);

    let mut config = Config {
        interactive: true,
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
        config.url_rpc_arg = true;
        config.url_rpc = x.to_str().unwrap().to_string();
    }
    let path = arg_matches
        .value_of_os("FILE")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    debug!("config: {:#?}", config);

    let txt = match std::fs::read_to_string(&path) {
        Ok(x) => x,
        Err(error) => {
            error!("failed to read {}: {}", path, error);
            return;
        }
    };
    let tmp = post(&config, &txt);
    // TODO: panic only on my errors
    let _breakpoint = 1;
}

// -------------------------------------------------------------------------------------------------
// api

pub fn post(config: &Config, txt: &str) -> Result<(), failure::Error> {
    let poster = txt_to_poster(config, txt)?;
    poster.post(&config)?;
    Ok(())
}

// -------------------------------------------------------------------------------------------------
// post

// key - (line, value)
// TODO: avoid magic: .0 .1; name it
type Header = HashMap<String, (i32, String)>;

trait Poster {
    fn post(&self, config: &Config) -> Result<(), failure::Error>;

    fn post_json(&self, config: &Config, json: &serde_json::Value) -> Result<(), failure::Error> {
        // TODO: short log
        debug!("{}", json.to_string());

        // TODO: env (TRACPOST_USER TRACPOST_PASS) or arguments
        let user = "wsh";
        let pass = "1";

        let client = reqwest::Client::new();
        let mut res = client
            .post(&config.url_rpc)
            .basic_auth(user, Some(pass))
            .json(&json)
            .send()?;

        // TODO
        // match res.status() {
        // }

        match res.text() {
            Ok(x) => {
                info!("{}", x);
            }
            Err(x) => warn!("invalid response json: {}", x),
        }
        Ok(())
    }
}

#[derive(Default)]
struct PostData {
    url_rpc: String,
    attributes: HashMap<String, String>,
    notify: bool,
    // date_time: not implemented
}

// int ticket.create(
//     string summary, string description, struct attributes={},
//     boolean notify=False, DateTime when=None)
#[derive(Default)]
struct PostDataCreate {
    post_data: PostData,
    summary: String,
    description: String,
}

impl Poster for PostDataCreate {
    fn post(&self, config: &Config) -> Result<(), failure::Error> {
        let json = serde_json::json!({
            "method": "ticket.create",
            "params": [
                self.summary,
                self.description,
                self.post_data.attributes,
                self.post_data.notify,
                // when
            ],
        });

        let tmp = self.post_json(config, &json)?;

        // TODO: re-get, warn ignored like foo:bar

        Ok(())
    }
}

// array ticket.update(
//     int id, string comment, struct attributes={},
//     boolean notify=False, string author="", DateTime when=None)
#[derive(Default)]
struct PostDataUpdate {
    post_data: PostData,
    id: u32,
    comment: String,
    author: Option<String>, // TODO
}

impl Poster for PostDataUpdate {
    fn post(&self, config: &Config) -> Result<(), failure::Error> {
        let json = serde_json::json!({
            "method": "ticket.get",
            "params": [ self.id ],
        });

        // TODO: short log
        debug!("GET");
        debug!("{}", json.to_string());

        let client = reqwest::Client::new();
        let mut res = client
            .post(&config.url_rpc)
            .basic_auth(&config.trac_user, Some(&config.trac_pass))
            .json(&json)
            .send()?;

        // TODO
        // match res.status() {
        // }

        match res.text() {
            Ok(x) => {
                info!("{}", x);
            }
            Err(x) => warn!("invalid response json: {}", x),
        }

        let json = serde_json::json!({
            "method": "ticket.update",
            "params": [
                self.id,
                self.comment,
                self.post_data.attributes,
                self.post_data.notify,
                // TODO: author
                // when
            ],
        });

        let tmp = self.post_json(config, &json);

        // TODO: check result
        //   {"error": {"message": "ServiceException details : no such column: foo", "code": -32603,
        //    "name": "JSONRPCError"}, "result": null, "id": null}

        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------
// txt

fn txt_to_poster(config: &Config, txt: &str) -> Result<Box<dyn Poster>, failure::Error> {
    let (mut header, _linenum_body) = parse_header(txt)?;
    if let Some(x) = header.remove("description") {
        warn!("line {}: \"description\" is reserved key; ignored", x.0);
    }
    header_to_poster(config, header, txt)
}

// Returns Header and the line-number of the beginning of the body
fn parse_header(txt: &str) -> Result<(Header, usize), failure::Error> {
    if !txt.starts_with("{{{\n#!comment\n") {
        ret_e!(
            "text must start with {{{{{{\\n#!comment\\n, but given:\n\"{}\"",
            utils::partial_str(txt, 20)
        );
    }

    let mut header = Header::new();
    let mut i_line: usize = 2;
    for mut line in txt["{{{\n#!comment\n".len()..].lines() {
        i_line += 1;
        line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        if let Some(pos) = line.find('#') {
            line = &line[..pos].trim_end();
        }
        debug!("line {}: {}", i_line, line);
        if line == "}}}" {
            debug!("end of header");
            return Ok((header, i_line));
        }

        let (key, val) = parse_kv(i_line, line)?;
        if header.contains_key(key) {
            ret_e!("line {}: duplicate key: \"{}\"", i_line, key);
        }
        header.insert(key.to_string(), (i_line as i32, val.to_string()));
    }

    ret_e!("end-of-comment (\"}}}}}}\") is missing");
}

fn parse_kv(i_line: usize, line: &str) -> Result<(&str, &str), failure::Error> {
    let pos = match line.find(':') {
        None => {
            ret_e!("line {}: colon (:) not found; given: {}:", i_line, line);
        }
        Some(x) => x,
    };
    let key = line[..pos].trim();
    let val = line[pos + 1..].trim();
    Ok((key, val))
}

fn header_to_poster(
    config: &Config,
    mut header: Header,
    txt: &str,
) -> Result<Box<dyn Poster>, failure::Error> {
    // TODO: line-number-sequential logging
    if let Some(x) = header.remove("url_rpc") {
        if config.url_rpc_arg {
            info!(
                "line {}: \"url_rpc\" is ignored since --url-rpc={} is given",
                x.0, config.url_rpc
            );
        }
    } else {
        if !config.url_rpc_arg {
            ret_e!("please set --url-rpc");
        }
    }

    for key in ["id", "url"].iter() {
        if let Some(x) = header.get(*key) {
            if x.1.is_empty() {
                info!("line {}: \"{}\" is empty; ignores it", x.0, key);
                header.remove(*key);
            }
        }
    }
    if header.contains_key("id") || header.contains_key("url") {
        return Ok(Box::new(header_to_post_data_update(header, txt)?));
    }
    Ok(Box::new(header_to_post_data_create(header, txt)?))
}

fn header_to_post_data_create(
    mut header: Header,
    txt: &str,
) -> Result<PostDataCreate, failure::Error> {
    let summary = match header.remove("summary") {
        None => {
            ret_e!("please set \"summary: <summary>\"");
        }
        Some(x) => x.1,
    };

    Ok(PostDataCreate {
        post_data: header_to_post_data(&mut header)?,
        summary,
        description: txt.to_string(),
        ..Default::default()
    })
}

fn header_to_post_data_update(
    mut header: Header,
    txt: &str,
) -> Result<PostDataUpdate, failure::Error> {
    if !header.contains_key("id") && !header.contains_key("url") {
        ret_e!("BUG: id or url not set")
    }
    if header.contains_key("id") && header.contains_key("url") {
        warn!("both \"id\" and \"url\" is set; ignores \"url\"");
        header.remove("url");
    }

    // TODO: cleanup

    let mut data = PostDataUpdate {
        ..Default::default()
    };

    if header.contains_key("url") {
        ret_e!("TODO: url");
        // header.remove("url");
    }

    let id = header.remove("id").unwrap(); // TODO: error handling
    if id.1.is_empty() {
        info!("line {}: \"id\" is empty; create new ticket", id.0);
    } else {
        info!(
            "line {}: \"id\" is empty ({}); create new ticket",
            id.0, id.1
        );
        data.id = match id.1.parse() {
            Ok(x) => x,
            Err(x) => {
                ret_e!("TODO {}", x);
            }
        };
    }

    if let Some(x) = header.remove("comment") {
        data.comment = x.1;
    }

    data.post_data = header_to_post_data(&mut header)?;

    data.post_data
        .attributes
        .insert("description".to_string(), txt.to_string());

    Ok(data)
}

fn header_to_post_data(header: &mut Header) -> Result<PostData, failure::Error> {
    let mut data = PostData {
        notify: false, // TODO
        ..Default::default()
    };
    for (key, (_i_line, val)) in header {
        data.attributes.insert(key.to_string(), val.to_string());
    }
    Ok(data)
}

// -------------------------------------------------------------------------------------------------
// tests

// TODO: set logger
// TODO: test without TRAC_ADMIN
// TODO: test multi-root project

#[cfg(test)]
mod tests {
    // TODO: concat tests: create -> update

    // const config: crate::Config = crate::Config {
    //     trac_user: "wsh".to_string(),
    //     trac_pass: "1".to_string(),
    // };
    fn config() -> crate::Config {
        crate::Config {
            trac_user: "wsh".to_string(),
            trac_pass: "1".to_string(),
            url_rpc_arg: false,
            url_rpc: "http://localhost:8000/login/rpc".to_string(),
            interactive: false,
        }
    }

    // TOOD: test only if trac available
    #[test]
    fn test_post_create() {
        // create (url: (empty))
        let txt = textwrap::dedent(
            &r"
            {{{
            #!comment
            url_rpc: http://localhost:8000/login/rpc
            # url:
            
            summary: hi there
            
            # component: component1
            milestone: milestone1
            priority: blocker
            # resolution: fixed
            severity:
            # status: accepted
            type: defect
            
            foo: bar
            # owner:
            # description -> warning?
            }}}
            
            test test
            test
            "[1..],
        );
        let tmp = crate::post(&config(), &txt);
        // TODO
        let _breakpoint = 1;
    }

    #[test]
    #[ignore]
    fn test_post_upate() {
        // update (url: (ticket url))
        let txt = textwrap::dedent(
            &r"
            {{{
            #!comment
            url_rpc: http://localhost:8000/login/rpc
            id: 1
            # url: http://localhost:8000/ticket/1  # TODO
            
            summary: hi there
            
            component:
            milestone: milestone9999
            priority: blocker
            resolution: fixed
            severity:
            # status: accepted
            type: defect
            
            # foo: bar
            }}}
            
            test test
            test
            "[1..],
        );
        let tmp = crate::post(&config(), &txt);
        let txt = textwrap::dedent(
            &r"
            {{{
            #!comment
            url_rpc: http://localhost:8000/login/rpc
            id: 1
            
            summary: hi there
            
            component: component1    # +
            # milestone:             # unchanged
            priority: blocker
            # resolution: fixed
            severity:
            # status: accepted
            type: defect
            }}}
            
            test test
            test
            "[1..],
        );
        let tmp = crate::post(&config(), &txt);
        let _breakpoint = 1;
    }

    #[test]
    #[ignore]
    fn test_headers() {
        // TODO: assert error

        // invalid start
        let txt = textwrap::dedent(
            &r"
            invalid start
            {{{
            #!comment
            }}}"[1..],
        );
        let tmp = crate::post(&config(), &txt);

        // missing colong
        let txt = textwrap::dedent(
            &r"
            {{{
            #!comment
            missing colon
            }}}"[1..],
        );
        let tmp = crate::post(&config(), &txt);

        // duplicate
        let txt = textwrap::dedent(
            &r"
            {{{
            #!comment
            key1: val1
            key1: val2
            }}}"[1..],
        );
        let tmp = crate::post(&config(), &txt);

        // no url
        let txt = textwrap::dedent(
            &r"
            {{{
            #!comment
            }}}"[1..],
        );
        let tmp = crate::post(&config(), &txt);

        let _breakpoint = 1;
    }
}
