use std::collections::HashMap;
use cpp_parser::models::{Property, PropertyValue};
use log::trace;

/// Property processor for processing class properties
#[derive(Debug, Default)]
pub struct PropertyProcessor {}

impl PropertyProcessor {
    /// Create a new property processor
    pub fn new() -> Self {
        Self {}
    }
    
    /// Collect properties from a list of properties without processing them
    pub fn collect_properties_from_list(&self, properties: &[Property]) -> Vec<(String, String)> {
        // Pre-allocate properties vector with estimated capacity
        let mut collected_properties = Vec::with_capacity(properties.len());
        
        // Collect properties without complex processing
        for property in properties {
            let key = &property.name;
            let value = &property.value;
            
            match value {
                PropertyValue::String(s) => {
                    trace!("Collecting string property: {} = {}", key, s);
                    collected_properties.push((key.clone(), s.clone()));
                }
                PropertyValue::Number(n) => {
                    trace!("Collecting number property: {} = {}", key, n);
                    collected_properties.push((key.clone(), n.to_string()));
                }
                PropertyValue::Boolean(b) => {
                    trace!("Collecting boolean property: {} = {}", key, b);
                    collected_properties.push((key.clone(), b.to_string()));
                }
                PropertyValue::Array(_) => {
                    trace!("Collecting array property: {}", key);
                    collected_properties.push((key.clone(), "[array]".to_string()));
                }
                PropertyValue::Reference(ref_name) => {
                    trace!("Collecting reference property: {} = {}", key, ref_name);
                    collected_properties.push((key.clone(), ref_name.clone()));
                }
            }
        }
        
        collected_properties
    }
    
    /// Backward compatibility method for old HashMap<String, Value> interface
    pub fn collect_properties(&self, properties: &HashMap<String, PropertyValue>) -> Vec<(String, String)> {
        // Pre-allocate properties vector with estimated capacity
        let mut collected_properties = Vec::with_capacity(properties.len());
        
        // Collect properties without complex processing
        for (key, value) in properties {
            match value {
                PropertyValue::String(s) => {
                    trace!("Collecting string property: {} = {}", key, s);
                    collected_properties.push((key.clone(), s.clone()));
                }
                PropertyValue::Number(n) => {
                    trace!("Collecting number property: {} = {}", key, n);
                    collected_properties.push((key.clone(), n.to_string()));
                }
                PropertyValue::Boolean(b) => {
                    trace!("Collecting boolean property: {} = {}", key, b);
                    collected_properties.push((key.clone(), b.to_string()));
                }
                PropertyValue::Array(_) => {
                    trace!("Collecting array property: {}", key);
                    collected_properties.push((key.clone(), "[array]".to_string()));
                }
                PropertyValue::Reference(ref_name) => {
                    trace!("Collecting reference property: {} = {}", key, ref_name);
                    collected_properties.push((key.clone(), ref_name.clone()));
                }
            }
        }
        
        collected_properties
    }
    
    /// Process properties from a list of properties
    pub fn process_properties_from_list(&self, properties: &[Property]) -> Vec<(String, String)> {
        // Pre-allocate properties vector with estimated capacity
        let mut processed_properties = Vec::with_capacity(properties.len());
        
        // Process properties
        for property in properties {
            let key = &property.name;
            let value = &property.value;
            
            match value {
                PropertyValue::String(s) => {
                    trace!("Processing string property: {} = {}", key, s);
                    processed_properties.push((key.clone(), s.clone()));
                }
                PropertyValue::Number(n) => {
                    trace!("Processing number property: {} = {}", key, n);
                    processed_properties.push((key.clone(), n.to_string()));
                }
                PropertyValue::Array(arr) => {
                    trace!("Processing array property: {} with {} elements", key, arr.len());
                    // Convert array to string representation
                    let arr_str = arr.iter()
                        .map(|v| self.value_to_string(v))
                        .collect::<Vec<_>>()
                        .join(", ");
                    processed_properties.push((key.clone(), format!("[{}]", arr_str)));
                }
                PropertyValue::Boolean(b) => {
                    trace!("Processing boolean property: {} = {}", key, b);
                    processed_properties.push((key.clone(), b.to_string()));
                }
                PropertyValue::Reference(ref_name) => {
                    trace!("Processing reference property: {} = {}", key, ref_name);
                    processed_properties.push((key.clone(), format!("ref:{}", ref_name)));
                }
            }
        }
        
        processed_properties
    }
    
    /// Backward compatibility method for old HashMap<String, Value> interface
    pub fn process_properties(&self, properties: &HashMap<String, PropertyValue>) -> Vec<(String, String)> {
        // Pre-allocate properties vector with estimated capacity
        let mut processed_properties = Vec::with_capacity(properties.len());
        
        // Process properties
        for (key, value) in properties {
            match value {
                PropertyValue::String(s) => {
                    trace!("Processing string property: {} = {}", key, s);
                    processed_properties.push((key.clone(), s.clone()));
                }
                PropertyValue::Number(n) => {
                    trace!("Processing number property: {} = {}", key, n);
                    processed_properties.push((key.clone(), n.to_string()));
                }
                PropertyValue::Array(arr) => {
                    trace!("Processing array property: {} with {} elements", key, arr.len());
                    // Convert array to string representation
                    let arr_str = arr.iter()
                        .map(|v| self.value_to_string(v))
                        .collect::<Vec<_>>()
                        .join(", ");
                    processed_properties.push((key.clone(), format!("[{}]", arr_str)));
                }
                PropertyValue::Boolean(b) => {
                    trace!("Processing boolean property: {} = {}", key, b);
                    processed_properties.push((key.clone(), b.to_string()));
                }
                PropertyValue::Reference(ref_name) => {
                    trace!("Processing reference property: {} = {}", key, ref_name);
                    processed_properties.push((key.clone(), format!("ref:{}", ref_name)));
                }
            }
        }
        
        processed_properties
    }
    
    /// Convert a value to a string representation
    fn value_to_string(&self, value: &PropertyValue) -> String {
        match value {
            PropertyValue::String(s) => format!("\"{}\"", s),
            PropertyValue::Number(n) => n.to_string(),
            PropertyValue::Array(arr) => {
                let arr_str = arr.iter()
                    .map(|v| self.value_to_string(v))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", arr_str)
            }
            PropertyValue::Boolean(b) => b.to_string(),
            PropertyValue::Reference(ref_name) => format!("ref:{}", ref_name),
        }
    }
    
    /// Get the type of a value as a string
    pub fn get_value_type(&self, value: &PropertyValue) -> String {
        match value {
            PropertyValue::String(_) => "string".to_string(),
            PropertyValue::Number(_) => "number".to_string(),
            PropertyValue::Array(arr) => {
                if arr.is_empty() {
                    "array".to_string()
                } else {
                    format!("array<{}>", self.get_value_type(&arr[0]))
                }
            },
            PropertyValue::Boolean(_) => "boolean".to_string(),
            PropertyValue::Reference(_) => "reference".to_string(),
        }
    }
} 