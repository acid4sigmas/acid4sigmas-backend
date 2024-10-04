use std::{fs::File, io::Read, path::PathBuf};

use actix_web::{get, HttpResponse};

use actix_web_lab::extract::Path;

use crate::{
    error_response,
    util::html_utils::{inject_hr_after_block_elements, markdown_to_html},
};

use serde::Deserialize;

#[derive(Deserialize)]
struct QueryParams {
    lang: Option<String>,
}

#[get("/faith/book/{chapter_id}")]
pub async fn faith_book(
    Path(chapter_id): Path<String>,
    query: actix_web::web::Query<QueryParams>,
) -> HttpResponse {
    println!("id: {}", chapter_id);

    let mut lang = query.lang.as_deref().unwrap_or("en");

    if lang != "de" {
        lang = "en"
    }

    let path = PathBuf::from(format!("./assets/faith_book/{}/{}.md", lang, chapter_id));

    println!("path: {:?}", path);

    if !path.exists() {
        return error_response!(404, "couldnt find this chapter");
    }

    let mut file = File::open(path).unwrap();

    let mut content = String::new();

    File::read_to_string(&mut file, &mut content).unwrap();

    let html = markdown_to_html(&content);

    let html_result = inject_hr_after_block_elements(&html);

    println!("result: {}", html_result);

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_result)
}
