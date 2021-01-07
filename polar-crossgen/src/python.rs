use serde_reflection::Format;
pub struct Python;

impl super::codegen::TypeMapping for Python {
    fn quote_type(&self, format: &Format) -> String {
        use Format::*;
        match format {
            TypeName(x) => format!("\"{}\"", x),
            Unit => "()".into(),
            Bool => "bool".into(),
            I8 | I16 | I32 | I64 | I128 | U8 | U16 | U32 | U64 | U128 => "int".into(),
            F32 | F64 => "float".into(),
            Char => "str".into(), // serde_json serializes char as a string
            Str => "str".into(),
            Bytes => "bytes".into(),

            Option(format) => format!("typing.Optional[{}]", self.quote_type(format)),
            Seq(format) => format!("typing.Sequence[{}]", self.quote_type(format)),
            Map { key, value } => {
                format!(
                    "typing.Dict[{}, {}]",
                    self.quote_type(key),
                    self.quote_type(value)
                )
            }
            Tuple(formats) => format!(
                "typing.Tuple[{}]",
                formats
                    .iter()
                    .map(|format| self.quote_type(format))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            TupleArray { content, .. } => {
                format!("typing.Sequence[{}]", self.quote_type(content))
            }

            Variable(_) => panic!("unexpected value"),
        }
    }
}
