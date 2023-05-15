use std::collections::{HashMap, HashSet};
use std::fmt::{self};

pub type AttrMap = HashMap<String, String>;

#[derive(PartialEq, Eq, Clone)]
pub struct ElementData {
    pub tag_name: String,
    attributes: AttrMap,
}

#[derive(PartialEq, Eq, Clone)]
pub enum NodeType {
    Text(String),
    Element(ElementData),
    Comment(String),
}

#[derive(PartialEq, Eq, Clone)]
pub struct Node {
    pub children: Vec<Node>,
    pub node_type: NodeType,
}

impl ElementData {
    pub fn new(tag_name: String, attributes: AttrMap) -> ElementData {
        ElementData {
            tag_name,
            attributes,
        }
    }

    pub fn get_id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    pub fn get_classes(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            Some(classlist) => classlist.split(' ').collect(),
            None => HashSet::new(),
        }
    }
}

impl Node {
    pub fn new(node_type: NodeType, children: Vec<Node>) -> Node {
        Node {
            children,
            node_type,
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.node_type)
    }
}

impl fmt::Debug for ElementData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut attributes = String::new();
        for (key, value) in self.attributes.iter() {
            attributes.push_str(&format!(" {}=\"{}\"", key, value));
        }
        write!(f, "<{},{}>", self.tag_name, attributes)
    }
}

impl fmt::Debug for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NodeType::Text(ref t) | NodeType::Comment(ref t) => write!(f, "{}", t),
            NodeType::Element(ref e) => write!(f, "{:?}", e),
        }
    }
}

pub fn pretty_print(n:&Node,ident_size:usize){
    let indent = (0..ident_size).map(|_| " ").collect::<String>();

    match n.node_type{
        NodeType::Text(ref t) => println!("{}{}",indent,t),
        NodeType::Comment(ref t) => println!("{}<!--{}-->",indent,t),
        NodeType::Element(ref e) => {
            println!("{}{:?}",indent,e);
            for child in &n.children{
                pretty_print(child,ident_size+4);
            }
        }
    }

    for child in &n.children{
        pretty_print(child,ident_size+2);
    }

    match n.node_type{
        NodeType::Element(ref e) => println!("{}<{}/>",indent,e.tag_name),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_data_new() {
        let mut attrs = AttrMap::new();
        attrs.insert("id".to_string(), "test-id".to_string());
        attrs.insert("class".to_string(), "test-class".to_string());

        let element = ElementData::new("div".to_string(), attrs);

        assert_eq!(element.tag_name, "div");
        assert_eq!(element.get_id(), Some(&"test-id".to_string()));
        assert_eq!(element.get_classes(), ["test-class"].iter().cloned().collect());
    }

    #[test]
    fn test_node_new() {
        let text_node = Node::new(NodeType::Text("hello".to_string()), vec![]);
        assert_eq!(text_node.node_type, NodeType::Text("hello".to_string()));

        let mut attrs = AttrMap::new();
        attrs.insert("id".to_string(), "test-id".to_string());

        let element_node = Node::new(
            NodeType::Element(ElementData::new("div".to_string(), attrs.clone())),
            vec![text_node.clone()],
        );

        assert_eq!(element_node.node_type, NodeType::Element(ElementData::new("div".to_string(), attrs.clone())));
        assert_eq!(element_node.children, vec![text_node]);
    }

    #[test]
    fn test_pretty_print() {
        let text_node = Node::new(NodeType::Text("hello".to_string()), vec![]);

        let mut attrs = AttrMap::new();
        attrs.insert("id".to_string(), "test-id".to_string());

        let element_node = Node::new(
            NodeType::Element(ElementData::new("div".to_string(), attrs)),
            vec![text_node.clone()],
        );

        // 使用println!的测试可能较为复杂，因为它输出到控制台。这个测试主要是为了确保pretty_print没有panic。
        pretty_print(&element_node, 0);
    }
}

