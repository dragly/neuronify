pub mod components;
pub mod spawn;
pub mod step;

#[cfg(test)]
mod tests;

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct LegacySimulation {
    pub nodes: Vec<LegacyNode>,
    pub edges: Vec<LegacyEdge>,
    pub file_format_version: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct LegacyNode {
    pub filename: String,
    pub x: f64,
    pub y: f64,
    pub engine: LegacyEngine,
    pub inhibitory: bool,
    pub label: String,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub text: Option<String>,
    pub maximum_value: Option<f64>,
    pub minimum_value: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct LegacyEngine {
    pub capacitance: Option<f64>,
    pub resting_potential: Option<f64>,
    pub initial_potential: Option<f64>,
    pub threshold: Option<f64>,
    pub voltage: Option<f64>,
    pub resistance: Option<f64>,
    pub refractory_period: Option<f64>,
    pub voltage_clamped: Option<bool>,
    pub minimum_voltage: Option<f64>,
    pub maximum_voltage: Option<f64>,
    pub fire_output: Option<f64>,
    pub current_output: Option<f64>,
    pub inhibitory: Option<bool>,
    pub adaptation: Option<f64>,
    pub time_constant: Option<f64>,
    pub conductance: Option<f64>,
    // Synapse properties
    pub tau: Option<f64>,
    pub maximum_current: Option<f64>,
    pub delay: Option<f64>,
    pub alpha_function: Option<bool>,
    pub exponential: Option<f64>,
    pub linear: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct LegacyEdge {
    pub filename: String,
    pub from: usize,
    pub to: usize,
    pub engine: LegacyEngine,
}

pub fn parse_legacy_nfy(json_str: &str) -> Result<LegacySimulation, String> {
    let root: Value = serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {e}"))?;

    let file_format_version = root.get("fileFormatVersion").and_then(|v| v.as_u64()).map(|v| v as u32);
    let is_v2 = file_format_version.map_or(false, |v| v <= 2);

    let nodes = parse_nodes(&root, is_v2)?;
    let edges = parse_edges(&root, is_v2)?;

    Ok(LegacySimulation {
        nodes,
        edges,
        file_format_version,
    })
}

fn parse_nodes(root: &Value, is_v2: bool) -> Result<Vec<LegacyNode>, String> {
    let nodes_array = root.get("nodes").and_then(|v| v.as_array()).ok_or("Missing 'nodes' array")?;
    let mut result = Vec::new();

    for node_val in nodes_array {
        let filename = if is_v2 {
            node_val.get("fileName").or_else(|| node_val.get("filename"))
        } else {
            node_val.get("filename").or_else(|| node_val.get("fileName"))
        }
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

        let (props, engine_val) = if is_v2 {
            // v2: properties at node level, engine is a direct sub-object
            (node_val, node_val.get("engine"))
        } else {
            // v3/v4: properties inside savedProperties
            let sp = node_val.get("savedProperties").unwrap_or(node_val);
            (sp, sp.get("engine"))
        };

        let engine = parse_engine(engine_val);

        let x = get_f64(props, "x").or_else(|| get_f64(node_val, "x")).unwrap_or(0.0);
        let y = get_f64(props, "y").or_else(|| get_f64(node_val, "y")).unwrap_or(0.0);
        let inhibitory = props
            .get("inhibitory")
            .and_then(|v| v.as_bool())
            .or_else(|| node_val.get("inhibitory").and_then(|v| v.as_bool()))
            .unwrap_or(false);
        let label = props
            .get("label")
            .and_then(|v| v.as_str())
            .or_else(|| node_val.get("label").and_then(|v| v.as_str()))
            .unwrap_or("")
            .to_string();
        let text = props.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());
        let width = get_f64(props, "width");
        let height = get_f64(props, "height");
        let maximum_value = get_f64(props, "maximumValue");
        let minimum_value = get_f64(props, "minimumValue");

        result.push(LegacyNode {
            filename,
            x,
            y,
            engine,
            inhibitory,
            label,
            width,
            height,
            text,
            maximum_value,
            minimum_value,
        });
    }

    Ok(result)
}

fn parse_edges(root: &Value, is_v2: bool) -> Result<Vec<LegacyEdge>, String> {
    let edges_array = root.get("edges").and_then(|v| v.as_array()).ok_or("Missing 'edges' array")?;
    let mut result = Vec::new();

    for edge_val in edges_array {
        let from = edge_val.get("from").and_then(|v| v.as_u64()).ok_or("Edge missing 'from'")? as usize;
        let to = edge_val.get("to").and_then(|v| v.as_u64()).ok_or("Edge missing 'to'")? as usize;

        let (filename, engine_val) = if is_v2 {
            let fname = edge_val
                .get("fileName")
                .or_else(|| edge_val.get("filename"))
                .and_then(|v| v.as_str())
                .unwrap_or("Edge.qml")
                .to_string();
            (fname, edge_val.get("engine"))
        } else {
            let sp = edge_val.get("savedProperties");
            let fname = edge_val
                .get("filename")
                .and_then(|v| v.as_str())
                .or_else(|| sp.and_then(|s| s.get("filename")).and_then(|v| v.as_str()))
                .unwrap_or("Edge.qml")
                .to_string();
            let eng = sp.and_then(|s| s.get("engine")).or_else(|| edge_val.get("engine"));
            (fname, eng)
        };

        let engine = parse_engine(engine_val);

        result.push(LegacyEdge {
            filename,
            from,
            to,
            engine,
        });
    }

    Ok(result)
}

fn parse_engine(engine_val: Option<&Value>) -> LegacyEngine {
    let Some(e) = engine_val else {
        return LegacyEngine::default();
    };

    LegacyEngine {
        capacitance: get_f64(e, "capacitance"),
        resting_potential: get_f64(e, "restingPotential"),
        initial_potential: get_f64(e, "initialPotential"),
        threshold: get_f64(e, "threshold"),
        voltage: get_f64(e, "voltage"),
        resistance: get_f64(e, "resistance"),
        refractory_period: get_f64(e, "refractoryPeriod"),
        voltage_clamped: e.get("voltageClamped").and_then(|v| v.as_bool()),
        minimum_voltage: get_f64(e, "minimumVoltage"),
        maximum_voltage: get_f64(e, "maximumVoltage"),
        fire_output: get_f64(e, "fireOutput"),
        current_output: get_f64(e, "currentOutput"),
        inhibitory: e.get("inhibitory").and_then(|v| v.as_bool()),
        adaptation: get_f64(e, "adaptation"),
        time_constant: get_f64(e, "timeConstant"),
        conductance: get_f64(e, "conductance"),
        tau: get_f64(e, "tau"),
        maximum_current: get_f64(e, "maximumCurrent"),
        delay: get_f64(e, "delay"),
        alpha_function: e.get("alphaFunction").and_then(|v| v.as_bool()),
        exponential: get_f64(e, "exponential"),
        linear: get_f64(e, "linear"),
    }
}

fn get_f64(val: &Value, key: &str) -> Option<f64> {
    val.get(key).and_then(|v| v.as_f64())
}
