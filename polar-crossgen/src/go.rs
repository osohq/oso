use serde_reflection::Format;
pub struct Go;

impl super::codegen::TypeMapping for Go {
    fn quote_type(&self, format: &Format) -> String {
        use Format::*;
        match format {
            TypeName(x) => x.to_owned(),
            Unit => "struct {}".into(),
            Bool => "bool".into(),
            I8 => "int8".into(),
            I16 => "int16".into(),
            I32 => "int32".into(),
            I64 => "int64".into(),
            I128 => todo!("unsupported"), //"serde.Int128".into(),
            U8 => "uint8".into(),
            U16 => "uint16".into(),
            U32 => "uint32".into(),
            U64 => "uint64".into(),
            U128 => todo!("unsupported"), //"serde.Uint128".into(),
            F32 => "float32".into(),
            F64 => "float64".into(),
            Char => "string".into(), // serde_json serializes char as a string
            Str => "string".into(),
            Bytes => "[]byte".into(),

            Option(format) => format!("*{}", self.quote_type(format)),
            Seq(format) => format!("[]{}", self.quote_type(format)),
            Map { key, value } => {
                format!("map[{}]{}", self.quote_type(key), self.quote_type(value))
            }
            Tuple(formats) => format!(
                "struct {{{}}}",
                formats
                    .iter()
                    .enumerate()
                    .map(|(index, format)| format!("Field{} {}", index, self.quote_type(format)))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            TupleArray { content, size } => format!("[{}]{}", size, self.quote_type(content)),

            Variable(_) => panic!("unexpected value"),
        }
    }
}
