//! Codegen for Polar API types

use handlebars::{handlebars_helper, Handlebars};
use handlebars::{Context, Helper, Output, RenderContext, RenderError};
use heck::{ToSnakeCase, ToUpperCamelCase};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_reflection::{ContainerFormat, Format, Named, Registry, VariantFormat};

use std::collections::HashMap;
use std::fmt;

pub struct Codegen<'a> {
    handlebars: Handlebars<'a>,
    output: String,
    typemap: &'static dyn TypeMapping,
    types: HashMap<String, TypeVariant>,
}

pub trait TypeMapping {
    fn quote_type(&self, format: &Format) -> String;
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
        .ok_or_else(|| RenderError::new("Param 0 is required for format helper."))?;
    let rendered = param.value().to_string();
    out.write(rendered.as_ref())?;
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum TypeVariant {
    EnumVariant,
    Enum,
    NewType,
    Struct,
    UnitVariant,
    Unknown,
}

impl fmt::Display for TypeVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let r = match self {
            Self::EnumVariant => "enum_variant",
            Self::Enum => "enum",
            Self::NewType => "newtype",
            Self::Struct => "struct",
            Self::UnitVariant => "unit_variant",
            Self::Unknown => "unknown",
        };
        write!(f, "{}", r)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Field {
    variable: String,
    type_name: String,
    variant: TypeVariant,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EnumVariant {
    name: String,
    variant: VariantFormat,
}

impl<'a> Codegen<'a> {
    pub fn new(lang: &str, typemap: &'static dyn TypeMapping) -> anyhow::Result<Self> {
        handlebars_helper!(snake_case: |s: str| s.to_snake_case());
        handlebars_helper!(camel_case: |s: str| s.to_upper_camel_case());
        handlebars_helper!(isEnum: |s: str| s == TypeVariant::Enum.to_string());
        handlebars_helper!(containsEnum: |fields: Vec<Field>| {
            fields.iter().any(|f| matches!(f.variant, TypeVariant::Enum))
        });

        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(handlebars::no_escape);
        handlebars.register_templates_directory(".hbs", format!("templates/{}", lang))?;
        handlebars.register_helper("format", Box::new(format_helper));
        handlebars.register_helper("snake-case", Box::new(snake_case));
        handlebars.register_helper("camel-case", Box::new(camel_case));
        handlebars.register_helper("isEnum", Box::new(isEnum));
        handlebars.register_helper("containsEnum", Box::new(containsEnum));
        handlebars.set_strict_mode(true);
        let output = handlebars.render("preamble", &())?;
        Ok(Self {
            handlebars,
            output,
            typemap,
            types: Default::default(),
        })
    }
    pub fn output(&mut self, registry: &Registry) -> anyhow::Result<String> {
        for (name, format) in registry {
            self.generate_type_info(name, format)?;
        }
        for (name, format) in registry {
            self.output_container(name, format)?;
        }

        Ok(self.output.clone())
    }

    fn quote_type(&self, format: &Format) -> String {
        self.typemap.quote_type(format)
    }

    fn render<S: serde::Serialize>(&mut self, template: &str, input: &S) -> anyhow::Result<()> {
        self.output += &self.handlebars.render(template, input)?;
        Ok(())
    }

    fn get_field(&self, field: &Named<Format>) -> Field {
        let type_name = self.quote_type(&field.value);
        let variant = self
            .types
            .get(&type_name)
            .cloned()
            .unwrap_or(TypeVariant::Unknown);
        Field {
            // TODO: This wont be camel case for non-Go language
            variable: field.name.to_upper_camel_case(),
            variant,
            type_name,
        }
    }

    fn output_struct(&mut self, name: &str, fields: &[Named<Format>]) -> anyhow::Result<()> {
        self.render(
            "struct",
            &json!({"name": name, "fields": fields
                .iter()
                .map(|f| self.get_field(f))
                .collect::<Vec<Field>>()
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
                for (_, variant) in variants.iter() {
                    self.output_variant(name, &variant.name, &variant.value)?;
                }
                self.render(
                    "enum",
                    &json!({
                        "name": name,
                        "variants": variants.clone().into_iter().map(|(_, var)| {
                            EnumVariant { name: var.name, variant: var.value }
                        }).collect::<Vec<EnumVariant>>()
                    }),
                )?;
            }
            _ => todo!("{:?}", format),
        };
        Ok(())
    }

    fn generate_type_info(&mut self, name: &str, format: &ContainerFormat) -> anyhow::Result<()> {
        use ContainerFormat::*;
        self.types.insert(
            name.to_owned(),
            match format {
                UnitStruct => TypeVariant::UnitVariant,
                NewTypeStruct(_) => TypeVariant::NewType,
                Struct(_) => TypeVariant::Struct,
                Enum(_) => TypeVariant::Enum,
                // TupleStruct(formats) => formats
                //     .iter()
                //     .enumerate()
                //     .map(|(i, f)| Named {
                //         name: format!("Field{}", i),
                //         value: f.clone(),
                //     })
                //     .collect(),
                _ => todo!("{:?}", format),
            },
        );
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
                        "name": base.to_owned() + name,
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
}
