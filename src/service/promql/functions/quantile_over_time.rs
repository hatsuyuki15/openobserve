// Copyright 2025 OpenObserve Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use datafusion::error::{DataFusionError, Result};

use crate::service::promql::{
    aggregations::prepare_vector,
    common::quantile,
    value::{InstantValue, Sample, Value},
};

/// https://prometheus.io/docs/prometheus/latest/querying/functions/#quantile_over_time
pub(crate) fn quantile_over_time(timestamp: i64, phi_quantile: f64, data: Value) -> Result<Value> {
    eval(data, phi_quantile, timestamp, false)
}

pub(crate) fn eval(
    data: Value,
    phi_quantile: f64,
    timestamp: i64,
    _keep_name_label: bool,
) -> Result<Value> {
    let data = match data {
        Value::Matrix(v) => v,
        Value::None => return Ok(Value::None),
        v => {
            return Err(DataFusionError::Plan(format!(
                "quantile_over_time: matrix argument expected, got {:?}",
                v.get_type()
            )));
        }
    };

    if data.is_empty() {
        return prepare_vector(timestamp, f64::NAN);
    }

    let mut rate_values = Vec::with_capacity(data.len());
    for mut metric in data {
        let labels = std::mem::take(&mut metric.labels);
        // if !keep_name_label {
        //     labels = metric.labels.without_metric_name();
        // };

        let eval_ts = metric.time_window.as_ref().unwrap().eval_ts;
        let input: Vec<f64> = metric.samples.iter().map(|x| x.value).collect();
        if let Some(value) = quantile(&input, phi_quantile) {
            rate_values.push(InstantValue {
                labels,
                sample: Sample::new(eval_ts, value),
            });
        }
    }
    Ok(Value::Vector(rate_values))
}
