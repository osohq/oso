use std::borrow::Borrow;

use crate::ast::*;
use polar_core::{
    formatting::{precedence, source_lines},
    sources::Source,
    terms::{Operator, ToPolarString},
};
use pretty::{Doc, RcDoc};

pub struct PrettyContext {
    source: String,
    position: usize,
}

fn comments_in_content(content: &String) -> Vec<String> {
    return content
        .split("\n")
        .filter(|s| s.contains('#'))
        .map(|s| {
            s.chars()
                .skip_while(|s| *s != '#')
                .collect::<String>()
                .trim()
                .to_string()
        })
        .collect();
}

fn contains_double_line_break(content: &String) -> bool {
    content.split("\n").filter(|s| s.trim().is_empty()).count() > 2
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
            comments_in_content(&trailing_content).join("\n")
        } else {
            "".to_string()
        }
    }
}

pub trait ToDoc {
    fn to_doc(&self, context: &mut PrettyContext) -> RcDoc<()>;
}

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
        let double_line_break = contains_double_line_break(&content_since_last);
        let comments = comments_in_content(&content_since_last);
        let mut doc: RcDoc<()> = if double_line_break {
            RcDoc::hardline()
        } else {
            RcDoc::nil()
        };
        if comments.len() > 0 {
            doc = doc
                .append(RcDoc::intersperse(
                    comments
                        .into_iter()
                        .map(|c| RcDoc::text(c))
                        .collect::<Vec<_>>(),
                    RcDoc::hardline(),
                ))
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
        if args_docs.len() == 0 {
            return name_doc.append(RcDoc::text("()"));
        }
        name_doc.append(
            RcDoc::text("(")
                .append(
                    RcDoc::line_()
                        .append(
                            RcDoc::intersperse(args_docs, RcDoc::text(",").append(Doc::line()))
                                .group(),
                        )
                        .nest(2),
                )
                .append(Doc::line_())
                .append(RcDoc::text(")"))
                .group(),
        )
    }
}

impl ToDoc for List {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        let docs: Vec<RcDoc<_>> = self
            .0
            .iter()
            .map(|node| node.to_doc(&mut context))
            .collect();
        RcDoc::text("[")
            .append(
                RcDoc::line_()
                    .append(RcDoc::intersperse(docs, RcDoc::text(",").append(Doc::line())).group())
                    .nest(2),
            )
            .append(Doc::line_())
            .append(RcDoc::text("]"))
            .group()
    }
}

fn dot_operation_to_doc<'a>(args: &'a Vec<Node>, mut context: &mut PrettyContext) -> RcDoc<'a, ()> {
    let left_doc = to_doc_parens(Operator::Dot, &args[0], &mut context);
    let right_doc = to_doc_parens(Operator::Dot, &args[1], &mut context);
    // TODO: figure out multiline version
    // let multiline_version = left_doc.clone()append(
    //     RcDoc::line_()
    //         .append(RcDoc::text("."))
    //         .append(right_doc.clone())
    //         .nest(2),
    // );
    // multiline_version.union(single_line_version)
    let single_line_version = left_doc.append(RcDoc::text(".")).append(right_doc);
    single_line_version
}

fn to_doc_parens<'a>(
    operator: Operator,
    node: &'a Node,
    mut context: &mut PrettyContext,
) -> RcDoc<'a, ()> {
    match &node.value {
        Value::Expression(op) if (precedence(&op.operator) < precedence(&operator)) => {
            RcDoc::text("(")
                .append(node.to_doc(&mut context))
                .append(RcDoc::text(")"))
        }
        _ => node.to_doc(&mut context),
    }
}

fn join_args<'a>(
    operator: Operator,
    args: &'a Vec<Node>,
    joiner: String,
    mut context: &mut PrettyContext,
) -> RcDoc<'a, ()> {
    RcDoc::intersperse(
        args.iter()
            .map(|arg| to_doc_parens(operator, arg, &mut context)),
        RcDoc::text(joiner).append(RcDoc::line()),
    )
    .group()
}

impl ToDoc for Operation {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        use Operator::*;
        match self.operator {
            Debug => RcDoc::text("debug()"),
            Print => RcDoc::text("print(")
                .append(join_args(Print, &self.args, ",".to_string(), &mut context))
                .append(RcDoc::text(")")),
            Cut => RcDoc::text("cut"),
            ForAll => RcDoc::text("forall(")
                .append(join_args(And, &self.args, ",".to_string(), &mut context))
                .append(RcDoc::text(")")),
            New => {
                if self.args.len() == 1 {
                    RcDoc::text("new ").append(to_doc_parens(New, &self.args[0], &mut context))
                } else {
                    RcDoc::text("new (")
                        .append(join_args(Print, &self.args, ",".to_string(), &mut context))
                        .append(RcDoc::text(")"))
                }
            }
            Dot => dot_operation_to_doc(&self.args, &mut context),
            // Unary operators
            Not => RcDoc::text("not ").append(self.args[0].to_doc(&mut context)),
            // Binary operators
            Mul | Div | Mod | Rem | Add | Sub | Eq | Geq | Leq | Neq | Gt | Lt | Unify | Isa
            | In | Assign => match self.args.len() {
                2 => join_args(
                    self.operator,
                    &self.args,
                    format!(" {}", self.operator.to_polar()),
                    &mut context,
                )
                .nest(2),
                3 => {
                    // format!(
                    //     "{} {} {} = {}",
                    //     to_polar_parens(self.operator, &self.args[0]),
                    //     self.operator.to_polar(),
                    //     to_polar_parens(self.operator, &self.args[1]),
                    //     to_polar_parens(self.operator, &self.args[2]),
                    // )
                    RcDoc::text("TODO: ternary unify??")
                }
                // Invalid
                _ => RcDoc::text("TODO: binary operator with a bunch of args??"),
            },
            // n-ary operators
            And | Or => join_args(self.operator, &self.args, " and".to_string(), &mut context),
        }
    }
}

impl ToDoc for ResourceBlock {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        let keyword_doc = if let Some(keyword) = &self.keyword {
            keyword.to_doc(&mut context).append(RcDoc::space())
        } else {
            RcDoc::nil()
        };
        let resource = keyword_doc
            .append(self.resource.to_doc(&mut context))
            .append(RcDoc::space());

        if self.lines.is_empty() {
            return resource.append(RcDoc::text("{}"));
        }
        resource
            .append(RcDoc::text("{"))
            .append(
                RcDoc::line()
                    .append(RcDoc::intersperse(
                        self.lines
                            .iter()
                            .map(|l| l.to_doc(&mut context))
                            .collect::<Vec<RcDoc<_>>>(),
                        RcDoc::line(),
                    ))
                    .nest(2),
            )
            .append(RcDoc::line())
            .append(RcDoc::text("}"))
    }
}

impl ToDoc for ResourceBlockDeclaration {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        self.0
            .to_doc(&mut context)
            .append(RcDoc::text(" = "))
            .append(self.1.to_doc(&mut context))
    }
}

impl ToDoc for ShorthandRule {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        let head_doc = self.head.to_doc(&mut context);
        let mut body_doc = self.body.0.to_doc(&mut context);
        if let Some((sym, relation)) = &self.body.1 {
            body_doc = body_doc
                .append(RcDoc::space().append(sym.to_doc(&mut context)))
                .append(RcDoc::space())
                .append(relation.to_doc(&mut context));
        }
        head_doc.append(RcDoc::text(" if ")).append(body_doc)
    }
}

impl ToDoc for ResourceBlockLine {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        let inner = match self {
            ResourceBlockLine::Declaration(d) => d.to_doc(&mut context),
            ResourceBlockLine::ShorthandRule(s) => s.to_doc(&mut context),
        };
        inner.append(";")
    }
}

impl ToDoc for File {
    fn to_doc(&self, mut context: &mut PrettyContext) -> RcDoc<()> {
        let docs: Vec<RcDoc<_>> = self
            .0
            .iter()
            .map(|node| node.to_doc(&mut context))
            .collect();
        RcDoc::intersperse(docs, RcDoc::hardline())
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
            Value::List(l) => l.to_doc(&mut context),
            Value::Variable(s) => RcDoc::text(s.0.clone()),
            Value::RestVariable(s) => RcDoc::text(format!("*{}", s.0)),
            Value::Expression(e) => e.to_doc(&mut context),
            Value::Symbol(s) => RcDoc::text(s),
            Value::Rule(r) => r.to_doc(&mut context),
            Value::File(file) => file.to_doc(&mut context),
            Value::ResourceBlock(rb) => rb.to_doc(&mut context),
            Value::ResourceBlockLine(rbl) => rbl.to_doc(&mut context),
        }
    }
}
