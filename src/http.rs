use std::collections::HashMap;

use super::utils;

// array ticket.get(int id)
// Fetch a ticket. Returns [id, time_created, time_changed, attributes].

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Ticket {
    error: serde_json::Value,  // null
    id: serde_json::Value,     // null
    result: serde_json::Value, // array
}

pub(crate) fn ticket_get(
    config: &crate::Config,
    id: u32,
) -> Result<HashMap<String, String>, failure::Error> {
    let json = serde_json::json!({
        "method": "ticket.get",
        "params": [ id ],
    });
    let text = post_json(&config, &json)?;

    let ticket = match serde_json::from_str::<Ticket>(&text) {
        Ok(x) => x,
        Err(x) => {
            ret_e!("failed to deserialize json: {}", x);
        }
    };
    // debug!("{:#?}", ticket);
    // debug!("{}", serde_json::to_string_pretty(&ticket)?);

    // let _tmp = &ticket.result[0]; // serde_json::Value::Number id
    // let _tmp = &ticket.result[1]; // serde_json::Value::Object time_created
    // let _tmp = &ticket.result[2]; // serde_json::Value::Object time_changed
    let attributes = &ticket.result[3]; // serde_json::Value::Object attributes
    let obj = match attributes.as_object() {
        None => {
            ret_e!("unexpected json from Trac json-rpc server: .result[3] is not object");
        }
        Some(x) => x,
    };

    // TODO: obj.iter().map();
    let mut ret = HashMap::new();
    for (key, val) in obj {
        let s = match val.as_str() {
            None => {
                if key == "changetime" || key == "time" {
                    continue;
                }
                warn!(
                    "unexpected json from Trac json-rpc server: .result[3].{}: {}",
                    key, val
                );
                continue;
            }
            Some(x) => x,
        };
        ret.insert(key.to_string(), s.to_string());
    }

    Ok(ret)
}

pub(crate) fn post_json(
    config: &crate::Config,
    json: &serde_json::Value,
) -> Result<String, failure::Error> {
    // TODO: short log
    info!("--> {}", utils::partial_str(json.to_string().as_ref(), 50));

    let url_rpc = match &config.url_rpc {
        None => panic!("BUG: url_rpc not set"),
        Some(x) => x.as_str(),
    };
    let resp = match reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // TODO: config
        .build()
        .unwrap()
        .post(url_rpc)
        .basic_auth(&config.trac_user, Some(&config.trac_pass))
        .json(&json)
        .send()
    {
        Ok(x) => x,
        Err(x) => ret_e!("failed to POST to Trac json-rpc server: {}", x),
    };

    match resp.status() {
        reqwest::StatusCode::OK => {}
        x => {
            ret_e!("POST failed: {}", x);
        }
    };

    Ok(text(resp)?)
}

fn text(mut resp: reqwest::Response) -> Result<String, failure::Error> {
    let text = match resp.text() {
        Ok(x) => {
            debug!("<-- {}", utils::partial_str(x.as_ref(), 50));
            x
        }
        Err(x) => {
            // error!("TODO {}", x);
            // return Err(x);
            ret_e!("TODO {}", x);
        }
    };
    Ok(text)
}
