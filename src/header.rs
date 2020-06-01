use super::utils;

// -------------------------------------------------------------------------------------------------
// header

// key - (line, value)
// TODO: avoid magic: .0 .1; name it
pub(crate) type Header = std::collections::HashMap<String, (i32, String)>;

// Returns Header and the line-number of the beginning of the body
pub(crate) fn parse(txt: &str) -> Result<(Header, usize), failure::Error> {
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
        if let Some(pos) = line.find("# ") {
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
