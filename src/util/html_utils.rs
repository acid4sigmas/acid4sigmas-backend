use kuchiki::parse_html;
use kuchiki::traits::*;
use markup5ever::local_name;
use markup5ever::namespace_url;
use markup5ever::ns;
use pulldown_cmark::{html, Options, Parser};
use regex::Regex;

pub fn inject_hr_after_block_elements(html: &str) -> String {
    let document = parse_html().one(html);

    let selectors = vec!["p", "h1", "h2", "h3", "ul"];

    for selector in selectors {
        let elements = document.select(selector).unwrap();
        for node in elements {
            let hr_element = kuchiki::NodeRef::new_element(
                markup5ever::interface::QualName::new(None, ns!(html), local_name!("hr")),
                vec![].into_iter().collect::<Vec<(_, _)>>(),
            );
            node.as_node().insert_after(hr_element);
        }
    }

    let mut serialized_html = Vec::new();
    document.serialize(&mut serialized_html).unwrap();

    String::from_utf8(serialized_html).unwrap()
}

pub fn markdown_to_html(markdown_input: &str) -> String {
    let parser = Parser::new_ext(markdown_input, Options::all());
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let re = Regex::new(r"(?m)<p>\s*</p>").unwrap();

    html_output = re.replace_all(&html_output, "<br/>").to_string();
    println!("coke: {:?}", html_output);
    html_output
}
