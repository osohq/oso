use polar_core::{
    parser::Line,
    rules::Rule,
    terms::{Term, ToPolarString},
};

pub struct PrettyPrintContext {
    source: String,
}

impl PrettyPrintContext {
    pub fn new(source: String) -> Self {
        Self { source }
    }
}

pub trait PrettyPrint {
    fn to_pretty_string(&self, _context: &PrettyPrintContext) -> String;
}

impl PrettyPrint for Term {
    fn to_pretty_string(&self, _context: &PrettyPrintContext) -> String {
        self.to_polar()
    }
}

impl PrettyPrint for Rule {
    fn to_pretty_string(&self, context: &PrettyPrintContext) -> String {
        let no_body = self.clone_with_no_body();
        let rule_str = no_body.to_polar();
        let mut rule_chars = rule_str.chars();
        rule_chars.next_back();
        let mut rule_str: String = rule_chars.collect();
        rule_str.push_str(" if\n  ");
        rule_str.push_str(&self.body.to_pretty_string(context));
        rule_str.push(';');
        rule_str
    }
}

impl PrettyPrint for Line {
    fn to_pretty_string(&self, context: &PrettyPrintContext) -> String {
        match self {
            Line::Rule(rule) => rule.to_pretty_string(context),
            Line::ResourceBlock { .. } => "RESOURCE BLOCK".to_string(),
            _ => "UNKNOWN LINE".to_string(),
        }
    }
}
