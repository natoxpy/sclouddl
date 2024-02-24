use crate::consts::SOUNDCLOUD_BASE;
use regex::Regex;

fn get_client_id(str: String) -> Option<String> {
    let reg_client_id = Regex::new("client_id:\"\\w+\"").unwrap();
    let mut locs = reg_client_id.capture_locations();
    reg_client_id.captures_read(&mut locs, &str)?;
    let loc = locs.get(0)?;
    let sc = &str[loc.0..loc.1];

    let client_id = sc.replace("client_id:\"", "").replace("\"", "");
    Some(client_id)
}

pub async fn gen_key() -> Option<String> {
    let req = reqwest::get(SOUNDCLOUD_BASE)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let reg_link = Regex::new(r"^(https://).*(js)$").unwrap();
    let scripts_split = req
        .split("<script crossorigin src=\"")
        .map(|splt| {
            splt.to_string()
                .split("\"></script>")
                .collect::<Vec<&str>>()
                .get(0)
                .unwrap_or(&"")
                .to_string()
        })
        .filter(|splt| reg_link.is_match(splt))
        .collect::<Vec<String>>();

    for script_split_link in scripts_split.iter() {
        let sreq = reqwest::get(script_split_link)
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        if let Some(client_id) = get_client_id(sreq) {
            return Some(client_id);
        }
    }

    None
}
