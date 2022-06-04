use crate::{
    domain::services::library::LibraryService,
    infrastructure::{auth::Claims, domain::repositories::library::LibraryRepositoryImpl},
    presentation::graphql::loader::{DatabaseLoader, UserCategoryId},
};
use async_graphql::{dataloader::DataLoader, Context, Object, Result};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Debug, Clone)]
pub struct Category {
    id: Option<i64>,
    name: String,
}

impl Default for Category {
    fn default() -> Self {
        Self {
            id: None,
            name: "Default".to_string(),
        }
    }
}
impl From<crate::domain::entities::library::Category> for Category {
    fn from(val: crate::domain::entities::library::Category) -> Self {
        Self {
            id: val.id,
            name: val.name,
        }
    }
}

#[Object]
impl Category {
    async fn id(&self) -> Option<i64> {
        self.id
    }

    async fn name(&self) -> String {
        self.name.clone()
    }

    async fn count(&self, ctx: &Context<'_>) -> Result<i64> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        Ok(ctx
            .data::<DataLoader<DatabaseLoader>>()?
            .load_one(UserCategoryId(claims.sub, self.id))
            .await?
            .unwrap_or(0))
    }
}

#[derive(Default)]
pub struct CategoryRoot;

#[Object]
impl CategoryRoot {
    async fn get_categories(&self, ctx: &Context<'_>) -> Result<Vec<Category>> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let categories = ctx
            .data::<LibraryService<LibraryRepositoryImpl>>()?
            .get_categories_by_user_id(claims.sub)
            .await?
            .into_par_iter()
            .map(|cat| cat.into())
            .collect();

        Ok(categories)
    }

    async fn get_category(&self, ctx: &Context<'_>, id: Option<i64>) -> Result<Category> {
        let _ = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let category = ctx
            .data::<LibraryService<LibraryRepositoryImpl>>()?
            .get_category_by_id(id)
            .await?
            .into();

        Ok(category)
    }
}

#[derive(Default)]
pub struct CategoryMutationRoot;

#[Object]
impl CategoryMutationRoot {
    async fn create_category(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "category name")] name: String,
    ) -> Result<Category> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let category = ctx
            .data::<LibraryService<LibraryRepositoryImpl>>()?
            .create_category(claims.sub, &name)
            .await?
            .into();

        Ok(category)
    }

    async fn update_category(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "category id")] id: i64,
        #[graphql(desc = "category name")] name: String,
    ) -> Result<Category> {
        let _ = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let category = ctx
            .data::<LibraryService<LibraryRepositoryImpl>>()?
            .rename_category(id, &name)
            .await?
            .into();

        Ok(category)
    }

    async fn delete_category(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "category id")] id: i64,
    ) -> Result<u64> {
        let _ = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<LibraryService<LibraryRepositoryImpl>>()?
            .delete_category(id)
            .await?;

        Ok(1)
    }
}
