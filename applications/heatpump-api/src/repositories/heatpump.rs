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
        let limit = params.limit.unwrap_or(100);
        let offset = params.offset.unwrap_or(0);

        // Build query based on which filters are present
        let (query_str, mut query) = if params.device_id.is_some() && params.room.is_some() && params.start_time.is_some() && params.end_time.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE device_id = $1 AND room = $2 AND ts >= $3 AND ts <= $4 
                 ORDER BY ts DESC LIMIT $5 OFFSET $6"
            )
            .bind(params.device_id.as_ref().unwrap())
            .bind(params.room.as_ref().unwrap())
            .bind(params.start_time.as_ref().unwrap())
            .bind(params.end_time.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else if params.device_id.is_some() && params.room.is_some() && params.start_time.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE device_id = $1 AND room = $2 AND ts >= $3 
                 ORDER BY ts DESC LIMIT $4 OFFSET $5"
            )
            .bind(params.device_id.as_ref().unwrap())
            .bind(params.room.as_ref().unwrap())
            .bind(params.start_time.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else if params.device_id.is_some() && params.room.is_some() && params.end_time.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE device_id = $1 AND room = $2 AND ts <= $3 
                 ORDER BY ts DESC LIMIT $4 OFFSET $5"
            )
            .bind(params.device_id.as_ref().unwrap())
            .bind(params.room.as_ref().unwrap())
            .bind(params.end_time.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else if params.device_id.is_some() && params.room.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE device_id = $1 AND room = $2 
                 ORDER BY ts DESC LIMIT $3 OFFSET $4"
            )
            .bind(params.device_id.as_ref().unwrap())
            .bind(params.room.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else if params.device_id.is_some() && params.start_time.is_some() && params.end_time.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE device_id = $1 AND ts >= $2 AND ts <= $3 
                 ORDER BY ts DESC LIMIT $4 OFFSET $5"
            )
            .bind(params.device_id.as_ref().unwrap())
            .bind(params.start_time.as_ref().unwrap())
            .bind(params.end_time.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else if params.device_id.is_some() && params.start_time.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE device_id = $1 AND ts >= $2 
                 ORDER BY ts DESC LIMIT $3 OFFSET $4"
            )
            .bind(params.device_id.as_ref().unwrap())
            .bind(params.start_time.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else if params.device_id.is_some() && params.end_time.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE device_id = $1 AND ts <= $2 
                 ORDER BY ts DESC LIMIT $3 OFFSET $4"
            )
            .bind(params.device_id.as_ref().unwrap())
            .bind(params.end_time.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else if params.device_id.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE device_id = $1 
                 ORDER BY ts DESC LIMIT $2 OFFSET $3"
            )
            .bind(params.device_id.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else if params.room.is_some() && params.start_time.is_some() && params.end_time.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE room = $1 AND ts >= $2 AND ts <= $3 
                 ORDER BY ts DESC LIMIT $4 OFFSET $5"
            )
            .bind(params.room.as_ref().unwrap())
            .bind(params.start_time.as_ref().unwrap())
            .bind(params.end_time.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else if params.room.is_some() && params.start_time.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE room = $1 AND ts >= $2 
                 ORDER BY ts DESC LIMIT $3 OFFSET $4"
            )
            .bind(params.room.as_ref().unwrap())
            .bind(params.start_time.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else if params.room.is_some() && params.end_time.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE room = $1 AND ts <= $2 
                 ORDER BY ts DESC LIMIT $3 OFFSET $4"
            )
            .bind(params.room.as_ref().unwrap())
            .bind(params.end_time.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else if params.room.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE room = $1 
                 ORDER BY ts DESC LIMIT $2 OFFSET $3"
            )
            .bind(params.room.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else if params.start_time.is_some() && params.end_time.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE ts >= $1 AND ts <= $2 
                 ORDER BY ts DESC LIMIT $3 OFFSET $4"
            )
            .bind(params.start_time.as_ref().unwrap())
            .bind(params.end_time.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else if params.start_time.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE ts >= $1 
                 ORDER BY ts DESC LIMIT $2 OFFSET $3"
            )
            .bind(params.start_time.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else if params.end_time.is_some() {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump WHERE ts <= $1 
                 ORDER BY ts DESC LIMIT $2 OFFSET $3"
            )
            .bind(params.end_time.as_ref().unwrap())
            .bind(limit)
            .bind(offset);
            ("", q)
        } else {
            let q = sqlx::query(
                "SELECT ts, device_id, room, outdoor_temp, supplyline_temp, returnline_temp, 
                        hotwater_temp, brine_out_temp, brine_in_temp, integral, 
                        flowlinepump_speed, brinepump_speed, runtime_compressor, runtime_hotwater, 
                        runtime_3kw, runtime_6kw, brinepump_on, compressor_on, flowlinepump_on, 
                        hotwater_production, circulation_pump, aux_heater_3kw_on, aux_heater_6kw_on 
                 FROM heatpump 
                 ORDER BY ts DESC LIMIT $1 OFFSET $2"
            )
            .bind(limit)
            .bind(offset);
            ("", q)
        };

        let rows = query.fetch_all(&self.pool).await?;

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
        let (_, query) = if params.device_id.is_some() && params.room.is_some() && params.start_time.is_some() && params.end_time.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE device_id = $1 AND room = $2 AND ts >= $3 AND ts <= $4")
                .bind(params.device_id.as_ref().unwrap())
                .bind(params.room.as_ref().unwrap())
                .bind(params.start_time.as_ref().unwrap())
                .bind(params.end_time.as_ref().unwrap());
            ("", q)
        } else if params.device_id.is_some() && params.room.is_some() && params.start_time.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE device_id = $1 AND room = $2 AND ts >= $3")
                .bind(params.device_id.as_ref().unwrap())
                .bind(params.room.as_ref().unwrap())
                .bind(params.start_time.as_ref().unwrap());
            ("", q)
        } else if params.device_id.is_some() && params.room.is_some() && params.end_time.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE device_id = $1 AND room = $2 AND ts <= $3")
                .bind(params.device_id.as_ref().unwrap())
                .bind(params.room.as_ref().unwrap())
                .bind(params.end_time.as_ref().unwrap());
            ("", q)
        } else if params.device_id.is_some() && params.room.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE device_id = $1 AND room = $2")
                .bind(params.device_id.as_ref().unwrap())
                .bind(params.room.as_ref().unwrap());
            ("", q)
        } else if params.device_id.is_some() && params.start_time.is_some() && params.end_time.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE device_id = $1 AND ts >= $2 AND ts <= $3")
                .bind(params.device_id.as_ref().unwrap())
                .bind(params.start_time.as_ref().unwrap())
                .bind(params.end_time.as_ref().unwrap());
            ("", q)
        } else if params.device_id.is_some() && params.start_time.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE device_id = $1 AND ts >= $2")
                .bind(params.device_id.as_ref().unwrap())
                .bind(params.start_time.as_ref().unwrap());
            ("", q)
        } else if params.device_id.is_some() && params.end_time.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE device_id = $1 AND ts <= $2")
                .bind(params.device_id.as_ref().unwrap())
                .bind(params.end_time.as_ref().unwrap());
            ("", q)
        } else if params.device_id.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE device_id = $1")
                .bind(params.device_id.as_ref().unwrap());
            ("", q)
        } else if params.room.is_some() && params.start_time.is_some() && params.end_time.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE room = $1 AND ts >= $2 AND ts <= $3")
                .bind(params.room.as_ref().unwrap())
                .bind(params.start_time.as_ref().unwrap())
                .bind(params.end_time.as_ref().unwrap());
            ("", q)
        } else if params.room.is_some() && params.start_time.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE room = $1 AND ts >= $2")
                .bind(params.room.as_ref().unwrap())
                .bind(params.start_time.as_ref().unwrap());
            ("", q)
        } else if params.room.is_some() && params.end_time.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE room = $1 AND ts <= $2")
                .bind(params.room.as_ref().unwrap())
                .bind(params.end_time.as_ref().unwrap());
            ("", q)
        } else if params.room.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE room = $1")
                .bind(params.room.as_ref().unwrap());
            ("", q)
        } else if params.start_time.is_some() && params.end_time.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE ts >= $1 AND ts <= $2")
                .bind(params.start_time.as_ref().unwrap())
                .bind(params.end_time.as_ref().unwrap());
            ("", q)
        } else if params.start_time.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE ts >= $1")
                .bind(params.start_time.as_ref().unwrap());
            ("", q)
        } else if params.end_time.is_some() {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump WHERE ts <= $1")
                .bind(params.end_time.as_ref().unwrap());
            ("", q)
        } else {
            let q = sqlx::query("SELECT COUNT(*) as count FROM heatpump");
            ("", q)
        };

        let row = query.fetch_one(&self.pool).await?;
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
