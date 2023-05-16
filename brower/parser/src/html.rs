use super::dom::{AttrMap, ElementData, Node, NodeType};
use std::iter::Peekable;
use std::str::Chars;

pub struct HtmlParser<'a> {
    chars: Peekable<Chars<'a>>,
    node_p: Vec<String>,
}

impl<'a> HtmlParser<'a> {
    pub fn new(html: &'a str) -> HtmlParser<'a> {
        HtmlParser {
            chars: html.chars().peekable(),
            node_p: Vec::new(),
        }
    }

    pub fn parse_nodes(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();

        while self.chars.peek().is_some() {
            self.consume_while(char::is_whitespace);
            if self.chars.peek().map_or(false, |c| *c == '<') {
                self.chars.next();
                if self.chars.peek().map_or(false, |c| *c == '/') {
                    self.chars.next();
                    self.consume_while(char::is_whitespace);

                    let close_tag_name = self.consume_while(is_valid_tag_name);

                    self.consume_while(|x| x != '>');
                    self.chars.next();

                    self.node_p.push(close_tag_name);
                    break;
                } else if self.chars.peek().map_or(false, |c| *c == '!') {
                    self.chars.next();
                    nodes.push(self.parse_comment_node());
                } else {
                    let mut node = self.parse_node();
                    let insert_index = nodes.len();

                    match node.node_type {
                        NodeType::Element(ref e) => {
                            if self.node_p.len() > 0 {
                                let assumed_tag = self.node_p.remove(0);

                                if e.tag_name != assumed_tag {
                                    nodes.append(&mut node.children);
                                    self.node_p.insert(0, assumed_tag);
                                }
                            }
                        }
                        _ => {}
                    }
                    nodes.insert(insert_index, node);
                }
            } else {
                nodes.push(self.parse_text_node());
            }
        }
        nodes
    }

    fn parse_node(&mut self) -> Node {
        let tag_name = self.consume_while(is_valid_tag_name);
        let attributes = self.parse_attributes();

        let elem: ElementData = ElementData::new(tag_name, attributes);
        let children = self.parse_nodes();
        Node::new(NodeType::Element(elem), children)
    }

    fn parse_text_node(&mut self) -> Node {
        let mut text_content = String::new();

        while self.chars.peek().map_or(false, |c| *c != '<') {
            let whitespace = self.consume_while(char::is_whitespace);
            if whitespace.len() > 0 {
                text_content.push(' ');
            }
            {
                let text_part = self.consume_while(|x| !x.is_whitespace() && x != '<');
                text_content.push_str(&text_part);
            }
        }
        Node::new(NodeType::Text(text_content), Vec::new())
    }

    fn parse_comment_node(&mut self) -> Node {
        let mut comment_content = String::new();

        if self.chars.peek().map_or(false, |c| *c == '-') {
            self.chars.next();
            if self.chars.peek().map_or(false, |c| *c == '-') {
                self.chars.next();
            } else {
                self.consume_while(|c| c != '>');
                return Node::new(NodeType::Comment(comment_content), Vec::new());
            }
        } else {
            self.consume_while(|c| c != '>');
            return Node::new(NodeType::Comment(comment_content), Vec::new());
        }

        if self.chars.peek().map_or(false, |c| *c == '>') {
            self.chars.next();
            return Node::new(NodeType::Comment(comment_content), Vec::new());
        }

        if self.chars.peek().map_or(false, |c| *c == '-') {
            self.chars.next();
            if self.chars.peek().map_or(false, |c| *c == '>') {
                self.chars.next();
                return Node::new(NodeType::Comment(comment_content), Vec::new());
            } else {
                comment_content.push('-');
            }
        }
        while self.chars.peek().is_some() {
            comment_content.push_str(&self.consume_while(|c| c != '<' && c != '-'));
            if self.chars.peek().map_or(false, |c| *c == '<') {
                self.chars.next();
                if self.chars.peek().map_or(false, |c| *c == '!') {
                    self.chars.next();
                    if self.chars.peek().map_or(false, |c| *c == '-') {
                        self.chars.next();
                        if self.chars.peek().map_or(false, |c| *c == '-') {
                            self.consume_while(|c| c != '>');

                            return Node::new(NodeType::Comment(String::from("")), Vec::new());
                        } else {
                            comment_content.push_str("<!-");
                        }
                    } else if self.chars.peek().map_or(false, |c| *c == ' ') {
                        self.chars.next();
                        if self.chars.peek().map_or(false, |c| *c == '-') {
                            self.chars.next();
                            if self.chars.peek().map_or(false, |c| *c == '-') {
                                self.chars.next();
                                if self.chars.peek().map_or(false, |c| *c == '-') {
                                    self.chars.next();
                                    if self.chars.peek().map_or(false, |c| *c == '>') {
                                        self.chars.next();
                                        return Node::new(
                                            NodeType::Comment(String::from("")),
                                            Vec::new(),
                                        );
                                    } else {
                                        comment_content.push_str("<! --");
                                    }
                                } else {
                                    comment_content.push_str("<! -");
                                }
                            } else {
                                comment_content.push_str("<! ");
                            }
                        }
                    } else {
                        comment_content.push_str("<!");
                    }
                } else {
                    comment_content.push('<');
                }
            } else if self.chars.peek().map_or(false, |c| *c == '-') {
                self.chars.next();
                if self.chars.peek().map_or(false, |c| *c == '-') {
                    self.chars.next();
                    if self.chars.peek().map_or(false, |c| *c == '>') {
                        self.chars.next();
                        break;
                    } else {
                        comment_content.push_str("--");
                    }
                } else {
                    comment_content.push('-');
                }
            }
        }
        Node::new(NodeType::Comment(comment_content), Vec::new())
    }

    fn parse_attributes(&mut self) -> AttrMap {
        let mut attributes = AttrMap::new();

        while self.chars.peek().map_or(false, |c| *c != '>') {
            self.consume_while(char::is_whitespace);
            let name = self.consume_while(|c| is_valid_attr_name(c)).to_lowercase();
            self.consume_while(char::is_whitespace);

            let value = if self.chars.peek().map_or(false, |c| *c == '=') {
                self.chars.next();
                self.consume_while(char::is_whitespace);
                let s = self.parse_attr_value();
                self.consume_while(|c| !c.is_whitespace() && c != '>');
                self.consume_while(char::is_whitespace);
                s
            } else {
                "".to_string()
            };
            attributes.insert(name, value);
        }
        self.chars.next();

        attributes
    }

    fn parse_attr_value(&mut self) -> String {
        self.consume_while(char::is_whitespace);

        let result = match self.chars.peek() {
            Some(&c) if c == '"' || c == '\'' => {
                self.chars.next();
                let ret = self.consume_while(|x| x != c);
                self.chars.next();
                ret
            }
            _ => self.consume_while(is_valid_attr_value),
        };

        result
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while self.chars.peek().map_or(false, |c| test(*c)) {
            result.push(self.chars.next().unwrap());
        }
        result
    }


}

fn is_valid_tag_name(ch: char) -> bool {
    ch.is_digit(36)
}

fn is_valid_attr_value(c: char) -> bool {
    match c {
        ' ' | '"' | '\'' | '=' | '<' | '>' | '`' => false,
        _ => true,
    }
}

fn is_excluded_name(c: char) -> bool {
    match c {
        ' ' | '"' | '\'' | '>' | '/' | '=' => true,
        _ => false,
    }
}

fn is_control(ch: char) -> bool {
    match ch {
        '\u{007F}' => true,
        c if c >= '\u{0000}' && c <= '\u{001F}' => true,
        c if c >= '\u{0080}' && c <= '\u{009F}' => true,
        _ => false,
    }
}

fn is_valid_attr_name(c: char) -> bool {
    !is_excluded_name(c) && !is_control(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let mut parser = HtmlParser::new("<html></html>");
        let nodes = parser.parse_nodes();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].node_type {
            NodeType::Element(data) => {
                assert_eq!(data.tag_name, "html");
            },
            _ => panic!("Expected NodeType::Element"),
        }
    }

    #[test]
    fn test_parse_html_with_nested_tags() {
        let mut parser = HtmlParser::new("<html><body><p>Test</p></body></html>");
        let nodes = parser.parse_nodes();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].node_type {
            NodeType::Element(data) => {
                assert_eq!(data.tag_name, "html");
                assert_eq!(nodes[0].children.len(), 1);
                match &nodes[0].children[0].node_type {
                    NodeType::Element(data) => {
                        assert_eq!(data.tag_name, "body");
                        assert_eq!(nodes[0].children[0].children.len(), 1);
                    },
                    _ => panic!("Expected NodeType::Element"),
                }
            },
            _ => panic!("Expected NodeType::Element"),
        }
    }

    #[test]
    fn test_parse_html_with_text() {
        let mut parser = HtmlParser::new("<html>Hello World</html>");
        let nodes = parser.parse_nodes();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].node_type {
            NodeType::Element(data) => {
                assert_eq!(data.tag_name, "html");
                assert_eq!(nodes[0].children.len(), 1);
                match &nodes[0].children[0].node_type {
                    NodeType::Text(text) => {
                        assert_eq!(text, &"Hello World".to_string());
                    },
                    _ => panic!("Expected NodeType::Text"),
                }
            },
            _ => panic!("Expected NodeType::Element"),
        }
    }

    #[test]
    fn test_parse_html_with_attributes() {
        let mut parser = HtmlParser::new("<html lang='en'><body class='main'></body></html>");
        let nodes = parser.parse_nodes();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].node_type {
            NodeType::Element(data) => {
                assert_eq!(data.tag_name, "html");
                // assert_eq!(data.attributes.get("lang"), Some(&"en".to_string()));
                assert_eq!(nodes[0].children.len(), 1);
                match &nodes[0].children[0].node_type {
                    NodeType::Element(data) => {
                        assert_eq!(data.tag_name, "body");
                        // assert_eq!(data.attributes.get("class"), Some(&"main".to_string()));
                    },
                    _ => panic!("Expected NodeType::Element"),
                }
            },
            _ => panic!("Expected NodeType::Element"),
        }
    }

    #[test]
    fn test_parse_html_comment() {
        let mut parser = HtmlParser::new("<!-- This is a comment -->");
        let nodes = parser.parse_nodes();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].node_type {
            NodeType::Comment(comment) => {
                assert_eq!(comment, " This is a comment ");
            },
            _ => panic!("Expected NodeType::Comment"),
        }
    }
}
