use convert_case::{Case, Casing};
use serde::Serialize;

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberType {
    Basic,
    Vector,
    Optional,
    OptionalVector,
}

#[derive(Serialize, Debug)]
pub struct StructMember {
    pub member_type: MemberType,
    pub name: String,
    pub type_name: String,
    pub json_name: String,
    pub tag: Option<String>,
    pub dont_fail_on_deserialization: bool,
    #[serde(skip)]
    pub fixed_name: bool,
}

#[derive(Serialize, Debug)]
pub struct StructDecl {
    pub full_name: String,
    pub inner_root: String,
    pub members: Vec<StructMember>,
}

impl StructDecl {
    pub fn apply_commands<'a>(&mut self, commands: impl Iterator<Item = (&'a str, &'a str)>) {
        for command in commands {
            match command {
                ("json_transform", "snake_case") => {
                    for member in &mut self.members {
                        if !member.fixed_name {
                            member.json_name = member.json_name.to_case(Case::Snake);
                        }
                    }
                }
                ("json_inner", value) => self.inner_root = value.to_owned(),
                _ => eprintln!("Unknown command: {command:?}"),
            }
        }
    }
}

impl StructMember {
    pub fn apply_commands<'a>(&mut self, commands: impl Iterator<Item = (&'a str, &'a str)>) {
        for command in commands {
            match command {
                ("json_rename", value) => {
                    self.json_name = value.to_string();
                    self.fixed_name = true;
                }
                ("json_dont_fail_on_deserialization", value) => {
                    self.dont_fail_on_deserialization = value == "true" // TODO: ignore case?
                }
                ("json_tag", value) => self.tag = Some(value.to_string()),
                ("json_transform", "snake_case") => {
                    self.json_name = self.json_name.to_case(Case::Snake);
                    self.fixed_name = true;
                }
                _ => eprintln!("Unknown command: {command:?}"),
            }
        }
    }
}

macro_rules! enum_global {
    ($target:ident for $name:ty { $($variant:ident = $member:ident,)* }) => {
        pub struct $target;

        impl minijinja::value::StructObject for $target {
            fn get_field(&self, name: &str) -> Option<minijinja::Value> {
                match name {
                    $(stringify!($member) => Some(minijinja::Value::from_serializable(&<$name>::$variant)),)*
                    _ => None,
                }
            }

            fn static_fields(&self) -> Option<&'static [&'static str]> {
                Some(&[$(stringify!($member),)*][..])
            }
        }

        // ensure all variants are covered
        #[doc(hidden)]
        #[allow(unused)]
        const _: () = {
            const fn check_variants(e: &$name) -> u8 {
                match e {
                    $(<$name>::$variant => 0,)*
                }
            }
        };
    };
}

enum_global! {
    MemberTypeGlobal for MemberType {
        Basic = BASIC,
        Vector = VECTOR,
        Optional = OPTIONAL,
        OptionalVector = OPTIONAL_VECTOR,
    }
}
