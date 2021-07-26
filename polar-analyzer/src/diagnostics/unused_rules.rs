use polar_core::{
    formatting::ToPolarString,
    kb::KnowledgeBase,
    parser::Line,
    terms::{Operator, Value},
    visitor::Visitor,
};

pub type UnusedRule = (String, usize, usize);

pub fn find_unused_rules(kb: &KnowledgeBase, src: &str) -> Vec<UnusedRule> {
    let parse_result = polar_core::parser::parse_file_with_errors(0, src);

    let mut visitor = UnusedRuleVisitor {
        kb: &kb,
        unused_rules: vec![],
    };

    if let Ok((lines, _)) = parse_result {
        for line in lines {
            match line {
                Line::Rule(r) => {
                    visitor.visit_term(&r.body);
                }
                Line::Query(q) => {
                    visitor.visit_term(&q);
                }
            }
        }
    }

    visitor.unused_rules
}

struct UnusedRuleVisitor<'kb> {
    unused_rules: Vec<UnusedRule>,
    kb: &'kb KnowledgeBase,
}

impl<'kb> Visitor for UnusedRuleVisitor<'kb> {
    fn visit_term(&mut self, t: &polar_core::terms::Term) {
        match t.value() {
            Value::Expression(op) if op.operator == Operator::Dot => {
                // do nothing, we cannot have any rules inside a dot
                return;
            }
            Value::Call(c) => {
                if let Some(rules) = self.kb.rules.get(&c.name) {
                    if rules.get_applicable_rules(&c.args).is_empty() {
                        let (left, right) = t.span().unwrap_or((0, 0));
                        let message = format!(
                            r#"There are no rules matching the format:
  {}
Found:
  {}
"#,
                            c.to_polar(),
                            rules
                                .rules
                                .iter()
                                .map(|(_, r)| r.to_polar())
                                .collect::<Vec<String>>()
                                .join("\n  ")
                        );
                        self.unused_rules.push((message, left, right));
                    }
                } else {
                    let (left, right) = t.span().unwrap_or((0, 0));
                    let message = format!("There are no rules with the name \"{}\"", c.name);
                    self.unused_rules.push((message, left, right));
                }
            }
            _ => {}
        }
        polar_core::visitor::walk_term(self, t)
    }
}
