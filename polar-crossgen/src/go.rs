use handlebars::{handlebars_helper, Handlebars};
use handlebars::{Context, Helper, Output, RenderContext, RenderError};
use heck::{CamelCase, SnakeCase};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_reflection::{ContainerFormat, Format, Named, Registry, VariantFormat};

pub struct Codegen<'a> {
    handlebars: Handlebars<'a>,
    output: String,
}

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
            NewTypeStruct(format) => {
                self.render(
                    "newtype",
                    &json!({
                        "name": name,
                        "type": self.quote_type(format),
                    }),
                )?;
            }
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
