use quick_xml::events::Event;
use quick_xml::Reader;

/// An element of a parsed SOAP response: local name, text and children.
/// Namespaces are dropped, since the local names in a response are unique.
#[derive(Debug, Default)]
pub(super) struct XmlNode {
    name: String,
    pub(super) text: String,
    children: Vec<XmlNode>,
}

impl XmlNode {
    pub(super) fn parse(xml: &str) -> Result<XmlNode, String> {
        let mut reader = Reader::from_str(xml);
        let mut stack: Vec<XmlNode> = vec![XmlNode::default()];
        loop {
            match reader.read_event() {
                Ok(Event::Start(start)) => stack.push(XmlNode {
                    name: local_name(start.local_name().as_ref()),
                    ..Default::default()
                }),
                Ok(Event::Empty(empty)) => {
                    let node = XmlNode {
                        name: local_name(empty.local_name().as_ref()),
                        ..Default::default()
                    };
                    attach(&mut stack, node)?;
                }
                Ok(Event::Text(text)) => {
                    let value = text.unescape().map_err(|e| e.to_string())?;
                    if let Some(current) = stack.last_mut() {
                        current.text.push_str(&value);
                    }
                }
                Ok(Event::End(_)) => {
                    let node = stack.pop().ok_or("XML non bilanciato")?;
                    attach(&mut stack, node)?;
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => return Err(format!("Risposta XML non valida: {e}")),
            }
        }
        stack.pop().ok_or_else(|| "XML vuoto".to_string())
    }

    pub(super) fn child(&self, name: &str) -> Option<&XmlNode> {
        self.children.iter().find(|c| c.name == name)
    }

    pub(super) fn children_named<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a XmlNode> + 'a {
        self.children.iter().filter(move |c| c.name == name)
    }

    pub(super) fn text_of(&self, name: &str) -> Option<String> {
        self.child(name)
            .map(|c| c.text.trim().to_string())
            .filter(|t| !t.is_empty())
    }

    pub(super) fn find(&self, name: &str) -> Option<&XmlNode> {
        if self.name == name {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(name))
    }

    /// The first node, depth first, that has a direct child called `child`.
    pub(super) fn parent_of(&self, child: &str) -> Option<&XmlNode> {
        if self.child(child).is_some() {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.parent_of(child))
    }
}

fn attach(stack: &mut [XmlNode], node: XmlNode) -> Result<(), String> {
    stack
        .last_mut()
        .map(|parent| parent.children.push(node))
        .ok_or_else(|| "XML non bilanciato".to_string())
}

fn local_name(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw).into_owned()
}
