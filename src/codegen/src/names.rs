use crate::util;

use super::*;
use app::FieldTy;

use std::collections::HashMap;

pub(crate) struct Names {
    pub container_module_name: syn::Ident,

    pub container_alias_name: Option<syn::Ident>,

    pub models: HashMap<app::ModelId, ModelNames>,

    pub fields: HashMap<app::FieldId, FieldNames>,

    /// Structs generated for each relation
    pub relations: HashMap<app::FieldId, RelationNames>,
}

pub(crate) struct ModelNames {
    /// Name of module containing model types
    pub module_name: syn::Ident,

    /// Primary model struct name
    pub struct_name: syn::Ident,

    /// Create model instance builder
    pub create_name: syn::Ident,

    /// Update model instance builder
    pub update_name: syn::Ident,
}

pub(crate) struct FieldNames {
    /// Name when used as a struct field
    pub field_name: syn::Ident,

    /// Name when used as a const
    pub const_name: syn::Ident,
}

pub(crate) struct RelationNames {
    /// Name of the public API type used to access the relation
    pub struct_name: syn::Ident,

    /// If a has_many relation, the singularized name
    pub singular_name: Option<syn::Ident>,
}

impl Names {
    pub(crate) fn from_schema(schema: &app::Schema) -> Names {
        let mut models = HashMap::new();
        let mut fields = HashMap::new();
        let mut relations = HashMap::new();

        for model in schema.models() {
            // Generate model names
            let names = ModelNames::from_model(model);
            models.insert(model.id, names);

            // Find relations and generate associated struct names
            for field in &model.fields {
                let field_name = util::ident(&field.name);
                let struct_name = util::ident(&util::type_name(&field.name));

                fields.insert(
                    field.id,
                    FieldNames {
                        field_name,
                        const_name: util::ident(&util::const_name(&field.name)),
                    },
                );

                match &field.ty {
                    FieldTy::HasMany(rel) => {
                        let singular_name = Some(util::ident(&rel.singular.snake_case()));
                        relations.insert(
                            field.id,
                            RelationNames {
                                struct_name,
                                singular_name,
                            },
                        );
                    }
                    FieldTy::BelongsTo(..) | FieldTy::HasOne(..) => {
                        relations.insert(
                            field.id,
                            RelationNames {
                                struct_name,
                                singular_name: None,
                            },
                        );
                    }
                    FieldTy::Primitive(..) => {}
                }
            }
        }

        Names {
            container_module_name: util::ident("db"),
            container_alias_name: None,
            models,
            fields,
            relations,
        }
    }
}

impl ModelNames {
    fn from_model(model: &app::Model) -> ModelNames {
        let module_name = util::ident(&model.name.snake_case());
        let struct_name = util::ident(&model.name.upper_camel_case());
        let create_name = util::ident(&format!("Create{}", model.name.upper_camel_case()));
        let update_name = util::ident(&format!("Update{}", model.name.upper_camel_case()));

        ModelNames {
            module_name,
            struct_name,
            create_name,
            update_name,
        }
    }
}
