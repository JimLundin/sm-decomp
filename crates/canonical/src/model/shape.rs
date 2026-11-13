//! A State's shape, traced from its serde code: the names and order of its structs' fields and
//! its enums' variants, and the primitive type of each leaf, hashed as the derive hashes a
//! [`Canonical::SCHEMA`](crate::Canonical::SCHEMA).
//!
//! serde shows a type's structure only while it reads a value, and one read takes one variant
//! of each enum, so the tracer reads the type pass after pass, each time taking at each enum a
//! variant not yet read to its end, until every variant of every enum it can reach has been
//! read. A type whose own code refuses the values the tracer gives (a zero where it wants
//! another number, say) can't be traced, and [`shape`] says so.

use std::collections::{BTreeMap, BTreeSet};

use serde::de::{self, DeserializeOwned, DeserializeSeed, IntoDeserializer, Visitor};

use super::Unencodable;
use crate::SchemaHasher;

/// The passes a trace may take: far more than any State's variants need.
const PASSES: usize = 4096;
/// How deep a type may nest: a State is plain data, so deeper means a recursive type.
const DEPTH: usize = 64;

/// The shape of `T`, a model's State.
///
/// # Errors
/// [`Unencodable`] when `T`'s serde code refuses the tracer, reads a form the codec doesn't
/// take, nests too deeply or has too many variants to trace.
pub fn shape<T: DeserializeOwned>() -> Result<u64, Unencodable> {
    let mut registry = Registry::default();
    for _ in 0..PASSES {
        let mut root = String::new();
        registry.incomplete = false;
        T::deserialize(&mut Tracer {
            registry: &mut registry,
            out: &mut root,
            depth: 0,
        })?;
        if !registry.incomplete {
            let hasher = registry.seen.iter().fold(
                SchemaHasher::new().name("serde").name(&root),
                |hasher, part| hasher.name(part),
            );
            return Ok(hasher.finish());
        }
    }
    Err(Unencodable::Custom(String::from(
        "too many variants to trace",
    )))
}

/// What the passes have seen.
#[derive(Default)]
struct Registry {
    /// Each container's description: its kind, name and parts.
    seen: BTreeSet<String>,
    /// Per enum, by its name and variants, whether each variant has been read to its end.
    done: BTreeMap<String, Vec<bool>>,
    /// Whether the value being read reached an enum with a variant not yet read to its end.
    incomplete: bool,
}

struct Tracer<'a> {
    registry: &'a mut Registry,
    out: &'a mut String,
    depth: usize,
}

impl Tracer<'_> {
    /// A tracer writing into `out`, one level deeper.
    fn into<'b>(&'b mut self, out: &'b mut String) -> Result<Tracer<'b>, Unencodable> {
        let depth = self.depth.saturating_add(1);
        if depth > DEPTH {
            return Err(Unencodable::Custom(String::from(
                "the type nests too deeply to be plain data",
            )));
        }
        Ok(Tracer {
            registry: &mut *self.registry,
            out,
            depth,
        })
    }

    fn word(&mut self, word: &str) {
        self.out.push_str(word);
        self.out.push(' ');
    }
}

/// The parts of a container being traced: `labels` names them, or they are counted.
struct Parts<'t, 'a> {
    tracer: &'t mut Tracer<'a>,
    left: usize,
    labels: &'static [&'static str],
    index: usize,
}

impl<'t, 'a> Parts<'t, 'a> {
    const fn new(tracer: &'t mut Tracer<'a>, left: usize, labels: &'static [&'static str]) -> Self {
        Self {
            tracer,
            left,
            labels,
            index: 0,
        }
    }

    fn part<'de, T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Unencodable> {
        if self.left == 0 {
            return Ok(None);
        }
        self.left = self.left.saturating_sub(1);
        if let Some(label) = self.labels.get(self.index) {
            self.tracer.word(label);
        }
        self.index = self.index.saturating_add(1);
        seed.deserialize(&mut *self.tracer).map(Some)
    }
}

impl<'de> de::SeqAccess<'de> for Parts<'_, '_> {
    type Error = Unencodable;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Unencodable> {
        self.part(seed)
    }
}

impl<'de> de::MapAccess<'de> for Parts<'_, '_> {
    type Error = Unencodable;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Unencodable> {
        self.part(seed)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Unencodable> {
        seed.deserialize(&mut *self.tracer)
    }
}

/// The variant a pass takes at an enum.
struct Taken<'t> {
    tracer: Tracer<'t>,
    index: u32,
}

impl<'de, 't> de::EnumAccess<'de> for Taken<'t> {
    type Error = Unencodable;
    type Variant = Tracer<'t>;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Tracer<'t>), Unencodable> {
        let index: de::value::U32Deserializer<Unencodable> = self.index.into_deserializer();
        Ok((seed.deserialize(index)?, self.tracer))
    }
}

impl<'de> de::VariantAccess<'de> for Tracer<'_> {
    type Error = Unencodable;

    fn unit_variant(self) -> Result<(), Unencodable> {
        Ok(())
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(
        mut self,
        seed: T,
    ) -> Result<T::Value, Unencodable> {
        seed.deserialize(&mut self)
    }

    fn tuple_variant<V: Visitor<'de>>(
        mut self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        visitor.visit_seq(Parts::new(&mut self, len, &[]))
    }

    fn struct_variant<V: Visitor<'de>>(
        mut self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        visitor.visit_seq(Parts::new(&mut self, fields.len(), fields))
    }
}

macro_rules! trace_primitive {
    ($($method:ident: $word:literal => $visit:ident($value:expr)),* $(,)?) => {$(
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
            self.word($word);
            visitor.$visit($value)
        }
    )*};
}

/// A form the codec doesn't take, so neither does the tracer.
fn untraceable(form: &str) -> Unencodable {
    Unencodable::Custom(format!(
        "the codec can't read {form}: a type must say what it reads"
    ))
}

impl<'de> de::Deserializer<'de> for &mut Tracer<'_> {
    type Error = Unencodable;

    trace_primitive!(
        deserialize_bool: "bool" => visit_bool(false),
        deserialize_i8: "i8" => visit_i8(0), deserialize_i16: "i16" => visit_i16(0),
        deserialize_i32: "i32" => visit_i32(0), deserialize_i64: "i64" => visit_i64(0),
        deserialize_i128: "i128" => visit_i128(0), deserialize_u8: "u8" => visit_u8(0),
        deserialize_u16: "u16" => visit_u16(0), deserialize_u32: "u32" => visit_u32(0),
        deserialize_u64: "u64" => visit_u64(0), deserialize_u128: "u128" => visit_u128(0),
        deserialize_f32: "f32" => visit_f32(0.0), deserialize_f64: "f64" => visit_f64(0.0),
        deserialize_char: "char" => visit_char('\0'), deserialize_str: "str" => visit_str(""),
        deserialize_string: "str" => visit_str(""), deserialize_bytes: "bytes" => visit_bytes(&[]),
        deserialize_byte_buf: "bytes" => visit_bytes(&[]),
    );

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        self.word("()");
        visitor.visit_unit()
    }

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Unencodable> {
        Err(untraceable("a value that doesn't say its type"))
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        self.word("Option<");
        let value = visitor.visit_some(&mut *self)?;
        self.word(">");
        Ok(value)
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        self.word(name);
        self.registry.seen.insert(format!("unit {name}"));
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        self.word(name);
        let mut own = format!("newtype {name} ( ");
        let value = visitor.visit_newtype_struct(&mut self.into(&mut own)?)?;
        own.push(')');
        self.registry.seen.insert(own);
        Ok(value)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        self.word("seq<");
        let value = visitor.visit_seq(Parts::new(self, 1, &[]))?;
        self.word(">");
        Ok(value)
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        self.word(&format!("tuple{len}("));
        let value = visitor.visit_seq(Parts::new(self, len, &[]))?;
        self.word(")");
        Ok(value)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        self.word(name);
        let mut own = format!("tuple {name} {len} ( ");
        let value = visitor.visit_seq(Parts::new(&mut self.into(&mut own)?, len, &[]))?;
        own.push(')');
        self.registry.seen.insert(own);
        Ok(value)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        self.word("map<");
        let value = visitor.visit_map(Parts::new(self, 1, &[]))?;
        self.word(">");
        Ok(value)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        self.word(name);
        let mut own = format!("struct {name} {{ ");
        let value =
            visitor.visit_seq(Parts::new(&mut self.into(&mut own)?, fields.len(), fields))?;
        own.push('}');
        self.registry.seen.insert(own);
        Ok(value)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        let key = format!("{name}[{}]", variants.join(","));
        self.word(&key);
        let done = self
            .registry
            .done
            .entry(key.clone())
            .or_insert_with(|| vec![false; variants.len()]);
        let index = done.iter().position(|done| !done).unwrap_or(0);
        let variant = variants
            .get(index)
            .ok_or_else(|| Unencodable::Custom(format!("{name} has no variants")))?;
        let parent = std::mem::replace(&mut self.registry.incomplete, false);
        let mut own = format!("variant {key}::{variant} ( ");
        let value = visitor.visit_enum(Taken {
            tracer: self.into(&mut own)?,
            index: u32::try_from(index).unwrap_or(u32::MAX),
        })?;
        own.push(')');
        self.registry.seen.insert(own);
        let read_to_its_end = !self.registry.incomplete;
        let complete = self.registry.done.get_mut(&key).is_some_and(|done| {
            if let Some(slot) = done.get_mut(index) {
                *slot = read_to_its_end;
            }
            done.iter().all(|done| *done)
        });
        self.registry.incomplete = parent || !complete;
        Ok(value)
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Unencodable> {
        Err(untraceable("an identifier"))
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(
        self,
        _visitor: V,
    ) -> Result<V::Value, Unencodable> {
        Err(untraceable("an ignored value"))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}
