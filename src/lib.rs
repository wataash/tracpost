// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

#[macro_use]
pub mod utils;

mod header;
mod http;

// TODO: --comment='update foo' --attr=foo:bar --attr=fizz:buzz
//   when create: commment='update foo' is ignored on create

// TODO: line-context warner: implement line-log queue

// -------------------------------------------------------------------------------------------------
// api

#[derive(Debug, Default)]
pub struct Config {
    pub trac_user: String,
    pub trac_pass: String,
    pub url_rpc: Option<String>, // priority: arg (--url-rpc) > "url_rpc" in file
    pub update_comment: Option<String>,
    pub test_mode: bool,
}

pub fn post(mut config: Config, txt: &str) -> Result<(), failure::Error> {
    let poster = txt_to_poster(&mut config, txt)?;
    poster.post(&config)?;
    Ok(())
}

// -------------------------------------------------------------------------------------------------
// data

trait Poster {
    fn post(&self, config: &Config) -> Result<(), failure::Error>;
}

#[derive(Default)]
struct PostData {
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

impl PostDataCreate {
    fn diff_ticket(&self, _config: &Config) -> Result<(), failure::Error> {
        let mut ticket = HashMap::<String, String>::new();
        ticket.insert("description".to_string(), "".to_string());
        let mut ticket_new = self.post_data.attributes.clone();
        ticket_new.insert("description".to_string(), self.description.to_string());
        let _tmp = diff_ticket(&ticket, &ticket_new);
        Ok(())
    }
}

impl Poster for PostDataCreate {
    fn post(&self, config: &Config) -> Result<(), failure::Error> {
        self.diff_ticket(&config)?;
        if !config.test_mode {
            utils::y_n("create?")?;
        }

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

        let _tmp = http::post_json(config, &json)?;

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
    // author: Option<String>, // TODO
}

impl PostDataUpdate {
    fn diff_ticket(&self, config: &Config) -> Result<(), failure::Error> {
        let ticket = http::ticket_get(&config, self.id)?;
        let _tmp = diff_ticket(&ticket, &self.post_data.attributes);
        Ok(())
    }
}

impl Poster for PostDataUpdate {
    fn post(&self, config: &Config) -> Result<(), failure::Error> {
        self.diff_ticket(&config)?;
        if !config.test_mode {
            utils::y_n("update?")?;
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

        let _tmp = http::post_json(config, &json);

        // TODO: check result
        //   {"error": {"message": "ServiceException details : no such column: foo", "code": -32603,
        //    "name": "JSONRPCError"}, "result": null, "id": null}

        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------
// data - txt

fn txt_to_poster(config: &mut Config, txt: &str) -> Result<Box<dyn Poster>, failure::Error> {
    let (mut header, _linenum_body) = header::parse(txt)?;
    if let Some(x) = header.remove("description") {
        warn!("line {}: \"description\" is reserved key; ignored", x.0);
    }
    header_to_poster(config, header, txt)
}

fn header_to_poster(
    mut config: &mut Config,
    mut header: header::Header,
    txt: &str,
) -> Result<Box<dyn Poster>, failure::Error> {
    // TODO: line-number-sequential logging

    match header.remove("url_rpc") {
        None => {
            if let None = config.url_rpc {
                ret_e!("please set --url-rpc");
            }
        }
        Some(x) => match &mut config.url_rpc {
            None => {
                config.url_rpc = Some(x.1);
            }
            Some(y) => {
                warn!(
                    "line {}: \"url_rpc\" is ignored since --url-rpc={} is given",
                    x.0, y
                );
            }
        },
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
    mut header: header::Header,
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
    mut header: header::Header,
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
        debug!("line {}: \"id\" is empty; create new ticket", id.0);
    } else {
        debug!("line {}: \"id:{}\"; update", id.0, id.1);
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

fn header_to_post_data(header: &mut header::Header) -> Result<PostData, failure::Error> {
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
// data - diff

fn diff_ticket(a: &HashMap<String, String>, b: &HashMap<String, String>) {
    let mut a_sorted: Vec<_> = a.iter().collect();
    a_sorted.sort_by(|x, y| x.0.cmp(&y.0));
    let mut b_sorted: Vec<_> = b.iter().collect();
    b_sorted.sort_by(|x, y| x.0.cmp(&y.0));

    let desc_a = a.get("description").expect("BUG: a.description not set");
    let desc_b = b.get("description").expect("BUG: b.description not set");

    let mut a_string = String::new();
    for (key, val) in a_sorted {
        if key == "description" {
            continue;
        }
        a_string.push_str(&format!("{}: {}\n", key, val));
    }
    a_string.push_str("\n--------------------------------------------------\n");
    a_string.push_str(desc_a);

    let mut b_string = String::new();
    for (key, val) in b_sorted {
        if key == "description" {
            continue;
        }
        b_string.push_str(&format!("{}: {}\n", key, val));
    }
    b_string.push_str("\n--------------------------------------------------\n");
    b_string.push_str(desc_b);

    std::fs::write("/tmp/tracpost.a", &a_string).expect("TODO: failed a");
    std::fs::write("/tmp/tracpost.b", &b_string).expect("TODO: failed b");
    // TODO: remove icdiff
    debug!("icdiff /tmp/tracpost.a /tmp/tracpost.b");
    let _output = std::process::Command::new("icdiff")
        .arg("/tmp/tracpost.a")
        .arg("/tmp/tracpost.b")
        .stdout(std::process::Stdio::inherit())
        .output()
        .expect("Failed to execute command");
}
