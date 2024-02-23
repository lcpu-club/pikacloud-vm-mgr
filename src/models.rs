use rustcracker::{
    components::machine::Config,
    model::{
        full_vm_configuration::{self, FullVmConfiguration},
        instance_info::InstanceInfo,
    },
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, PgPool, Row};
use chrono::prelude::*;
use uuid::Uuid;
use std::path::PathBuf;

use crate::error::VmManageError;

// Models for VM manage related requests and responses

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmCreateRequest {
    pub config: MachineCreateConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmCreateResponse {
    pub vmid: Uuid,
    pub created_at: chrono::DateTime<Local>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmQueryStatusRequest {
    pub vmid: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmQueryStatusResponse {
    pub vmid: Uuid,
    pub info: VmViewInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Operation {
    Start,
    Pause,
    Resume,
    Stop,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmOperateRequest {
    pub vmid: Uuid,
    pub operation: Operation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmOperateResponse {
    pub vmid: Uuid,
    pub time: chrono::DateTime<Local>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmDeleteRequest {
    pub vmid: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmDeleteResponse {
    pub vmid: Uuid,
    pub time: chrono::DateTime<Local>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmCreateSnapshotRequest {
    pub vmid: Uuid,
    pub snapshot_path: PathBuf,
    pub memory_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmLoadSnapshotRequest {
    pub vmid: Uuid,
    pub snapshot_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmModifyMetadataRequest {
    pub vmid: Uuid,
    pub metadata: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmModifyMetadataResponse {
    pub vmid: Uuid,
    pub time: chrono::DateTime<Local>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmSnapshotDetailRequest {
    pub vmid: Uuid,
    pub snapshot_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmRestoreAllRequest {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmRestoreAllResponse {
    pub infos: Vec<VmViewInfo>,
}

// Schemas

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MachineCreateConfig {
    pub memory_size_in_mib: i32,
    pub vcpu_count: i32,
    pub kernel_name: String,
    pub kernel_version: String,
    pub enable_hyperthreading: Option<bool>,
    pub initial_metadata: Option<String>,
    pub volume_size_in_mib: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmViewConfig {
    user_id: Option<Uuid>,
    vmid: Option<Uuid>,
    config: Option<sqlx::types::Json<full_vm_configuration::FullVmConfiguration>>,
    execute_dir: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmViewInfo {
    pub vmid: Uuid,
    pub vm_info: InstanceInfo,
    pub full_config: FullVmConfiguration,
    pub boot_config: Config,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotInfo {
    pub snapshot_id: Uuid,
    pub snapshot_path: PathBuf,
    pub memory_path: PathBuf,
    pub created_at: DateTime<Local>,
}


pub(crate) const MAX_CACHE_CAPACITY_ENV: &'static str = "MAX_CACHE_CAPACITY";
pub(crate) const DEFAULT_MAX_CACHE_CAPACITY: u64 = 10000;

pub(crate) const MACHINE_CORE_TABLE_NAME: &'static str = "MACHINE_CORE_TABLE_NAME";
pub(crate) const DEFAULT_MACHINE_CORE_TABLE: &'static str = "machine_core";

pub(crate) const VM_CONFIG_TABLE_NAME: &'static str = "VM_CONFIG_TABLE_NAME";
pub(crate) const DEFAULT_VM_CONFIG_TABLE: &'static str = "vmconfig";

pub(crate) const VM_MEM_SNAPSHOT_TABLE_NAME: &'static str = "SNAPSHOT_TABLE_NAME";
pub(crate) const DEFAULT_VM_MEM_SNAPSHOT_TABLE: &'static str = "snapshots";

// SQLs

const CREATE_VMVIEWCONFIGS_TABLE_SQL: &'static str = r#"
    CREATE TABLE if not exists $1 (
        id                  INT SERIAL PRIMARY KEY,
        user_id             VARCHAR(128) NOT NULL,
        vmid                VARCHAR(128) NOT NULL,
        execute_dir         VARCHAR(256) NOT NULL,
        config              JSON
    );
"#;

const CREATE_MACHINE_CORE_TABLE_SQL: &'static str = r#"
    CREATE TABLE if not exists $1 (
        vmid                UUID,
        machine_core        JSON
    );
"#;

const DROP_VMVIEWCONFIGS_TABLE_SQL: &'static str = r#"
    DROP TABLE if exists $1;
"#;

const DROP_MAHCINE_CORE_TABLE_SQL: &'static str = r#"
    DROP TABLE if exists $1;
"#;

const GET_VMVIEWCONFIGS_SQL: &'static str = r#"
    SELECT * FROM $1;
"#;

const GET_VMVIEWCONFIGS_BY_USER_ID: &'static str = r#"
    SELECT * FROM $1 WHERE user_id = $2;
"#;

const GET_VMVIEWCONFIGS_BY_VMID: &'static str = r#"
    SELECT * FROM $1 WHERE vmid = $2;
"#;

const GET_MACHINE_CORE: &'static str = r#"
    SELECT * FROM $1;
"#;

const GET_MACHINE_CORE_BY_VMID: &'static str = r#"
    SELECT * FROM $1 WHERE vmid = $2;
"#;

pub async fn drop_vmviewconfigs_table(db: &PgPool) -> Result<(), VmManageError> {
    sqlx::query(DROP_VMVIEWCONFIGS_TABLE_SQL)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn create_vmviewconfigs_table(db: &PgPool) -> Result<(), VmManageError> {
    let _ = sqlx::query(CREATE_VMVIEWCONFIGS_TABLE_SQL).execute(db).await?;
    Ok(())
}

// 获取所有的虚拟机config信息
pub async fn get_configs(db: &PgPool) -> Result<Vec<VmViewConfig>, VmManageError> {
    Ok(sqlx::query(GET_VMVIEWCONFIGS_SQL)
        .map(|row: PgRow| VmViewConfig {
            user_id: row.try_get("user_id").ok(),
            vmid: row.try_get("vmid").ok(),
            execute_dir: row.try_get("execute_dir").ok(),
            config: row.try_get("config").ok(),
        })
        .fetch_all(db)
        .await?)
}

pub async fn get_config_by_user_id(db: &PgPool) -> Result<Vec<VmViewConfig>, VmManageError> {
    Ok(sqlx::query(GET_VMVIEWCONFIGS_BY_USER_ID)
        .map(|row: PgRow| VmViewConfig {
            user_id: row.try_get("user_id").ok(),
            vmid: row.try_get("vmid").ok(),
            execute_dir: row.try_get("execute_dir").ok(),
            config: row.try_get("config").ok(),
        })
        .fetch_all(db)
        .await?)
}
