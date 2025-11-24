use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::{HeatpumpQueryParams, HeatpumpReading};
use chrono::{DateTime, Utc};
use sqlx::Row;

#[derive(Clone)]
pub struct HeatpumpRepository {
    pool: DbPool,
}

impl HeatpumpRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self, params: &HeatpumpQueryParams) -> Result<Vec<HeatpumpReading>> {
        let mut query = String::from(
            "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                    hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                    flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                    runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                    hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
             FROM heatpump WHERE 1=1"
        );

        let mut conditions = Vec::new();
        let mut arg_index = 1;

        if let Some(device_id) = &params.device_id {
            conditions.push(format!("device_id = ${}", arg_index));
            arg_index += 1;
        }

        if let Some(room) = &params.room {
            conditions.push(format!("room = ${}", arg_index));
            arg_index += 1;
        }

        if let Some(start_time) = &params.start_time {
            conditions.push(format!("ts >= ${}", arg_index));
            arg_index += 1;
        }

        if let Some(end_time) = &params.end_time {
            conditions.push(format!("ts <= ${}", arg_index));
            arg_index += 1;
        }

        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY ts DESC");

        let limit = params.limit.unwrap_or(100);
        let offset = params.offset.unwrap_or(0);
        query.push_str(&format!(" LIMIT ${} OFFSET ${}", arg_index, arg_index + 1));

        let mut sql_query = sqlx::query(&query);

        if let Some(device_id) = &params.device_id {
            sql_query = sql_query.bind(device_id);
        }

        if let Some(room) = &params.room {
            sql_query = sql_query.bind(room);
        }

        if let Some(start_time) = &params.start_time {
            sql_query = sql_query.bind(start_time);
        }

        if let Some(end_time) = &params.end_time {
            sql_query = sql_query.bind(end_time);
        }

        sql_query = sql_query.bind(limit).bind(offset);

        let rows = sql_query.fetch_all(&self.pool).await?;

        let readings: Vec<HeatpumpReading> = rows
            .iter()
            .map(|row| {
                HeatpumpReading {
                    ts: row.get("ts"),
                    device_id: row.get("device_id"),
                    room: row.get("room"),
                    outdoor_temp: row.get("outdoor_temp"),
                    supplyline_temp: row.get("supplyline_temp"),
                    returnline_temp: row.get("returnline_temp"),
                    hotwater_temp: row.get("hotwater_temp"),
                    brine_out_temp: row.get("brine_out_temp"),
                    brine_in_temp: row.get("brine_in_temp"),
                    integral: row.get("integral"),
                    flowlinepump_speed: row.get("flowlinepump_speed"),
                    brinepump_speed: row.get("brinepump_speed"),
                    runtime_compressor: row.get("runtime_compressor"),
                    runtime_hotwater: row.get("runtime_hotwater"),
                    runtime_3kw: row.get("runtime_3kw"),
                    runtime_6kw: row.get("runtime_6kw"),
                    brinepump_on: row.get("brinepump_on"),
                    compressor_on: row.get("compressor_on"),
                    flowlinepump_on: row.get("flowlinepump_on"),
                    hotwater_production: row.get("hotwater_production"),
                    circulation_pump: row.get("circulation_pump"),
                    aux_heater_3kw_on: row.get("aux_heater_3kw_on"),
                    aux_heater_6kw_on: row.get("aux_heater_6kw_on"),
                }
            })
            .collect();

        Ok(readings)
    }

    pub async fn count(&self, params: &HeatpumpQueryParams) -> Result<i64> {
        let mut query = String::from("SELECT COUNT(*) as count FROM heatpump WHERE 1=1");

        let mut conditions = Vec::new();
        let mut arg_index = 1;

        if let Some(device_id) = &params.device_id {
            conditions.push(format!("device_id = ${}", arg_index));
            arg_index += 1;
        }

        if let Some(room) = &params.room {
            conditions.push(format!("room = ${}", arg_index));
            arg_index += 1;
        }

        if let Some(start_time) = &params.start_time {
            conditions.push(format!("ts >= ${}", arg_index));
            arg_index += 1;
        }

        if let Some(end_time) = &params.end_time {
            conditions.push(format!("ts <= ${}", arg_index));
            arg_index += 1;
        }

        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }

        let mut sql_query = sqlx::query(&query);

        if let Some(device_id) = &params.device_id {
            sql_query = sql_query.bind(device_id);
        }

        if let Some(room) = &params.room {
            sql_query = sql_query.bind(room);
        }

        if let Some(start_time) = &params.start_time {
            sql_query = sql_query.bind(start_time);
        }

        if let Some(end_time) = &params.end_time {
            sql_query = sql_query.bind(end_time);
        }

        let row = sql_query.fetch_one(&self.pool).await?;
        let count: i64 = row.get("count");

        Ok(count)
    }

    pub async fn find_by_id(&self, ts: DateTime<Utc>, device_id: Option<String>) -> Result<HeatpumpReading> {
        let query = if let Some(device_id) = device_id {
            sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE ts = $1 AND device_id = $2 
                 ORDER BY ts DESC LIMIT 1"
            )
            .bind(ts)
            .bind(device_id)
        } else {
            sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE ts = $1 
                 ORDER BY ts DESC LIMIT 1"
            )
            .bind(ts)
        };

        let row = query.fetch_optional(&self.pool).await?;

        match row {
            Some(row) => Ok(HeatpumpReading {
                ts: row.get("ts"),
                device_id: row.get("device_id"),
                room: row.get("room"),
                outdoor_temp: row.get("outdoor_temp"),
                supplyline_temp: row.get("supplyline_temp"),
                returnline_temp: row.get("returnline_temp"),
                hotwater_temp: row.get("hotwater_temp"),
                brine_out_temp: row.get("brine_out_temp"),
                brine_in_temp: row.get("brine_in_temp"),
                integral: row.get("integral"),
                flowlinepump_speed: row.get("flowlinepump_speed"),
                brinepump_speed: row.get("brinepump_speed"),
                runtime_compressor: row.get("runtime_compressor"),
                runtime_hotwater: row.get("runtime_hotwater"),
                runtime_3kw: row.get("runtime_3kw"),
                runtime_6kw: row.get("runtime_6kw"),
                brinepump_on: row.get("brinepump_on"),
                compressor_on: row.get("compressor_on"),
                flowlinepump_on: row.get("flowlinepump_on"),
                hotwater_production: row.get("hotwater_production"),
                circulation_pump: row.get("circulation_pump"),
                aux_heater_3kw_on: row.get("aux_heater_3kw_on"),
                aux_heater_6kw_on: row.get("aux_heater_6kw_on"),
            }),
            None => Err(AppError::NotFound(format!(
                "Heatpump reading not found for ts={}",
                ts
            ))),
        }
    }

    pub async fn find_latest(&self, device_id: Option<String>) -> Result<HeatpumpReading> {
        let query = if let Some(device_id) = device_id {
            sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE device_id = $1 
                 ORDER BY ts DESC LIMIT 1"
            )
            .bind(device_id)
        } else {
            sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump 
                 ORDER BY ts DESC LIMIT 1"
            )
        };

        let row = query.fetch_optional(&self.pool).await?;

        match row {
            Some(row) => Ok(HeatpumpReading {
                ts: row.get("ts"),
                device_id: row.get("device_id"),
                room: row.get("room"),
                outdoor_temp: row.get("outdoor_temp"),
                supplyline_temp: row.get("supplyline_temp"),
                returnline_temp: row.get("returnline_temp"),
                hotwater_temp: row.get("hotwater_temp"),
                brine_out_temp: row.get("brine_out_temp"),
                brine_in_temp: row.get("brine_in_temp"),
                integral: row.get("integral"),
                flowlinepump_speed: row.get("flowlinepump_speed"),
                brinepump_speed: row.get("brinepump_speed"),
                runtime_compressor: row.get("runtime_compressor"),
                runtime_hotwater: row.get("runtime_hotwater"),
                runtime_3kw: row.get("runtime_3kw"),
                runtime_6kw: row.get("runtime_6kw"),
                brinepump_on: row.get("brinepump_on"),
                compressor_on: row.get("compressor_on"),
                flowlinepump_on: row.get("flowlinepump_on"),
                hotwater_production: row.get("hotwater_production"),
                circulation_pump: row.get("circulation_pump"),
                aux_heater_3kw_on: row.get("aux_heater_3kw_on"),
                aux_heater_6kw_on: row.get("aux_heater_6kw_on"),
            }),
            None => Err(AppError::NotFound("No heatpump readings found".to_string())),
        }
    }
}

