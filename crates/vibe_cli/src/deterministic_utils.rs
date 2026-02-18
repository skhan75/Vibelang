#[derive(Debug, Clone, PartialEq)]
pub enum DeterministicValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    List(Vec<DeterministicValue>),
    Void,
}

impl DeterministicValue {
    pub fn as_int(&self) -> Result<i64, String> {
        match self {
            DeterministicValue::Int(v) => Ok(*v),
            other => Err(format!(
                "expected Int but got `{}`",
                deterministic_value_name(other)
            )),
        }
    }

    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            DeterministicValue::Bool(v) => Ok(*v),
            other => Err(format!(
                "expected Bool but got `{}`",
                deterministic_value_name(other)
            )),
        }
    }

    pub fn as_list(&self) -> Result<&[DeterministicValue], String> {
        match self {
            DeterministicValue::List(v) => Ok(v),
            other => Err(format!(
                "expected List but got `{}`",
                deterministic_value_name(other)
            )),
        }
    }
}

pub fn deterministic_value_name(value: &DeterministicValue) -> &'static str {
    match value {
        DeterministicValue::Int(_) => "Int",
        DeterministicValue::Float(_) => "Float",
        DeterministicValue::Bool(_) => "Bool",
        DeterministicValue::Str(_) => "Str",
        DeterministicValue::List(_) => "List",
        DeterministicValue::Void => "Void",
    }
}

pub fn deterministic_value_to_string(value: &DeterministicValue) -> String {
    match value {
        DeterministicValue::Int(v) => v.to_string(),
        DeterministicValue::Float(v) => v.to_string(),
        DeterministicValue::Bool(v) => v.to_string(),
        DeterministicValue::Str(v) => format!("{v:?}"),
        DeterministicValue::List(values) => {
            let inner = values
                .iter()
                .map(deterministic_value_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{inner}]")
        }
        DeterministicValue::Void => "Void".to_string(),
    }
}

pub fn deterministic_len(value: &DeterministicValue) -> Result<DeterministicValue, String> {
    match value {
        DeterministicValue::List(v) => Ok(DeterministicValue::Int(v.len() as i64)),
        DeterministicValue::Str(v) => Ok(DeterministicValue::Int(v.chars().count() as i64)),
        other => Err(format!(
            "len expects List or Str, got `{}`",
            deterministic_value_name(other)
        )),
    }
}

pub fn deterministic_min(
    a: &DeterministicValue,
    b: &DeterministicValue,
) -> Result<DeterministicValue, String> {
    Ok(DeterministicValue::Int(a.as_int()?.min(b.as_int()?)))
}

pub fn deterministic_max(
    a: &DeterministicValue,
    b: &DeterministicValue,
) -> Result<DeterministicValue, String> {
    Ok(DeterministicValue::Int(a.as_int()?.max(b.as_int()?)))
}

pub fn deterministic_sorted_desc(value: &DeterministicValue) -> Result<DeterministicValue, String> {
    let values = value.as_list()?;
    let mut prev = None;
    for item in values {
        let current = item.as_int()?;
        if let Some(last) = prev {
            if current > last {
                return Ok(DeterministicValue::Bool(false));
            }
        }
        prev = Some(current);
    }
    Ok(DeterministicValue::Bool(true))
}

pub fn deterministic_sort_desc(value: &DeterministicValue) -> Result<DeterministicValue, String> {
    let values = value.as_list()?;
    let mut ints = Vec::with_capacity(values.len());
    for item in values {
        ints.push(item.as_int()?);
    }
    ints.sort_by(|a, b| b.cmp(a));
    Ok(DeterministicValue::List(
        ints.into_iter().map(DeterministicValue::Int).collect(),
    ))
}

pub fn deterministic_take(
    value: &DeterministicValue,
    count: &DeterministicValue,
) -> Result<DeterministicValue, String> {
    let values = value.as_list()?;
    let n = count.as_int()?.max(0) as usize;
    Ok(DeterministicValue::List(
        values.iter().take(n).cloned().collect(),
    ))
}
