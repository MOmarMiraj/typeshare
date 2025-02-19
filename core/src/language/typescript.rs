use crate::RenameExt;
use crate::{
    language::{Language, SupportedLanguage},
    parser::ParsedData,
    rust_types::{
        RustConst, RustConstExpr, RustEnum, RustEnumVariant, RustField, RustStruct, RustType,
        RustTypeAlias, RustTypeFormatError, SpecialRustType,
    },
};
use itertools::Itertools;
use joinery::JoinableIterator;
use std::{
    collections::HashMap,
    io::{self, Write},
};

use super::ScopedCrateTypes;

/// All information needed to generate Typescript type-code
#[derive(Default)]
pub struct TypeScript {
    /// Mappings from Rust type names to Typescript type names
    pub type_mappings: HashMap<String, String>,
    /// Whether or not to exclude the version header that normally appears at the top of generated code.
    /// If you aren't generating a snapshot test, this setting can just be left as a default (false)
    pub no_version_header: bool,
    /// Whether or not to include the reviver/replacer functions for Uint8Array.
    /// This by default should be false as unless the user expclitly wants to translate to its Uint8Array
    /// representation
    pub is_bytes: bool,
}

impl Language for TypeScript {
    fn type_map(&mut self) -> &HashMap<String, String> {
        &self.type_mappings
    }

    fn format_special_type(
        &mut self,
        special_ty: &SpecialRustType,
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        match special_ty {
            SpecialRustType::Vec(rtype) => {
                // TODO: https://github.com/1Password/typeshare/issues/231
                if rtype.contains_type(SpecialRustType::U8.id()) {
                    if let Some(conversion) =
                        self.type_map().get("Vec<u8>").map(ToString::to_string)
                    {
                        self.is_bytes = true;
                        return Ok(conversion);
                    }
                }
                Ok(format!("{}[]", self.format_type(rtype, generic_types)?))
            }
            SpecialRustType::Array(rtype, len) => {
                let formatted_type = self.format_type(rtype, generic_types)?;
                Ok(format!(
                    "[{}]",
                    std::iter::repeat(&formatted_type)
                        .take(*len)
                        .join_with(", ")
                ))
            }
            SpecialRustType::Slice(rtype) => {
                Ok(format!("{}[]", self.format_type(rtype, generic_types)?))
            }
            // We add optionality above the type formatting level
            SpecialRustType::Option(rtype) => self.format_type(rtype, generic_types),
            SpecialRustType::HashMap(rtype1, rtype2) => Ok(format!(
                "Record<{}, {}>",
                match rtype1.as_ref() {
                    RustType::Simple { id } if generic_types.contains(id) => {
                        return Err(RustTypeFormatError::GenericKeyForbiddenInTS(id.clone()));
                    }
                    _ => self.format_type(rtype1, generic_types)?,
                },
                self.format_type(rtype2, generic_types)?
            )),
            SpecialRustType::Unit => Ok("undefined".into()),
            SpecialRustType::String => Ok("string".into()),
            SpecialRustType::Char => Ok("string".into()),
            SpecialRustType::I8
            | SpecialRustType::U8
            | SpecialRustType::I16
            | SpecialRustType::U16
            | SpecialRustType::I32
            | SpecialRustType::U32
            | SpecialRustType::I54
            | SpecialRustType::U53
            | SpecialRustType::F32
            | SpecialRustType::F64 => Ok("number".into()),
            SpecialRustType::Bool => Ok("boolean".into()),
            SpecialRustType::U64
            | SpecialRustType::I64
            | SpecialRustType::ISize
            | SpecialRustType::USize => {
                panic!("64 bit types not allowed in Typeshare")
            }
        }
    }

    fn begin_file(&mut self, w: &mut dyn Write, _parsed_data: &ParsedData) -> io::Result<()> {
        if !self.no_version_header {
            writeln!(w, "/*")?;
            writeln!(w, " Generated by typeshare {}", env!("CARGO_PKG_VERSION"))?;
            writeln!(w, "*/")?;
            writeln!(w)?;
        }
        Ok(())
    }

    fn write_type_alias(&mut self, w: &mut dyn Write, ty: &RustTypeAlias) -> io::Result<()> {
        self.write_comments(w, 0, &ty.comments)?;

        let r#type = self
            .format_type(&ty.r#type, ty.generic_types.as_slice())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        writeln!(
            w,
            "export type {}{} = {}{};\n",
            ty.id.renamed,
            (!ty.generic_types.is_empty())
                .then(|| format!("<{}>", ty.generic_types.join(", ")))
                .unwrap_or_default(),
            r#type,
            ty.r#type
                .is_optional()
                .then_some(" | undefined")
                .unwrap_or_default(),
        )?;

        Ok(())
    }

    fn write_const(&mut self, w: &mut dyn Write, c: &RustConst) -> io::Result<()> {
        match c.expr {
            RustConstExpr::Int(val) => {
                let const_type = self
                    .format_type(&c.r#type, &[])
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                writeln!(
                    w,
                    "export const {}: {} = {};",
                    c.id.renamed.to_snake_case().to_uppercase(),
                    const_type,
                    val
                )
            }
        }
    }

    fn write_struct(&mut self, w: &mut dyn Write, rs: &RustStruct) -> io::Result<()> {
        self.write_comments(w, 0, &rs.comments)?;
        writeln!(
            w,
            "export interface {}{} {{",
            rs.id.renamed,
            (!rs.generic_types.is_empty())
                .then(|| format!("<{}>", rs.generic_types.join(", ")))
                .unwrap_or_default()
        )?;

        rs.fields
            .iter()
            .try_for_each(|f| self.write_field(w, f, rs.generic_types.as_slice()))?;

        writeln!(w, "}}\n")?;
        rs.fields.iter().try_for_each(|field| {
            let typescript_type = self
                .format_type(&field.ty, &rs.generic_types)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            if self.is_bytes {
                self.is_bytes = false;
                return writeln!(
                    w,
                    r#"export function TypeshareReviver(key: string, value: unknown): unknown {{
    return Array.isArray(value) && value.every(v => Number.isFinite(v) && v >= 0 && v <= 255)  
        ? new {typescript_type}(value) 
        : value;
}}

export function TypeshareReplacer(key: string, value: unknown): unknown {{
    if (value instanceof {typescript_type}) {{
        return Array.from(value);
    }}
    return value;
}}"#
                );
            }
            Ok(())
        })
    }

    fn write_enum(&mut self, w: &mut dyn Write, e: &RustEnum) -> io::Result<()> {
        self.write_comments(w, 0, &e.shared().comments)?;

        let generic_parameters = (!e.shared().generic_types.is_empty())
            .then(|| format!("<{}>", e.shared().generic_types.join(", ")))
            .unwrap_or_default();

        match e {
            RustEnum::Unit(shared) => {
                write!(
                    w,
                    "export enum {}{} {{",
                    shared.id.renamed, generic_parameters
                )?;

                self.write_enum_variants(w, e)?;

                writeln!(w, "\n}}\n")
            }
            RustEnum::Algebraic { shared, .. } => {
                write!(
                    w,
                    "export type {}{} = ",
                    shared.id.renamed, generic_parameters
                )?;

                self.write_enum_variants(w, e)?;

                write!(w, ";")?;
                writeln!(w)?;
                writeln!(w)
            }
        }
    }

    fn write_imports(
        &mut self,
        w: &mut dyn Write,
        imports: ScopedCrateTypes<'_>,
    ) -> std::io::Result<()> {
        for (path, ty) in imports {
            write!(w, "import {{ ")?;
            let ty_list = ty.iter().join(", ");
            write!(w, "{ty_list}")?;
            writeln!(w, " }} from \"./{path}\";")?;
        }
        writeln!(w)
    }

    fn ignored_reference_types(&self) -> Vec<&str> {
        self.type_mappings.keys().map(|s| s.as_str()).collect()
    }
}

impl TypeScript {
    fn write_enum_variants(&mut self, w: &mut dyn Write, e: &RustEnum) -> io::Result<()> {
        match e {
            // Write all the unit variants out (there can only be unit variants in
            // this case)
            RustEnum::Unit(shared) => shared.variants.iter().try_for_each(|v| match v {
                RustEnumVariant::Unit(shared) => {
                    writeln!(w)?;
                    self.write_comments(w, 1, &shared.comments)?;
                    write!(w, "\t{} = {:?},", shared.id.original, &shared.id.renamed)
                }
                _ => unreachable!(),
            }),

            // Write all the algebraic variants out (all three variant types are possible
            // here)
            RustEnum::Algebraic {
                tag_key,
                content_key,
                shared,
            } => shared.variants.iter().try_for_each(|v| {
                writeln!(w)?;
                self.write_comments(w, 1, &v.shared().comments)?;
                match v {
                    RustEnumVariant::Unit(shared) => write!(
                        w,
                        "\t| {{ {}: {:?}, {}?: undefined }}",
                        tag_key, shared.id.renamed, content_key
                    ),
                    RustEnumVariant::Tuple { ty, shared } => {
                        let r#type = self
                            .format_type(ty, e.shared().generic_types.as_slice())
                            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                        write!(
                            w,
                            "\t| {{ {}: {:?}, {}{}: {} }}",
                            tag_key,
                            shared.id.renamed,
                            content_key,
                            ty.is_optional().then_some("?").unwrap_or_default(),
                            r#type
                        )
                    }
                    RustEnumVariant::AnonymousStruct { fields, shared } => {
                        writeln!(
                            w,
                            "\t| {{ {}: {:?}, {}: {{",
                            tag_key, shared.id.renamed, content_key
                        )?;

                        fields.iter().try_for_each(|f| {
                            self.write_field(w, f, e.shared().generic_types.as_slice())
                        })?;

                        write!(w, "}}")?;
                        write!(w, "}}")
                    }
                }
            }),
        }
    }

    fn write_field(
        &mut self,
        w: &mut dyn Write,
        field: &RustField,
        generic_types: &[String],
    ) -> io::Result<()> {
        self.write_comments(w, 1, &field.comments)?;
        let ts_ty: String = match field.type_override(SupportedLanguage::TypeScript) {
            Some(type_override) => type_override.to_owned(),
            None => self
                .format_type(&field.ty, generic_types)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
        };

        let optional = field.ty.is_optional() || field.has_default;
        let double_optional = field.ty.is_double_optional();
        let is_readonly = field
            .decorators
            .get(&SupportedLanguage::TypeScript)
            .filter(|v| v.iter().any(|dec| dec.name() == "readonly"))
            .is_some();
        writeln!(
            w,
            "\t{}{}{}: {}{};",
            is_readonly.then_some("readonly ").unwrap_or_default(),
            typescript_property_aware_rename(&field.id.renamed),
            optional.then_some("?").unwrap_or_default(),
            ts_ty,
            double_optional.then_some(" | null").unwrap_or_default()
        )?;

        Ok(())
    }

    fn write_comments(
        &mut self,
        w: &mut dyn Write,
        indent: usize,
        comments: &[String],
    ) -> io::Result<()> {
        // Only attempt to write a comment if there are some, otherwise we're Ok()
        if !comments.is_empty() {
            let comment: String = {
                let tab_indent = "\t".repeat(indent);
                // If there's only one comment then keep it on the same line, otherwise we'll make a nice multi-line comment
                if comments.len() == 1 {
                    format!("{}/** {} */", tab_indent, comments.first().unwrap())
                } else {
                    let joined_comments = comments.join(&format!("\n{} * ", tab_indent));
                    format!(
                        "{tab}/**
{tab} * {comment}
{tab} */",
                        tab = tab_indent,
                        comment = joined_comments
                    )
                }
            };
            writeln!(w, "{}", comment)?;
        }
        Ok(())
    }
}

fn typescript_property_aware_rename(name: &str) -> String {
    if name.chars().any(|c| c == '-') {
        return format!("{:?}", name);
    }
    name.to_string()
}
