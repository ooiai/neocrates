#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParseResult {
    pub page: u32,
    pub bbox: (f32, f32, f32, f32),
    pub typeid: u32,
    pub size: (f32, f32),
    pub text: String,
}

/// Merges ParseResult entries into chunks not exceeding max_len characters.
pub fn smart_chunks(results: Vec<ParseResult>, max_len: usize) -> Vec<ParseResult> {
    let mut merged_results = Vec::new();
    let mut buffer = String::new();
    let mut last_page = 0;
    let mut last_bbox = (0.0, 0.0, 0.0, 0.0);
    let mut last_typeid = 0;
    let mut last_size = (0.0, 0.0);

    for result in results {
        let mut text = result.text;
        while text.chars().count() > max_len {
            let segment: String = text.chars().take(max_len).collect();
            if !buffer.is_empty() {
                merged_results.push(ParseResult {
                    page: last_page,
                    bbox: last_bbox,
                    typeid: last_typeid,
                    size: last_size,
                    text: buffer.clone(),
                });
                buffer.clear();
            }
            merged_results.push(ParseResult {
                page: result.page,
                bbox: result.bbox,
                typeid: result.typeid,
                size: result.size,
                text: segment,
            });
            text = text.chars().skip(max_len).collect();
        }

        if buffer.chars().count() + text.chars().count() > max_len {
            if !buffer.is_empty() {
                merged_results.push(ParseResult {
                    page: last_page,
                    bbox: last_bbox,
                    typeid: last_typeid,
                    size: last_size,
                    text: buffer.clone(),
                });
                buffer.clear();
            }
        }
        if !text.is_empty() {
            if buffer.is_empty() {
                last_page = result.page;
                last_bbox = result.bbox;
                last_typeid = result.typeid;
                last_size = result.size;
            }
            buffer.push_str(&text);
        }
    }

    if !buffer.is_empty() {
        merged_results.push(ParseResult {
            page: last_page,
            bbox: last_bbox,
            typeid: last_typeid,
            size: last_size,
            text: buffer,
        });
    }

    merged_results
}

//let new_vec = smart_merge_parse_results(result, 512);
