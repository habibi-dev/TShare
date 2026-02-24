use crate::features::setting::entity::prelude::Setting;
use crate::features::setting::entity::setting;
use crate::features::setting::entity::setting::{ActiveModel, Model};
use crate::utility::state::app_state;
use sea_orm::*;

pub struct SettingService;

impl SettingService {
    /// Insert or update a setting based on meta_key uniqueness
    pub async fn upsert(meta_key: String, meta_value: Option<String>) -> Result<Model, DbErr> {
        let db = &app_state()._db;

        // Check if the meta_key already exists
        let existing = Setting::find()
            .filter(setting::Column::MetaKey.eq(&meta_key))
            .one(db)
            .await?;

        match existing {
            Some(model) => {
                // Update existing record
                let mut active_model: ActiveModel = model.into();
                active_model.meta_value = Set(meta_value);
                active_model.updated_at = Set(chrono::Utc::now().naive_utc());
                active_model.update(db).await
            }
            None => {
                // Insert new record
                let now = chrono::Utc::now().naive_utc();
                let new_setting = ActiveModel {
                    id: NotSet,
                    meta_key: Set(meta_key),
                    meta_value: Set(meta_value),
                    created_at: Set(now),
                    updated_at: Set(now),
                };
                new_setting.insert(db).await
            }
        }
    }

    /// Get a setting by meta_key
    pub async fn get_by_key(meta_key: &str) -> Result<Option<Model>, DbErr> {
        let db = &app_state()._db;

        Setting::find()
            .filter(setting::Column::MetaKey.eq(meta_key))
            .one(db)
            .await
    }

    /// Get all settings
    pub async fn get_all() -> Result<Vec<Model>, DbErr> {
        let db = &app_state()._db;

        Setting::find().all(db).await
    }

    /// Delete a setting by meta_key
    pub async fn delete_by_key(meta_key: &str) -> Result<DeleteResult, DbErr> {
        let db = &app_state()._db;

        Setting::delete_many()
            .filter(setting::Column::MetaKey.eq(meta_key))
            .exec(db)
            .await
    }

    /// Update only the meta_value for an existing meta_key
    pub async fn update_value(
        meta_key: &str,
        meta_value: Option<String>,
    ) -> Result<Option<Model>, DbErr> {
        let db = &app_state()._db;

        let existing = Setting::find()
            .filter(setting::Column::MetaKey.eq(meta_key))
            .one(db)
            .await?;

        match existing {
            Some(model) => {
                let mut active_model: ActiveModel = model.into();
                active_model.meta_value = Set(meta_value);
                active_model.updated_at = Set(chrono::Utc::now().naive_utc());
                Ok(Some(active_model.update(db).await?))
            }
            None => Ok(None),
        }
    }
}
