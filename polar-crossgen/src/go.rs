use handlebars::{handlebars_helper, Handlebars};
use handlebars::{Context, Helper, Output, RenderContext, RenderError};
use serde::{Deserialize, Serialize};
use serde_json::json;
// use crate::{
//     common,
//     indent::{IndentConfig, IndentedWriter},
//     CodeGeneratorConfig, Encoding,
// };
use heck::{CamelCase, SnakeCase};
use serde_reflection::{ContainerFormat, Format, Named, Registry, VariantFormat};

pub struct Codegen<'a> {
    handlebars: Handlebars<'a>,
    output: String,
}

// #[derive(Clone, Deserialize, Serialize)]
// struct TypeInfo {
//     variable: String,
//     name: String,
//     deserialize_name: String,
//     deserialize_type_name: String,
//     nested: bool,
//     inner: Option<Box<TypeInfo>>,
// }

// default format helper
fn format_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> anyhow::Result<(), RenderError> {
    // get parameter from helper or throw an error
    let param = h
        .param(0)
        .ok_or(RenderError::new("Param 0 is required for format helper."))?;
    let rendered = param.value().to_string();
    out.write(rendered.as_ref())?;
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TypeInfo {
    variable: String,
    type_name: String,
}

impl<'a> Codegen<'a> {
    pub fn output(registry: &Registry) -> anyhow::Result<String> {
        handlebars_helper!(snake_case: |s: str| s.to_snake_case());
        handlebars_helper!(camel_case: |s: str| s.to_camel_case());

        let mut handlebars = Handlebars::new();
        handlebars.register_templates_directory(".hbs", "templates/go")?;
        handlebars.register_helper("format", Box::new(format_helper));
        handlebars.register_helper("snake-case", Box::new(snake_case));
        handlebars.register_helper("camel-case", Box::new(camel_case));
        handlebars.set_strict_mode(true);
        let output = r#"package oso

import (
    "encoding/json"
    "errors"
    "fmt"
)


"#
        .to_string();
        let mut gen = Self { handlebars, output };

        for (name, format) in registry {
            gen.output_container(name, format)?;
        }

        Ok(gen.output.clone())
    }

    fn render<S: serde::Serialize>(&mut self, template: &str, input: &S) -> anyhow::Result<()> {
        self.output += &self.handlebars.render(template, input)?;
        Ok(())
    }

    fn type_info(&self, field: &Named<Format>) -> TypeInfo {
        TypeInfo {
            // TODO: This wont be camel case for non-Go language
            variable: field.name.to_camel_case(),
            type_name: self.quote_type(&field.value),
        }
    }

    fn output_struct(&mut self, name: &str, fields: &[Named<Format>]) -> anyhow::Result<()> {
        self.render(
            "struct",
            &json!({"name": name, "fields": fields
                .iter()
                .map(|f| self.type_info(&f))
                .collect::<Vec<TypeInfo>>()
            }),
        )
    }

    fn output_container(&mut self, name: &str, format: &ContainerFormat) -> anyhow::Result<()> {
        use ContainerFormat::*;
        match format {
            UnitStruct => self.render("unit_struct", &"")?,
            // NewTypeStruct(format) => {
            //     match format.as_ref() {
            //         // See comment in `output_variant`.
            //     Format::TypeName(_) | Format::Option(_) => vec![Named {
            //         name: "Value".to_string(),
            //         value: format.as_ref().clone(),
            //     }],
            //     _ => {
            //         self.output_struct_or_variant_new_type_container(None, None, name, format)?;
            //         return Ok(());
            //     }
            //     }
            // },
            // TupleStruct(formats) => formats
            //     .iter()
            //     .enumerate()
            //     .map(|(i, f)| Named {
            //         name: format!("Field{}", i),
            //         value: f.clone(),
            //     })
            //     .collect(),
            Struct(fields) => self.output_struct(name, fields)?,
            Enum(variants) => {
                self.render("enum", &json!({
                    "name": name,
                    "variants": variants.iter().map(|(_, var)| var.name.clone() ).collect::<Vec<String>>()
                }))?;
                for (_, variant) in variants.iter() {
                    self.output_variant(name, &variant.name, &variant.value)?;
                }
            }
            _ => todo!("{:?}", format),
        };
        Ok(())
        // todo!()
        // self.output_struct_or_variant_container(None, None, name, &fields)
    }

    fn output_variant(
        &mut self,
        base: &str,
        name: &str,
        variant: &VariantFormat,
    ) -> anyhow::Result<()> {
        use VariantFormat::*;
        match variant {
            Unit => self.render(
                "unit_variant",
                &json!({
                    "base": base,
                    "name": name,
                }),
            )?,
            NewType(format) => {
                self.render(
                    "newtype",
                    &json!({
                        "name": base.to_owned() + &name,
                        "type": self.quote_type(format),
                    }),
                )?;
            }
            // Tuple(formats) => formats
            //     .iter()
            //     .enumerate()
            //     .map(|(i, f)| Named {
            //         name: format!("Field{}", i),
            //         value: f.clone(),
            //     })
            //     .collect(),
            Struct(fields) => {
                self.output_struct(&(base.to_string() + name), fields)?;
            }
            // Variable(_) => panic!("incorrect value"),
            _ => todo!("{:#?}", variant),
        };
        self.render(
            "enum_variant",
            &json!({
                "base": base,
                "name": name,
                // "variant": self.quote_type(format)
            }),
        )?;
        Ok(())
    }

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
            Char => "rune".into(),
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

// struct GoEmitter<'a, T> {
//     out: &'a mut T,
//     /// Current namespace (e.g. vec!["com", "my_org", "my_package", "MyClass"])
//     current_namespace: Vec<String>,
// }

// impl<'a, T> GoEmitter<'a, T>
// where
//     T: Write,
// {
//     fn output_preamble(&mut self, registry: &Registry) -> Result<()> {
//         writeln!(self.out, "package oso\n\n",)?;
//         // Go does not support disabling warnings on unused imports.
//         if registry.is_empty() {
//             return Ok(());
//         }
//         writeln!(
//             self.out,
//             r#"
// import (
// 	"encoding/json"
// 	"fmt"
// )
// "#
//         )?;
//         writeln!(self.out, ")\n")?;
//         Ok(())
//     }

//     fn has_int128(registry: &Registry) -> bool {
//         for format in registry.values() {
//             if format
//                 .visit(&mut |f| match f {
//                     Format::I128 | Format::U128 => {
//                         // Interrupt the visit if we find a (u)int128
//                         Err(serde_reflection::Error::Custom(String::new()))
//                     }
//                     _ => Ok(()),
//                 })
//                 .is_err()
//             {
//                 return true;
//             }
//         }
//         false
//     }

//     fn has_enum(registry: &Registry) -> bool {
//         for format in registry.values() {
//             if let ContainerFormat::Enum(_) = format {
//                 return true;
//             }
//         }
//         false
//     }

//     fn quote_type(&self, format: &Format) -> String {
//         use Format::*;
//         match format {
//             TypeName(x) => x.to_owned(),
//             Unit => "struct {}".into(),
//             Bool => "bool".into(),
//             I8 => "int8".into(),
//             I16 => "int16".into(),
//             I32 => "int32".into(),
//             I64 => "int64".into(),
//             I128 => "serde.Int128".into(),
//             U8 => "uint8".into(),
//             U16 => "uint16".into(),
//             U32 => "uint32".into(),
//             U64 => "uint64".into(),
//             U128 => "serde.Uint128".into(),
//             F32 => "float32".into(),
//             F64 => "float64".into(),
//             Char => "rune".into(),
//             Str => "string".into(),
//             Bytes => "[]byte".into(),

//             Option(format) => format!("*{}", self.quote_type(format)),
//             Seq(format) => format!("[]{}", self.quote_type(format)),
//             Map { key, value } => {
//                 format!("map[{}]{}", self.quote_type(key), self.quote_type(value))
//             }
//             Tuple(formats) => format!(
//                 "struct {{{}}}",
//                 formats
//                     .iter()
//                     .enumerate()
//                     .map(|(index, format)| format!("Field{} {}", index, self.quote_type(format)))
//                     .collect::<Vec<_>>()
//                     .join("; ")
//             ),
//             TupleArray { content, size } => format!("[{}]{}", size, self.quote_type(content)),

//             Variable(_) => panic!("unexpected value"),
//         }
//     }

//     fn needs_helper(format: &Format) -> bool {
//         use Format::*;
//         matches!(format, Option(_) | Seq(_) | Map { .. } | Tuple(_) | TupleArray { .. })
//     }

//     //     fn output_serialization_helper(&mut self, name: &str, format0: &Format) -> Result<()> {
//     //         use Format::*;

//     //         write!(
//     //             self.out,
//     //             "func serialize_{}(value {}, serializer serde.Serializer) error {{",
//     //             name,
//     //             self.quote_type(format0)
//     //         )?;

//     //         match format0 {
//     //             Option(format) => {
//     //                 write!(
//     //                     self.out,
//     //                     r#"
//     // if value != nil {{
//     // 	if err := serializer.SerializeOptionTag(true); err != nil {{ return err }}
//     // 	{}
//     // }} else {{
//     // 	if err := serializer.SerializeOptionTag(false); err != nil {{ return err }}
//     // }}
//     // "#,
//     //                     self.quote_serialize_value("(*value)", format)
//     //                 )?;
//     //             }

//     //             Seq(format) => {
//     //                 write!(
//     //                     self.out,
//     //                     r#"
//     // if err := serializer.SerializeLen(uint64(len(value))); err != nil {{ return err }}
//     // for _, item := range(value) {{
//     // 	{}
//     // }}
//     // "#,
//     //                     self.quote_serialize_value("item", format)
//     //                 )?;
//     //             }

//     //             Map { key, value } => {
//     //                 write!(
//     //                     self.out,
//     //                     r#"
//     // if err := serializer.SerializeLen(uint64(len(value))); err != nil {{ return err }}
//     // offsets := make([]uint64, len(value))
//     // count := 0
//     // for k, v := range(value) {{
//     // 	offsets[count] = serializer.GetBufferOffset()
//     // 	count += 1
//     // 	{}
//     // 	{}
//     // }}
//     // serializer.SortMapEntries(offsets);
//     // "#,
//     //                     self.quote_serialize_value("k", key),
//     //                     self.quote_serialize_value("v", value)
//     //                 )?;
//     //             }

//     //             Tuple(formats) => {
//     //                 writeln!(self.out)?;
//     //                 for (index, format) in formats.iter().enumerate() {
//     //                     let expr = format!("value.Field{}", index);
//     //                     writeln!(self.out, "{}", self.quote_serialize_value(&expr, format))?;
//     //                 }
//     //             }

//     //             TupleArray { content, size: _ } => {
//     //                 write!(
//     //                     self.out,
//     //                     r#"
//     // for _, item := range(value) {{
//     // 	{}
//     // }}
//     // "#,
//     //                     self.quote_serialize_value("item", content),
//     //                 )?;
//     //             }

//     //             _ => panic!("unexpected case"),
//     //         }
//     //         writeln!(self.out, "return nil")?;
//     //         self.out.unindent();
//     //         writeln!(self.out, "}}\n")
//     //     }

//     //     fn output_deserialization_helper(&mut self, name: &str, format0: &Format) -> Result<()> {
//     //         use Format::*;

//     //         write!(
//     //             self.out,
//     //             "func deserialize_{}(deserializer serde.Deserializer) ({}, error) {{",
//     //             name,
//     //             self.quote_type(format0),
//     //         )?;

//     //         match format0 {
//     //             Option(format) => {
//     //                 write!(
//     //                     self.out,
//     //                     r#"
//     // tag, err := deserializer.DeserializeOptionTag()
//     // if err != nil {{ return nil, err }}
//     // if tag {{
//     // 	value := new({})
//     // 	{}
//     //         return value, nil
//     // }} else {{
//     // 	return nil, nil
//     // }}
//     // "#,
//     //                     self.quote_type(format),
//     //                     self.quote_deserialize(format, "*value", "nil"),
//     //                 )?;
//     //             }

//     //             Seq(format) => {
//     //                 write!(
//     //                     self.out,
//     //                     r#"
//     // length, err := deserializer.DeserializeLen()
//     // if err != nil {{ return nil, err }}
//     // obj := make([]{}, length)
//     // for i := range(obj) {{
//     // 	{}
//     // }}
//     // return obj, nil
//     // "#,
//     //                     self.quote_type(format),
//     //                     self.quote_deserialize(format, "obj[i]", "nil")
//     //                 )?;
//     //             }

//     //             Map { key, value } => {
//     //                 write!(
//     //                     self.out,
//     //                     r#"
//     // length, err := deserializer.DeserializeLen()
//     // if err != nil {{ return nil, err }}
//     // obj := make(map[{0}]{1})
//     // previous_slice := serde.Slice {{ 0, 0 }}
//     // for i := 0; i < int(length); i++ {{
//     // 	var slice serde.Slice
//     // 	slice.Start = deserializer.GetBufferOffset()
//     // 	var key {0}
//     // 	{2}
//     // 	slice.End = deserializer.GetBufferOffset()
//     // 	if i > 0 {{
//     // 		err := deserializer.CheckThatKeySlicesAreIncreasing(previous_slice, slice)
//     // 		if err != nil {{ return nil, err }}
//     // 	}}
//     // 	previous_slice = slice
//     // 	{3}
//     // }}
//     // return obj, nil
//     // "#,
//     //                     self.quote_type(key),
//     //                     self.quote_type(value),
//     //                     self.quote_deserialize(key, "key", "nil"),
//     //                     self.quote_deserialize(value, "obj[key]", "nil"),
//     //                 )?;
//     //             }

//     //             Tuple(formats) => {
//     //                 write!(
//     //                     self.out,
//     //                     r#"
//     // var obj {}
//     // {}
//     // return obj, nil
//     // "#,
//     //                     self.quote_type(format0),
//     //                     formats
//     //                         .iter()
//     //                         .enumerate()
//     //                         .map(|(i, f)| self.quote_deserialize(f, &format!("obj.Field{}", i), "obj"))
//     //                         .collect::<Vec<_>>()
//     //                         .join("\n")
//     //                 )?;
//     //             }

//     //             TupleArray { content, size } => {
//     //                 write!(
//     //                     self.out,
//     //                     r#"
//     // var obj [{1}]{0}
//     // for i := range(obj) {{
//     // 	{2}
//     // }}
//     // return obj, nil
//     // "#,
//     //                     self.quote_type(content),
//     //                     size,
//     //                     self.quote_deserialize(content, "obj[i]", "obj")
//     //                 )?;
//     //             }

//     //             _ => panic!("unexpected case"),
//     //         }
//     //         self.out.unindent();
//     //         writeln!(self.out, "}}\n")
//     //     }

//     fn output_variant(
//         &mut self,
//         base: &str,
//         index: u32,
//         name: &str,
//         variant: &VariantFormat,
//     ) -> Result<()> {
//         use VariantFormat::*;
//         let fields = match variant {
//             Unit => Vec::new(),
//             NewType(format) => match format.as_ref() {
//                 // We cannot define a "new type" (e.g. `type Foo Bar`) out of a typename `Bar` because `Bar`
//                 // could point to a Go interface. This would make `Foo` an interface as well. Interfaces can't be used
//                 // as structs (e.g. they cannot have methods).
//                 //
//                 // Similarly, option types are compiled as pointers but `type Foo *Bar` would prevent `Foo` from being a
//                 // valid pointer receiver.
//                 Format::TypeName(_) | Format::Option(_) => vec![Named {
//                     name: "Value".to_string(),
//                     value: format.as_ref().clone(),
//                 }],
//                 // Other cases are fine.
//                 _ => {
//                     self.output_struct_or_variant_new_type_container(
//                         Some(base),
//                         Some(index),
//                         name,
//                         format,
//                     )?;
//                     return Ok(());
//                 }
//             },
//             Tuple(formats) => formats
//                 .iter()
//                 .enumerate()
//                 .map(|(i, f)| Named {
//                     name: format!("Field{}", i),
//                     value: f.clone(),
//                 })
//                 .collect(),
//             Struct(fields) => fields
//                 .iter()
//                 .map(|f| Named {
//                     name: f.name.to_camel_case(),
//                     value: f.value.clone(),
//                 })
//                 .collect(),
//             Variable(_) => panic!("incorrect value"),
//         };
//         self.output_struct_or_variant_container(Some(base), Some(index), name, &fields)
//     }

//     fn output_struct_or_variant_container(
//         &mut self,
//         variant_base: Option<&str>,
//         variant_index: Option<u32>,
//         name: &str,
//         fields: &[Named<Format>],
//     ) -> Result<()> {
//         let full_name = match variant_base {
//             None => name.to_string(),
//             Some(base) => format!("{}__{}", base, name),
//         };
//         // Struct
//         writeln!(self.out)?;

//         writeln!(
//             self.out,
//             r#"
// // {name}
// type {name} struct {{
//     {fields}
// }}
//         "#,
//             name = name,
//             fields = fields
//                 .iter()
//                 .map(|field| {
//                     format!(
//                         r#"
//     // {field_name}
//     {field_name} {field_type},"#,
//                         field_name = field.name,
//                         field_type = self.quote_type(&field.value)
//                     )
//                 })
//                 .collect::<Vec<String>>()
//                 .join("\n")
//         );

//         // Link to base interface.
//         if let Some(base) = variant_base {
//             writeln!(self.out, "\nfunc (*{}) is{}() {{}}", full_name, base)?;
//         }

//         // Serialize
//         // if self.generator.config.serialization {
//         //     writeln!(
//         //         self.out,
//         //         "\nfunc (obj *{}) Serialize(serializer serde.Serializer) error {{",
//         //         full_name
//         //     )?;

//         //     writeln!(
//         //         self.out,
//         //         "if err := serializer.IncreaseContainerDepth(); err != nil {{ return err }}"
//         //     )?;
//         //     if let Some(index) = variant_index {
//         //         writeln!(self.out, "serializer.SerializeVariantIndex({})", index)?;
//         //     }
//         //     for field in fields {
//         //         writeln!(
//         //             self.out,
//         //             "{}",
//         //             self.quote_serialize_value(&format!("obj.{}", &field.name), &field.value)
//         //         )?;
//         //     }
//         //     writeln!(self.out, "serializer.DecreaseContainerDepth()")?;
//         //     writeln!(self.out, "return nil")?;
//         //     self.out.unindent();
//         //     writeln!(self.out, "}}")?;

//         //     for encoding in &self.generator.config.encodings {
//         //         self.output_struct_serialize_for_encoding(&full_name, *encoding)?;
//         //     }
//         // }
//         // // Deserialize (struct) or Load (variant)
//         // if self.generator.config.serialization {
//         //     writeln!(
//         //         self.out,
//         //         "\nfunc {0}{1}(deserializer serde.Deserializer) ({1}, error) {{",
//         //         if variant_base.is_none() {
//         //             "Deserialize"
//         //         } else {
//         //             "load_"
//         //         },
//         //         full_name,
//         //     )?;

//         //     writeln!(self.out, "var obj {}", full_name)?;
//         //     writeln!(
//         //         self.out,
//         //         "if err := deserializer.IncreaseContainerDepth(); err != nil {{ return obj, err }}"
//         //     )?;
//         //     for field in fields {
//         //         writeln!(
//         //             self.out,
//         //             "{}",
//         //             self.quote_deserialize(&field.value, &format!("obj.{}", field.name), "obj")
//         //         )?;
//         //     }
//         //     writeln!(self.out, "deserializer.DecreaseContainerDepth()")?;
//         //     writeln!(self.out, "return obj, nil")?;
//         //     self.out.unindent();
//         //     writeln!(self.out, "}}")?;

//         //     if variant_base.is_none() {
//         //         for encoding in &self.generator.config.encodings {
//         //             self.output_struct_deserialize_for_encoding(&full_name, *encoding)?;
//         //         }
//         //     }
//         // }
//         Ok(())
//     }

//     // Same as output_struct_or_variant_container but we map the container with a single anonymous field
//     // to a new type in Go.
//     fn output_struct_or_variant_new_type_container(
//         &mut self,
//         variant_base: Option<&str>,
//         variant_index: Option<u32>,
//         name: &str,
//         format: &Format,
//     ) -> Result<()> {
//         let full_name = match variant_base {
//             None => name.to_string(),
//             Some(base) => format!("{}__{}", base, name),
//         };
//         // Struct
//         writeln!(self.out)?;
//         self.output_comment(name)?;
//         writeln!(self.out, "type {} {}", full_name, self.quote_type(format))?;

//         // Link to base interface.
//         if let Some(base) = variant_base {
//             writeln!(self.out, "\nfunc (*{}) is{}() {{}}", full_name, base)?;
//         }

//         // Serialize
//         if self.generator.config.serialization {
//             writeln!(
//                 self.out,
//                 "\nfunc (obj *{}) Serialize(serializer serde.Serializer) error {{",
//                 full_name
//             )?;

//             writeln!(
//                 self.out,
//                 "if err := serializer.IncreaseContainerDepth(); err != nil {{ return err }}"
//             )?;
//             if let Some(index) = variant_index {
//                 writeln!(self.out, "serializer.SerializeVariantIndex({})", index)?;
//             }
//             writeln!(
//                 self.out,
//                 "{}",
//                 self.quote_serialize_value(
//                     &format!("(({})(*obj))", self.quote_type(format)),
//                     format
//                 )
//             )?;
//             writeln!(self.out, "serializer.DecreaseContainerDepth()")?;
//             writeln!(self.out, "return nil")?;
//             self.out.unindent();
//             writeln!(self.out, "}}")?;

//             for encoding in &self.generator.config.encodings {
//                 self.output_struct_serialize_for_encoding(&full_name, *encoding)?;
//             }
//         }
//         // Deserialize (struct) or Load (variant)
//         if self.generator.config.serialization {
//             writeln!(
//                 self.out,
//                 "\nfunc {0}{1}(deserializer serde.Deserializer) ({1}, error) {{",
//                 if variant_base.is_none() {
//                     "Deserialize"
//                 } else {
//                     "load_"
//                 },
//                 full_name,
//             )?;

//             writeln!(self.out, "var obj {}", self.quote_type(format))?;
//             writeln!(self.out, "if err := deserializer.IncreaseContainerDepth(); err != nil {{ return ({})(obj), err }}", full_name)?;
//             writeln!(
//                 self.out,
//                 "{}",
//                 self.quote_deserialize(format, "obj", &format!("(({})(obj))", full_name))
//             )?;
//             writeln!(self.out, "deserializer.DecreaseContainerDepth()")?;
//             writeln!(self.out, "return ({})(obj), nil", full_name)?;
//             self.out.unindent();
//             writeln!(self.out, "}}")?;

//             if variant_base.is_none() {
//                 for encoding in &self.generator.config.encodings {
//                     self.output_struct_deserialize_for_encoding(&full_name, *encoding)?;
//                 }
//             }
//         }
//         // Custom code
//         self.output_custom_code(name)?;
//         Ok(())
//     }

//     fn output_enum_container(
//         &mut self,
//         name: &str,
//         variants: &BTreeMap<u32, Named<VariantFormat>>,
//     ) -> Result<()> {
//         writeln!(self.out)?;
//         self.output_comment(name)?;
//         writeln!(self.out, "type {} interface {{", name)?;
//         self.current_namespace.push(name.to_string());

//         writeln!(self.out, "is{}()", name)?;
//         if self.generator.config.serialization {
//             writeln!(self.out, "Serialize(serializer serde.Serializer) error")?;
//             for encoding in &self.generator.config.encodings {
//                 writeln!(
//                     self.out,
//                     "{}Serialize() ([]byte, error)",
//                     encoding.name().to_camel_case()
//                 )?;
//             }
//         }
//         self.out.unindent();
//         writeln!(self.out, "}}")?;

//         if self.generator.config.serialization {
//             write!(
//                 self.out,
//                 "\nfunc Deserialize{0}(deserializer serde.Deserializer) ({0}, error) {{",
//                 name
//             )?;

//             writeln!(
//                 self.out,
//                 r#"
// index, err := deserializer.DeserializeVariantIndex()
// if err != nil {{ return nil, err }}

// switch index {{"#,
//             )?;
//             for (index, variant) in variants {
//                 writeln!(
//                     self.out,
//                     r#"case {}:
// 	if val, err := load_{}__{}(deserializer); err == nil {{
// 		return &val, nil
// 	}} else {{
// 		return nil, err
// 	}}
// "#,
//                     index, name, variant.name
//                 )?;
//             }
//             writeln!(
//                 self.out,
//                 "default:
// 	return nil, fmt.Errorf(\"Unknown variant index for {}: %d\", index)",
//                 name,
//             )?;
//             writeln!(self.out, "}}")?;
//             self.out.unindent();
//             writeln!(self.out, "}}")?;

//             for encoding in &self.generator.config.encodings {
//                 self.output_struct_deserialize_for_encoding(name, *encoding)?;
//             }
//         }

//         for (index, variant) in variants {
//             self.output_variant(name, *index, &variant.name, &variant.value)?;
//         }
//         self.current_namespace.pop();
//         // Custom code
//         self.output_custom_code(name)?;
//         Ok(())
//     }

//     fn output_container(&mut self, name: &str, format: &ContainerFormat) -> Result<()> {
//         use ContainerFormat::*;
//         let fields = match format {
//             UnitStruct => Vec::new(),
//             NewTypeStruct(format) => match format.as_ref() {
//                 // See comment in `output_variant`.
//                 Format::TypeName(_) | Format::Option(_) => vec![Named {
//                     name: "Value".to_string(),
//                     value: format.as_ref().clone(),
//                 }],
//                 _ => {
//                     self.output_struct_or_variant_new_type_container(None, None, name, format)?;
//                     return Ok(());
//                 }
//             },
//             TupleStruct(formats) => formats
//                 .iter()
//                 .enumerate()
//                 .map(|(i, f)| Named {
//                     name: format!("Field{}", i),
//                     value: f.clone(),
//                 })
//                 .collect(),
//             Struct(fields) => fields
//                 .iter()
//                 .map(|f| Named {
//                     name: f.name.to_camel_case(),
//                     value: f.value.clone(),
//                 })
//                 .collect(),
//             Enum(variants) => {
//                 let variants = variants
//                     .iter()
//                     .map(|(i, f)| {
//                         (
//                             *i,
//                             Named {
//                                 name: f.name.to_camel_case(),
//                                 value: f.value.clone(),
//                             },
//                         )
//                     })
//                     .collect();
//                 self.output_enum_container(name, &variants)?;
//                 return Ok(());
//             }
//         };
//         self.output_struct_or_variant_container(None, None, name, &fields)
//     }
// }

// /// Installer for generated source files in Go.
// pub struct Installer {
//     install_dir: PathBuf,
//     serde_module_path: Option<String>,
// }

// impl Installer {
//     pub fn new(install_dir: PathBuf, serde_module_path: Option<String>) -> Self {
//         Installer {
//             install_dir,
//             serde_module_path,
//         }
//     }

//     fn runtime_installation_message(
//         &self,
//         name: &str,
//     ) -> std::result::Result<(), Box<dyn std::error::Error>> {
//         eprintln!(
//             "Not installing sources for published package {}{}",
//             match &self.serde_module_path {
//                 None => String::new(),
//                 Some(path) => format!("{}/", path),
//             },
//             name
//         );
//         Ok(())
//     }
// }
