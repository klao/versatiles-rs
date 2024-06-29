use super::VDLPipeline;
use anyhow::{anyhow, ensure, Result};
use std::{collections::HashMap, fmt::Debug, str::FromStr};

#[derive(Clone, Debug, PartialEq)]
pub struct VDLNode {
	pub name: String,
	pub properties: HashMap<String, Vec<String>>,
	pub children: Vec<VDLPipeline>,
}

impl VDLNode {
	#[allow(dead_code)]
	fn get_property_vec(&self, field: &str, min_size: usize) -> Result<&Vec<String>> {
		self
			.properties
			.get(field)
			.ok_or_else(|| anyhow!("field '{field}' not found"))
			.and_then(|list| {
				ensure!(
					list.len() >= min_size,
					"field '{field}' must have at least {min_size} entries"
				);
				Ok(list)
			})
	}

	fn get_property0(&self, field: &str) -> Result<Option<&String>> {
		self.properties.get(field).map_or(Ok(None), |list| {
			ensure!(
				list.len() == 1,
				"field '{field}' must have exactly one entry"
			);
			Ok(list.get(0))
		})
	}

	fn get_property1(&self, field: &str) -> Result<&String> {
		self
			.get_property0(field)?
			.ok_or_else(|| anyhow!("field '{field}' does not exist"))
	}

	pub fn get_property_string0(&self, field: &str) -> Result<Option<String>> {
		Ok(self.get_property0(field)?.map(|v| v.to_string()))
	}

	pub fn get_property_string1(&self, field: &str) -> Result<String> {
		self.get_property1(field).map(|v| v.to_string())
	}

	pub fn get_property_bool(&self, field: &str) -> Result<bool> {
		Ok(self.get_property0(field)?.map_or(false, |v| {
			matches!(
				v.trim().to_lowercase().as_str(),
				"1" | "true" | "yes" | "ok"
			)
		}))
	}

	pub fn get_property_number0<T>(&self, field: &str) -> Result<Option<T>>
	where
		T: FromStr,
		<T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
	{
		self
			.get_property0(field)?
			.map_or(Ok(None), |v| v.parse::<T>().map(Some).map_err(Into::into))
	}
}

impl From<&str> for VDLNode {
	fn from(name: &str) -> Self {
		VDLNode {
			name: name.to_string(),
			properties: HashMap::new(),
			children: vec![],
		}
	}
}

fn make_properties(input: Vec<(&str, &str)>) -> HashMap<String, Vec<String>> {
	input
		.iter()
		.map(|(k, v)| (k.to_string(), vec![v.to_string()]))
		.collect()
}

impl From<(&str, (&str, &str))> for VDLNode {
	fn from(input: (&str, (&str, &str))) -> Self {
		VDLNode {
			name: input.0.to_string(),
			properties: make_properties(vec![input.1]),
			children: vec![],
		}
	}
}

impl From<(&str, Vec<(&str, &str)>)> for VDLNode {
	fn from(input: (&str, Vec<(&str, &str)>)) -> Self {
		VDLNode {
			name: input.0.to_string(),
			properties: make_properties(input.1),
			children: vec![],
		}
	}
}

impl From<(&str, Vec<(&str, &str)>, VDLPipeline)> for VDLNode {
	fn from(input: (&str, Vec<(&str, &str)>, VDLPipeline)) -> Self {
		VDLNode {
			name: input.0.to_string(),
			properties: make_properties(input.1),
			children: vec![input.2],
		}
	}
}

impl From<(&str, Vec<(&str, &str)>, Vec<VDLPipeline>)> for VDLNode {
	fn from(input: (&str, Vec<(&str, &str)>, Vec<VDLPipeline>)) -> Self {
		VDLNode {
			name: input.0.to_string(),
			properties: make_properties(input.1),
			children: input.2,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn make_properties(input: &[(&str, &str)]) -> HashMap<String, Vec<String>> {
		input
			.iter()
			.map(|(k, v)| (k.to_string(), vec![v.to_string()]))
			.collect()
	}

	#[test]
	fn test_vdlnode_get_property() -> Result<()> {
		let node = VDLNode {
			name: "node".to_string(),
			properties: make_properties(&[("key1", "value1"), ("key2", "value2")]),
			children: vec![],
		};
		assert_eq!(
			node.get_property_vec("key1", 0)?,
			&vec!["value1".to_string()]
		);
		assert_eq!(
			node.get_property_vec("key2", 0)?,
			&vec!["value2".to_string()]
		);
		assert!(node.get_property_vec("key3", 0)?.len() == 0);
		Ok(())
	}
}
