use crate::configs::{DQComputeConfig, DQFilesystemConfig, DQTableConfig};
use crate::error::DQError;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, SchemaRef};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use sqlparser::ast::Statement;
use std::collections::HashMap;
use tokio::sync::Mutex;

static COMPUTE_FACTORIES: Lazy<Mutex<HashMap<String, Box<dyn DQComputeFactory>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[async_trait]
pub trait DQCompute: Send + Sync {
    async fn select(
        &mut self,
        statement: &Statement,
        schema: Option<SchemaRef>,
        files: Vec<String>,
    ) -> Result<Vec<RecordBatch>, DQError>;

    fn get_function_return_type(
        &self,
        _func: &String,
        _args: &Vec<String>,
        _schema: SchemaRef,
    ) -> Option<DataType> {
        None
    }
}

#[async_trait]
pub trait DQComputeFactory: Send + Sync {
    async fn create(
        &self,
        table_config: &DQTableConfig,
        compute_config: Option<&DQComputeConfig>,
        filesystem_config: Option<&DQFilesystemConfig>,
    ) -> Box<dyn DQCompute>;
}

pub async fn register_compute_factory(name: &str, factory: Box<dyn DQComputeFactory>) {
    let mut factories = COMPUTE_FACTORIES.lock().await;
    factories.insert(name.to_string(), factory);
}

pub async fn create_compute_using_factory(
    name: &str,
    table_config: &DQTableConfig,
    compute_config: Option<&DQComputeConfig>,
    filesystem_config: Option<&DQFilesystemConfig>,
) -> Option<Box<dyn DQCompute>> {
    let factories = COMPUTE_FACTORIES.lock().await;
    if let Some(factory) = factories.get(name) {
        let compute: Box<dyn DQCompute> = factory
            .create(table_config, compute_config, filesystem_config)
            .await;
        Some(compute)
    } else {
        None
    }
}
