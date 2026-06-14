//! Tests for dynamic JSON document fields backed by `serde_json::Value`.

use crate::prelude::*;

#[driver_test(id(ID), requires(document_collections))]
pub async fn serde_json_value_create_get(t: &mut Test) -> Result<(), BoxError> {
    #[derive(Debug, toasty::Model)]
    struct Item {
        #[key]
        #[auto]
        id: ID,
        #[document]
        data: serde_json::Value,
    }

    let mut db = t.setup_db(models!(Item)).await;
    let data = serde_json::json!({
        "a": { "b": { "c": "abc" } },
        "count": 3,
        "enabled": true,
        "tags": ["x", "y"],
        "explicit_null": null
    });

    let item = toasty::create!(Item { data: data.clone() })
        .exec(&mut db)
        .await?;
    let loaded = Item::get_by_id(&mut db, &item.id).await?;

    assert_eq!(loaded.data, data);

    Ok(())
}

#[driver_test(id(ID), requires(document_collections))]
pub async fn serde_json_value_update_whole_field(t: &mut Test) -> Result<(), BoxError> {
    #[derive(Debug, toasty::Model)]
    struct Item {
        #[key]
        #[auto]
        id: ID,
        #[document]
        data: serde_json::Value,
    }

    let mut db = t.setup_db(models!(Item)).await;
    let mut item = toasty::create!(Item {
        data: serde_json::json!({ "status": "old" })
    })
    .exec(&mut db)
    .await?;

    let updated = serde_json::json!(["new", { "explicit_null": null }]);
    item.update().data(updated.clone()).exec(&mut db).await?;
    let loaded = Item::get_by_id(&mut db, &item.id).await?;

    assert_eq!(item.data, updated);
    assert_eq!(loaded.data, updated);

    Ok(())
}

#[driver_test(id(ID), requires(document_collections))]
pub async fn serde_json_value_point_eq_string(t: &mut Test) -> Result<(), BoxError> {
    #[derive(Debug, toasty::Model)]
    struct Item {
        #[key]
        #[auto]
        id: ID,
        #[document]
        data: serde_json::Value,
    }

    let mut db = t.setup_db(models!(Item)).await;
    let matching = toasty::create!(Item {
        data: serde_json::json!({ "a": { "b": { "c": "abc" } } })
    })
    .exec(&mut db)
    .await?;
    toasty::create!(Item {
        data: serde_json::json!({ "a": { "b": { "c": "def" } } })
    })
    .exec(&mut db)
    .await?;

    let items = Item::filter(Item::fields().data().point::<String>("a/b/c").eq("abc"))
        .exec(&mut db)
        .await?;

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, matching.id);

    Ok(())
}

#[driver_test(id(ID), requires(document_collections))]
pub async fn serde_json_value_point_ne_string(t: &mut Test) -> Result<(), BoxError> {
    #[derive(Debug, toasty::Model)]
    struct Item {
        #[key]
        #[auto]
        id: ID,
        #[document]
        data: serde_json::Value,
    }

    let mut db = t.setup_db(models!(Item)).await;
    let matching = toasty::create!(Item {
        data: serde_json::json!({ "status": "active" })
    })
    .exec(&mut db)
    .await?;
    toasty::create!(Item {
        data: serde_json::json!({ "status": "archived" })
    })
    .exec(&mut db)
    .await?;

    let items = Item::filter(
        Item::fields()
            .data()
            .point::<String>("status")
            .ne("archived"),
    )
    .exec(&mut db)
    .await?;

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, matching.id);

    Ok(())
}

#[driver_test(id(ID), requires(and(document_collections, sql)))]
pub async fn serde_json_value_point_special_segments(t: &mut Test) -> Result<(), BoxError> {
    #[derive(Debug, toasty::Model)]
    struct Item {
        #[key]
        #[auto]
        id: ID,
        #[document]
        data: serde_json::Value,
    }

    let mut db = t.setup_db(models!(Item)).await;
    let matching = toasty::create!(Item {
        data: serde_json::json!({
            "a.b": { "quoted\"key": { "with/slash": "abc" } }
        })
    })
    .exec(&mut db)
    .await?;
    toasty::create!(Item {
        data: serde_json::json!({
            "a": { "b": { "quoted\"key": { "with/slash": "def" } } }
        })
    })
    .exec(&mut db)
    .await?;

    let items = Item::filter(
        Item::fields()
            .data()
            .point::<String>("a.b/quoted\"key/with~1slash")
            .eq("abc"),
    )
    .exec(&mut db)
    .await?;

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, matching.id);

    Ok(())
}

#[driver_test(id(ID), requires(document_collections))]
pub async fn serde_json_value_point_eq_bool_and_i64(t: &mut Test) -> Result<(), BoxError> {
    #[derive(Debug, toasty::Model)]
    struct Item {
        #[key]
        #[auto]
        id: ID,
        #[document]
        data: serde_json::Value,
    }

    let mut db = t.setup_db(models!(Item)).await;
    let matching = toasty::create!(Item {
        data: serde_json::json!({ "count": 3, "enabled": true })
    })
    .exec(&mut db)
    .await?;
    toasty::create!(Item {
        data: serde_json::json!({ "count": 4, "enabled": true })
    })
    .exec(&mut db)
    .await?;
    toasty::create!(Item {
        data: serde_json::json!({ "count": 3, "enabled": false })
    })
    .exec(&mut db)
    .await?;

    let items = Item::filter(
        Item::fields()
            .data()
            .point::<i64>("count")
            .eq(3)
            .and(Item::fields().data().point::<bool>("enabled").eq(true)),
    )
    .exec(&mut db)
    .await?;

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, matching.id);

    Ok(())
}
