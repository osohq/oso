use indoc::formatdoc;

use polar_core::{
    kb::KnowledgeBase,
    parser::Line,
    terms::{Operator, ToPolarString, Value},
    visitor::Visitor,
};

pub type UnusedRule = (String, usize, usize);

pub fn find_missing_rules(kb: &KnowledgeBase, src: &str) -> Vec<UnusedRule> {
    let parse_result = polar_core::parser::parse_lines(0, src);

    let mut visitor = UnusedRuleVisitor {
        kb,
        missing_rules: vec![],
    };

    if let Ok(lines) = parse_result {
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

    visitor.missing_rules
}

struct UnusedRuleVisitor<'kb> {
    missing_rules: Vec<UnusedRule>,
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
                        let message = formatdoc!(
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
                        self.missing_rules.push((message, left, right));
                    }
                } else {
                    let (left, right) = t.span().unwrap_or((0, 0));
                    let message = format!("There are no rules with the name \"{}\"", c.name);
                    self.missing_rules.push((message, left, right));
                }
            }
            _ => {}
        }
        polar_core::visitor::walk_term(self, t)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Polar;

    #[test]
    fn finds_missing_rules() {
        let p = Polar::wasm_new();
        p.load(
            r#"
            f(1);
            f(2);
            g(_x);
        "#,
            "test.policy",
        )
        .unwrap();

        let target_policy = r#"
        h(x) if f(x); # should be fine
        h(x) if g(x); # should be fine
        h(_x) if f(3); # missing
    "#;

        let missing_rules = p.with_kb(|kb| find_missing_rules(kb, target_policy));

        assert_eq!(missing_rules.len(), 1);
        let missing_rule = &target_policy[missing_rules[0].1..missing_rules[0].2];
        assert_eq!(missing_rule, "f(3)");
    }
}
