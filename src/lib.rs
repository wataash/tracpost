// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(unused_variables)]

use std::collections::HashMap;

#[macro_use]
mod utils;

// TODO: --comment='update foo' --attr=foo:bar
//   when create: commment='update foo' is ignored on create

// -------------------------------------------------------------------------------------------------
// post

// key - (line, value)
type Header = HashMap<String, (i32, String)>;

trait Poster {
    fn post(&self) -> Result<(), failure::Error>;

    fn post_json(&self, url_rpc: &str, json: &serde_json::Value) -> Result<(), failure::Error> {
        // TODO: short log
        debug!("{}", json.to_string());

        // TODO: env (TRACPOST_USER TRACPOST_PASS) or arguments
        let user = "wsh";
        let pass = "1";

        let client = reqwest::Client::new();
        let mut res = client
            .post(url_rpc)
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

// TODO: String?
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
    fn post(&self) -> Result<(), failure::Error> {
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

        let tmp = self.post_json(&self.post_data.url_rpc, &json)?;

        // TODO: check result

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
    fn post(&self) -> Result<(), failure::Error> {
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

        let tmp = self.post_json(&self.post_data.url_rpc, &json);

        // TODO: check result
        //   {"error": {"message": "ServiceException details : no such column: foo", "code": -32603,
        //    "name": "JSONRPCError"}, "result": null, "id": null}

        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------
// txt

fn txt_to_poster(txt: &str) -> Result<Box<dyn Poster>, failure::Error> {
    let (mut header, _linenum_body) = parse_header(txt)?;
    if let Some(x) = header.remove("description") {
        warn!("line {}: \"description\" is reserved key; ignored", x.0);
    }
    header_to_poster(header, txt)
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

fn header_to_poster(kvs: Header, txt: &str) -> Result<Box<dyn Poster>, failure::Error> {
    if kvs.contains_key("id") || kvs.contains_key("url") {
        return Ok(Box::new(header_to_post_data_update(kvs, txt)?));
    }
    Ok(Box::new(header_to_post_data_create(kvs, txt)?))
}

fn header_to_post_data_create(
    mut kvs: Header,
    txt: &str,
) -> Result<PostDataCreate, failure::Error> {
    let summary = match kvs.remove("summary") {
        None => {
            ret_e!("please set \"summary: <summary>\"");
        }
        Some(x) => x.1,
    };

    Ok(PostDataCreate {
        post_data: header_to_post_data(&mut kvs)?,
        summary,
        description: txt.to_string(),
        ..Default::default()
    })
}

fn header_to_post_data_update(
    mut kvs: Header,
    txt: &str,
) -> Result<PostDataUpdate, failure::Error> {
    if !kvs.contains_key("id") && !kvs.contains_key("url") {
        ret_e!("BUG: id or url not set")
    }
    if kvs.contains_key("id") && kvs.contains_key("url") {
        warn!("both \"id\" and \"url\" is set; ignores \"url\"");
        kvs.remove("url");
    }

    // TODO: cleanup

    let mut data = PostDataUpdate {
        ..Default::default()
    };

    if kvs.contains_key("url") {
        ret_e!("TODO: url");
        // kvs.remove("url");
    }

    // data.id = kvs.remove("id").unwrap().1.into(); // TODO: if invalid number
    let tmp = kvs.remove("id").unwrap().1; // TODO: if invalid number
    data.id = match tmp.parse() {
        Ok(x) => x,
        Err(x) => {
            ret_e!("TODO");
        }
    };

    if let Some(x) = kvs.remove("comment") {
        data.comment = x.1;
    }

    data.post_data = header_to_post_data(&mut kvs)?;

    data.post_data
        .attributes
        .insert("description".to_string(), txt.to_string());

    Ok(data)
}

fn header_to_post_data(kvs: &mut Header) -> Result<PostData, failure::Error> {
    let mut data = PostData {
        notify: false, // TODO
        ..Default::default()
    };

    data.url_rpc = match kvs.remove("url_rpc") {
        None => {
            ret_e!("please set \"url_rpc: <URL>\"");
        }
        Some(x) => x.1,
    };

    for (key, (_i_line, val)) in kvs {
        data.attributes.insert(key.to_string(), val.to_string());
    }

    Ok(data)
}

// -------------------------------------------------------------------------------------------------
// api

pub fn post(txt: &str) -> Result<(), failure::Error> {
    let poster = txt_to_poster(txt)?;
    poster.post()?;
    Ok(())
}

// -------------------------------------------------------------------------------------------------
// main

// TODO: move to main.rs
// TODO: clap

pub fn main() {
    // let tmp = post(&txt);
    // TODO
    let _breakpoint = 1;
}

// -------------------------------------------------------------------------------------------------
// tests

// TODO: set logger
// TODO: test without TRAC_ADMIN
// TODO: test multi-root project

#[cfg(test)]
mod tests {
    // TODO: concat tests: create -> update

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
        let tmp = crate::post(&txt);
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
        let tmp = crate::post(&txt);
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
        let tmp = crate::post(&txt);
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
        let tmp = crate::post(&txt);

        // missing colong
        let txt = textwrap::dedent(
            &r"
            {{{
            #!comment
            missing colon
            }}}"[1..],
        );
        let tmp = crate::post(&txt);

        // duplicate
        let txt = textwrap::dedent(
            &r"
            {{{
            #!comment
            key1: val1
            key1: val2
            }}}"[1..],
        );
        let tmp = crate::post(&txt);

        // no url
        let txt = textwrap::dedent(
            &r"
            {{{
            #!comment
            }}}"[1..],
        );
        let tmp = crate::post(&txt);

        let _breakpoint = 1;
    }
}
