use std::{borrow::Borrow, rc::Rc};

use crate::ast::*;
use polar_core::{
    formatting::source_lines,
    sources::Source,
    terms::{Operator, ToPolarString},
};
use pretty::{Doc, RcDoc};

pub struct PrettyContext {
    source: String,
    position: usize,
}

fn comments_in_content(content: String) -> Vec<String> {
    return content
        .split("\n")
        .filter(|s| s.contains('#'))
        .map(|s| s.trim().to_string())
        .collect();
}

impl PrettyContext {
    pub fn new(source: String) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    pub fn print_trailing_comments(&self) -> String {
        if self.position < self.source.len() {
            let trailing_content: String = self.source.chars().skip(self.position).collect();
            comments_in_content(trailing_content).join("\n")
        } else {
            "".to_string()
        }
    }
}

// // Tree structure for pretty printing
// pub enum Doc {
//     // Text, followed by a doc
//     Text(String, Box<Doc>),
//     // Line break with a number of indents, followed by a doc
//     Line(u32, Box<Doc>),
//     // Blank
//     Nil,
//     // Multiple options to render the same doc
//     Union(Box<Doc>, Box<Doc>),
// }

// pub trait PrettyPrint {
//     fn to_pretty_doc(&self, _context: &PrettyPrintContext) -> Doc;
//     // fn to_pretty_string(&self, _context: &PrettyPrintContext) -> String;
// }

// impl PrettyPrint for Term {
//     fn to_pretty_doc(&self, context: &PrettyPrintContext) -> Doc {
//         self.value().to_pretty_doc(context)
//     }
// }

// impl PrettyPrint for Value {
//     fn to_pretty_doc(&self) -> String {
//         match self {
//             Value::Dictionary(i) => i.to_polar(),
//             Value::Pattern(i) => i.to_polar(),
//             Value::Call(c) => c.to_polar(),
//             Value::List(l) => format!("[{}]", format_args!(Operator::And, l, ", "),),
//             Value::Variable(s) => s.to_polar(),
//             Value::Expression(e) => e.to_polar(),
//             val => Doc::Text(val.to_polar(), Box::new(Doc::Nil)),
//         }
//     }
// }

// impl PrettyPrint for ShorthandRule {
//     fn to_polar(&self) -> String {
//         let Self {
//             head,
//             body: (implier, relation),
//         } = self;
//         if let Some((keyword, relation)) = relation {
//             format!(
//                 "{} if {} {} {};",
//                 head.to_polar(),
//                 implier.to_polar(),
//                 keyword.to_polar(),
//                 relation.to_polar()
//             )
//         } else {
//             format!("{} if {};", head.to_polar(), implier.to_polar())
//         }
//     }
// }

// impl PrettyPrint for BlockType {
//     fn to_polar(&self) -> String {
//         match self {
//             Self::Actor => "actor".to_owned(),
//             Self::Resource => "resource".to_owned(),
//         }
//     }
// }

// impl PrettyPrint for ResourceBlock {
//     fn to_polar(&self) -> String {
//         let mut s = format!(
//             "{} {} {{\n",
//             self.block_type.to_polar(),
//             self.resource.to_polar()
//         );
//         if let Some(ref roles) = self.roles {
//             s += &format!("  roles = {};\n", roles.to_polar());
//         }
//         if let Some(ref permissions) = self.permissions {
//             s += &format!("  permissions = {};\n", permissions.to_polar());
//         }
//         if let Some(ref relations) = self.relations {
//             s += &format!("  relations = {};\n", relations.to_polar());
//         }
//         for rule in &self.shorthand_rules {
//             s += &format!("  {}\n", rule.to_polar());
//         }
//         s += "}";
//         s
//     }
// }

// impl PrettyPrint for Rule {
//     fn to_pretty_doc(&self, context: &PrettyPrintContext) -> Doc {
//         let no_body = self.clone_with_no_body();
//         let rule_str = no_body.to_polar();
//         let mut rule_chars = rule_str.chars();
//         rule_chars.next_back();
//         let mut rule_str: String = rule_chars.collect();
//         rule_str.push_str(" if");
//         Doc::Text(rule_str, Doc::Nest(2, &self.body.to_pretty_doc(context)));
//     }
// }

// impl PrettyPrint for Line {
//     fn to_pretty_string(&self, context: &PrettyPrintContext) -> String {
//         match self {
//             Line::Rule(rule) => rule.to_pretty_string(context),
//             Line::ResourceBlock { .. } => "RESOURCE BLOCK".to_string(),
//             _ => "UNKNOWN LINE".to_string(),
//         }
//     }
// }

pub trait ToDoc {
    fn to_doc(&self, context: &mut PrettyContext) -> RcDoc<()>;
}

// impl ToDoc for Line {
//     fn to_doc(&self, context: &mut PrettyContext) -> RcDoc<()> {
//         match self {
//             Line::Rule(rule) => rule.to_doc(),
//             Line::ResourceBlock { .. } => RcDoc::as_string("RESOURCE BLOCK".to_string())
//                 .append(RcDoc::hardline())
//                 .append(RcDoc::hardline()),
//             _ => RcDoc::as_string("UNKNOWN LINE".to_string())
//                 .append(RcDoc::hardline())
//                 .append(RcDoc::hardline()),
//         }
//     }
// }

impl ToDoc for Node {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        // TODO: find and print comments between context.position and self.start
        if self.start < context.position {
            panic!("Nodes are being processed out of order. Make sure that to_doc(..) is called on Nodes in the order they appear in the source.\n\nError caused by this node:\n{}\nBut processing has already been completed up to here:\n{}",
                source_lines(&Source::new(None, &context.source), self.start, 2),
                source_lines(&Source::new(None, &context.source), context.position, 2),
            );
        }
        let content_since_last: String = context
            .source
            .chars()
            .skip(context.position)
            .take(self.start - context.position)
            .collect();
        let comments = comments_in_content(content_since_last);
        let mut doc: RcDoc<()> = RcDoc::nil();
        if comments.len() > 0 {
            doc = RcDoc::intersperse(
                comments
                    .into_iter()
                    .map(|c| RcDoc::text(c))
                    .collect::<Vec<_>>(),
                RcDoc::hardline(),
            )
            .append(RcDoc::hardline())
        }
        context.position = self.start;
        let result = doc.append(self.value.to_doc(&mut context));
        context.position = self.end;
        result
    }
}

impl ToDoc for Rule {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        let name_doc = self.name.to_doc(&mut context);
        let param_docs: Vec<RcDoc<()>> = self
            .parameters
            .iter()
            .map(|p| p.to_doc(&mut context))
            .collect();
        let rule_head = name_doc.append(
            RcDoc::text("(")
                .append(Doc::line_())
                .append(RcDoc::intersperse(
                    param_docs,
                    RcDoc::text(",").append(Doc::line()),
                ))
                .nest(2)
                .append(Doc::line_())
                .append(RcDoc::text(")"))
                .nest(-2)
                .group(),
        );
        let body_doc = if let Some(body) = &self.body {
            RcDoc::text(" if")
                .append(RcDoc::line())
                .append(body.to_doc(context))
        } else {
            RcDoc::nil()
        };
        rule_head
            .append(body_doc)
            .nest(2)
            .group()
            .append(RcDoc::text(";"))
    }
}

impl ToDoc for Parameter {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        let doc = self.parameter.to_doc(&mut context);
        if let Some(specializer) = &self.specializer {
            doc.append(RcDoc::text(": ").append(specializer.to_doc(&mut context)))
        } else {
            doc
        }
    }
}

impl ToDoc for Field {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        // TODO: more concise way to do this?
        let is_same = if let Node {
            value: Value::Variable(x),
            ..
        } = self.0.borrow()
        {
            if let Node {
                value: Value::Variable(y),
                ..
            } = self.1.borrow()
            {
                x.0 == y.0
            } else {
                false
            }
        } else {
            false
        };
        if is_same {
            self.0.to_doc(&mut context)
        } else {
            self.0
                .to_doc(&mut context)
                .append(RcDoc::text(": "))
                .append(self.1.to_doc(&mut context))
        }
    }
}

impl ToDoc for Fields {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        let docs: Vec<RcDoc<_>> = self
            .iter()
            .map(|field| field.to_doc(&mut context))
            .collect();
        RcDoc::intersperse(docs, RcDoc::text(",").append(RcDoc::line())).group()
    }
}

impl ToDoc for Dictionary {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        let field_doc = self.0.to_doc(&mut context);
        RcDoc::text("{")
            .append(RcDoc::line().append(field_doc).nest(2))
            .append(RcDoc::line())
            .append(RcDoc::text("}"))
            .group()
    }
}

impl ToDoc for InstanceLiteral {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        self.tag.to_doc(&mut context).append(
            RcDoc::text("{")
                .append(
                    RcDoc::line()
                        .append(self.fields.to_doc(&mut context))
                        .nest(2),
                )
                .append(RcDoc::line())
                .append(RcDoc::text("}"))
                .group(),
        )
    }
}

impl ToDoc for Pattern {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        match self {
            Pattern::Dictionary(d) => d.to_doc(&mut context),
            Pattern::Instance(i) => i.to_doc(&mut context),
        }
    }
}

impl ToDoc for Call {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        let name_doc = self.name.to_doc(&mut context);
        let mut args_docs = self
            .args
            .iter()
            .map(|arg| arg.to_doc(&mut context))
            .collect::<Vec<RcDoc<_>>>();
        if let Some(kwargs) = &self.kwargs {
            args_docs = args_docs
                .into_iter()
                .chain(kwargs.iter().map(|field| field.to_doc(&mut context)))
                .collect();
        }
        name_doc.append(
            RcDoc::text("(")
                .append(
                    RcDoc::line_()
                        .append(RcDoc::intersperse(
                            args_docs,
                            RcDoc::text(",").append(Doc::line()),
                        ))
                        .nest(2)
                        .group(),
                )
                .append(Doc::line_())
                .append(RcDoc::text(")"))
                .group(),
        )
    }
}

impl ToDoc for Operation {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        let op_str = self.operator.to_polar();
        let arg_docs: Vec<RcDoc<_>> = self
            .args
            .iter()
            .map(|arg| arg.to_doc(&mut context))
            .collect();
        // TODO: if any of args is an operation AND the precedence of the
        // operation is LOWER than the precedence of this operation, then that
        // arg should be wrapped in parentheses
        // E.g. i * (y + z) => "+" is lower precedence than "*"
        RcDoc::intersperse(
            arg_docs,
            RcDoc::space()
                .append(RcDoc::text(op_str))
                .append(Doc::line()),
        )
        .group()
    }
}

impl ToDoc for File {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        let docs: Vec<RcDoc<_>> = self
            .0
            .iter()
            .map(|node| node.to_doc(&mut context))
            .collect();
        RcDoc::intersperse(docs, RcDoc::hardline().append(RcDoc::hardline()))
    }
}

impl ToDoc for Value {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        match self {
            Value::Number(i) => RcDoc::text(format!("{}", i)),
            Value::String(s) => RcDoc::text(format!("\"{}\"", s)),
            Value::Boolean(b) => {
                if *b {
                    RcDoc::text("true".to_string())
                } else {
                    RcDoc::text("false".to_string())
                }
            }
            Value::Dictionary(d) => d.to_doc(&mut context),
            Value::Pattern(p) => p.to_doc(&mut context),
            Value::Call(c) => c.to_doc(&mut context),
            Value::List(l) => RcDoc::text("TODO: LIST".to_string()),
            Value::Variable(s) => RcDoc::text(s.0.clone()),
            Value::RestVariable(s) => RcDoc::text(format!("*{}", s.0)),
            Value::Expression(e) => e.to_doc(&mut context),
            Value::Symbol(s) => RcDoc::text(s),
            Value::Rule(r) => r.to_doc(&mut context),
            Value::File(file) => file.to_doc(&mut context),
        }
    }
}
