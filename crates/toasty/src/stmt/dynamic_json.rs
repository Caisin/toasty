use super::{Expr, IntoExpr, List, Path};

use crate::schema::{Document, Field, Load};
use toasty_core::schema::app::{FieldPrimitive, FieldTy};
use toasty_core::{schema::db, stmt};

/// A typed path to a dynamic JSON document field.
#[derive(Debug)]
pub struct JsonPath<Origin> {
    path: Path<Origin, serde_json::Value>,
}

impl<Origin> JsonPath<Origin> {
    /// Creates a dynamic JSON document path wrapper from the field path.
    pub fn new(path: Path<Origin, serde_json::Value>) -> Self {
        Self { path }
    }

    /// Extracts a typed scalar leaf from this JSON document using a slash-separated path.
    pub fn point<T>(self, path: impl AsRef<str>) -> Expr<T>
    where
        T: JsonLeaf,
    {
        let path = parse_point_path(path.as_ref());
        let base = Box::new(stmt::Path::from(self.path).into_stmt());
        Expr::from_untyped(stmt::FuncJsonExtract {
            base,
            path,
            ty: T::json_leaf_ty(),
        })
    }
}

impl<Origin> Clone for JsonPath<Origin> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
        }
    }
}

impl<Origin> From<JsonPath<Origin>> for Path<Origin, serde_json::Value> {
    fn from(value: JsonPath<Origin>) -> Self {
        value.path
    }
}

impl<Origin> From<JsonPath<Origin>> for stmt::Path {
    fn from(value: JsonPath<Origin>) -> Self {
        value.path.into()
    }
}

/// Rust types that can be used as typed leaves of a dynamic JSON point lookup.
pub trait JsonLeaf {
    /// The Toasty statement type of this JSON leaf.
    fn json_leaf_ty() -> stmt::Type;
}

macro_rules! impl_json_leaf {
    ($($ty:ty => $stmt_ty:ident),* $(,)?) => {
        $(
            impl JsonLeaf for $ty {
                fn json_leaf_ty() -> stmt::Type {
                    stmt::Type::$stmt_ty
                }
            }
        )*
    };
}

impl_json_leaf! {
    bool => Bool,
    String => String,
    i8 => I8,
    i16 => I16,
    i32 => I32,
    i64 => I64,
    u8 => U8,
    u16 => U16,
    u32 => U32,
    u64 => U64,
    f32 => F32,
    f64 => F64,
    uuid::Uuid => Uuid,
}

impl<T> JsonLeaf for Option<T>
where
    T: JsonLeaf,
{
    fn json_leaf_ty() -> stmt::Type {
        T::json_leaf_ty()
    }
}

fn parse_point_path(path: &str) -> Vec<String> {
    let path = path.strip_prefix('/').unwrap_or(path);
    assert!(!path.is_empty(), "dynamic JSON point path cannot be empty");
    path.split('/').map(unescape_point_segment).collect()
}

fn unescape_point_segment(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    let mut chars = segment.chars();
    while let Some(ch) = chars.next() {
        if ch == '~' {
            match chars.next() {
                Some('0') => out.push('~'),
                Some('1') => out.push('/'),
                Some(other) => {
                    out.push('~');
                    out.push(other);
                }
                None => out.push('~'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

impl Load for serde_json::Value {
    type Output = Self;

    fn ty() -> stmt::Type {
        stmt::Type::Json
    }

    fn load(value: stmt::Value) -> crate::Result<Self> {
        stmt_value_to_json(value)
    }

    fn reload(target: &mut Self, value: stmt::Value) -> crate::Result<()> {
        *target = Self::load(value)?;
        Ok(())
    }
}

impl Field for serde_json::Value {
    type ExprTarget = Self;
    type Path<Origin> = JsonPath<Origin>;
    type ListPath<Origin> = Path<Origin, List<Self::ExprTarget>>;
    type Update<'a> = ();
    type Inner = Self;

    fn new_path<Origin>(path: Path<Origin, Self>) -> Self::Path<Origin> {
        JsonPath::new(path)
    }

    fn new_list_path<Origin>(path: Path<Origin, List<Self::ExprTarget>>) -> Self::ListPath<Origin> {
        path
    }

    fn new_update<'a>(
        _assignments: &'a mut toasty_core::stmt::Assignments,
        _projection: toasty_core::stmt::Projection,
    ) -> Self::Update<'a> {
    }

    fn field_ty(storage_ty: Option<db::Type>) -> FieldTy {
        FieldTy::Primitive(FieldPrimitive {
            ty: stmt::Type::Json,
            storage_ty,
            serialize: None,
        })
    }

    fn key_constraint<Origin>(&self, _target: Path<Origin, Self::Inner>) -> Expr<bool> {
        unreachable!("serde_json::Value fields cannot be used as foreign-key targets")
    }
}

impl Document for serde_json::Value {}

impl IntoExpr<serde_json::Value> for serde_json::Value {
    fn into_expr(self) -> Expr<serde_json::Value> {
        Expr::from_value(json_to_stmt_value(self))
    }

    fn by_ref(&self) -> Expr<serde_json::Value> {
        Expr::from_value(json_to_stmt_value(self.clone()))
    }
}

impl super::assignment::Assign<serde_json::Value> for serde_json::Value {
    fn into_assignment(self) -> super::assignment::Assignment<serde_json::Value> {
        super::set(<Self as IntoExpr<serde_json::Value>>::into_expr(self))
    }
}

fn json_to_stmt_value(value: serde_json::Value) -> stmt::Value {
    match value {
        serde_json::Value::Null => stmt::Value::Null,
        serde_json::Value::Bool(value) => stmt::Value::Bool(value),
        serde_json::Value::Number(value) => json_number_to_stmt_value(value),
        serde_json::Value::String(value) => stmt::Value::String(value),
        serde_json::Value::Array(values) => {
            stmt::Value::List(values.into_iter().map(json_to_stmt_value).collect())
        }
        serde_json::Value::Object(values) => stmt::Value::Object(stmt::ValueObject::from_json_vec(
            values
                .into_iter()
                .map(|(key, value)| (key, json_to_stmt_value(value)))
                .collect(),
        )),
    }
}

fn json_number_to_stmt_value(value: serde_json::Number) -> stmt::Value {
    if let Some(value) = value.as_i64() {
        stmt::Value::I64(value)
    } else if let Some(value) = value.as_u64() {
        stmt::Value::U64(value)
    } else if let Some(value) = value.as_f64() {
        stmt::Value::F64(value)
    } else {
        stmt::Value::Null
    }
}

fn stmt_value_to_json(value: stmt::Value) -> crate::Result<serde_json::Value> {
    Ok(match value {
        stmt::Value::Null => serde_json::Value::Null,
        stmt::Value::Bool(value) => serde_json::Value::Bool(value),
        stmt::Value::I8(value) => serde_json::json!(value),
        stmt::Value::I16(value) => serde_json::json!(value),
        stmt::Value::I32(value) => serde_json::json!(value),
        stmt::Value::I64(value) => serde_json::json!(value),
        stmt::Value::U8(value) => serde_json::json!(value),
        stmt::Value::U16(value) => serde_json::json!(value),
        stmt::Value::U32(value) => serde_json::json!(value),
        stmt::Value::U64(value) => serde_json::json!(value),
        stmt::Value::F32(value) => serde_json::json!(value),
        stmt::Value::F64(value) => serde_json::json!(value),
        stmt::Value::String(value) => serde_json::Value::String(value),
        stmt::Value::Uuid(value) => serde_json::Value::String(value.to_string()),
        stmt::Value::List(values) => serde_json::Value::Array(
            values
                .into_iter()
                .map(stmt_value_to_json)
                .collect::<crate::Result<Vec<_>>>()?,
        ),
        stmt::Value::Object(object) => serde_json::Value::Object(
            object
                .entries
                .into_iter()
                .map(|(key, value)| stmt_value_to_json(value).map(|value| (key, value)))
                .collect::<crate::Result<serde_json::Map<_, _>>>()?,
        ),
        other => {
            return Err(toasty_core::Error::type_conversion(
                other,
                "serde_json::Value",
            ));
        }
    })
}
