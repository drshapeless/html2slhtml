use std::fmt::Write;
use std::fs;
use tl::*;

fn remove_empty_lines(s: &str) -> String {
    let lines = s.lines();
    let non_empty_lines: Vec<&str> =
        lines.filter(|line| line.trim().len() > 0).collect();
    non_empty_lines.join("\n")
}

fn remove_first_and_last_line(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let out_str = lines[1..lines.len() - 1].join("\n");
    out_str
}

pub fn html2sl(html: &str) -> String {
    let mut sl = String::new();

    let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();

    fn spaces(count: usize) -> String {
        return "    ".repeat(count).as_str().to_owned();
    }

    fn handle_tag(
        tag: &HTMLTag,
        parser: &Parser,
        sl: &mut String,
        indent: usize,
    ) {
        let tag_name = tag.name().as_utf8_str();
        write!(sl, "{}{}()\n", spaces(indent), &tag_name).unwrap();
        let id = tag.attributes().id().map(|x| x.as_utf8_str());
        match &id {
            Some(x) => {
                write!(
                    sl,
                    "{}.id(\"{}\")\n",
                    spaces(indent + 1),
                    x.to_string()
                )
                .unwrap();
            }
            None => {}
        }

        match tag.attributes().class_iter() {
            None => {}
            Some(classes) => {
                for class in classes {
                    write!(
                        sl,
                        "{}.class(\"{}\")\n",
                        spaces(indent + 1),
                        class.to_owned()
                    )
                    .unwrap();
                }
            }
        }

        for (key, value_opt) in tag.attributes().iter() {
            if !(key.eq("class")) && !(key.eq("id")) {
                match value_opt {
                    None => write!(sl, "{}.{}()\n", spaces(indent + 1), key)
                        .unwrap(),
                    Some(value) => {
                        let mut key_str = key.to_string();
                        if key.eq("async") || key.eq("for") || key.eq("type") {
                            key_str = String::from("r#") + &key_str;
                        }
                        write!(
                            sl,
                            "{}.{}(\"{}\")\n",
                            spaces(indent + 1),
                            key_str,
                            value
                        )
                        .unwrap();
                    }
                }
            }
        }

        let children = tag.children();
        let nodes = children.top().as_slice();
        if !nodes.is_empty() {
            for child_node in nodes {
                handle_node(child_node.get(parser), parser, sl, indent + 1);
            }
        }
    }

    fn handle_node(
        node_opt: Option<&Node>,
        parser: &Parser,
        sl: &mut String,
        indent: usize,
    ) {
        match node_opt {
            None => {}
            Some(node) => match node {
                Node::Tag(tag) => {
                    write!(sl, "{}.child(\n", spaces(indent)).unwrap();
                    handle_tag(tag, parser, sl, indent);
                    write!(sl, "{})\n", spaces(indent)).unwrap();
                }
                Node::Comment(_) => {}
                Node::Raw(raw) => {
                    let text = raw.as_utf8_str();
                    let trimmed_text = text.trim();
                    if !trimmed_text.is_empty() {
                        write!(
                            sl,
                            "{}.child(\"{}\")\n",
                            spaces(indent),
                            trimmed_text
                        )
                        .unwrap();
                    }
                }
            },
        }
    }

    for node_handle in dom.children() {
        handle_node(node_handle.get(parser), parser, &mut sl, 1);
    }

    let s = remove_empty_lines(&sl);
    remove_first_and_last_line(s.as_str())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];

    let s = fs::read_to_string(filename).unwrap();

    println!("{}", html2sl(s.as_str()));
}
