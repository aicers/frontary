use std::include_bytes;

pub fn static_files() -> Vec<(&'static str, &'static [u8])> {
    let mut rtn: Vec<(&'static str, &'static [u8])> = Vec::new();
    let theme = include_bytes!("../static/frontary/theme.css");
    rtn.push(("theme.css", theme));

    rtn
}
