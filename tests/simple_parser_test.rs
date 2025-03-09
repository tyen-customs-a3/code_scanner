#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::fs;
    use anyhow::Result;
    use tempfile::tempdir;
    
    // Add required imports
    use code_scanner::class::scanner::simple_parser::SimpleParser;
    use code_scanner::class::processor::ClassProcessor;
    use code_scanner::class::types::ClassScanOptions;
    
    #[test]
    fn test_simple_parser() -> Result<()> {
        // Create a temporary test file
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test_class.hpp");
        
        // Write test content to the file
        let class_content = r#"
        class CfgMovesBasic;
        class CfgMovesMaleSdr: CfgMovesBasic {
            class States {
                // Jog speed adjustment: 12km/h
                class AmovPercMstpSlowWrflDnon;
                class AmovPercMrunSlowWrflDf: AmovPercMstpSlowWrflDnon {
                    speed = 0.634570;
                };
                class AmovPercMrunSlowWrflDfl: AmovPercMrunSlowWrflDf {
                    speed = 0.634570;
                };
            };
        };
        "#;
        
        fs::write(&file_path, class_content)?;
        
        // Create a simple parser and parse the file
        let parser = SimpleParser::new(true);
        let classes = parser.parse_file(&file_path)?;
        
        // Verify the results
        assert_eq!(classes.len(), 6, "Should have found 6 classes");
        
        // Check class names
        let class_names: Vec<String> = classes.iter().map(|c| c.name.clone()).collect();
        assert!(class_names.contains(&"CfgMovesBasic".to_string()), "Should contain CfgMovesBasic");
        assert!(class_names.contains(&"CfgMovesMaleSdr".to_string()), "Should contain CfgMovesMaleSdr");
        assert!(class_names.contains(&"States".to_string()), "Should contain States");
        assert!(class_names.contains(&"AmovPercMstpSlowWrflDnon".to_string()), "Should contain AmovPercMstpSlowWrflDnon");
        assert!(class_names.contains(&"AmovPercMrunSlowWrflDf".to_string()), "Should contain AmovPercMrunSlowWrflDf");
        assert!(class_names.contains(&"AmovPercMrunSlowWrflDfl".to_string()), "Should contain AmovPercMrunSlowWrflDfl");
        
        // Check parent classes
        let cfgmoves = classes.iter().find(|c| c.name == "CfgMovesMaleSdr").unwrap();
        assert_eq!(cfgmoves.parent, Some("CfgMovesBasic".to_string()), "CfgMovesMaleSdr should inherit from CfgMovesBasic");
        
        let amov_run = classes.iter().find(|c| c.name == "AmovPercMrunSlowWrflDf").unwrap();
        assert_eq!(amov_run.parent, Some("AmovPercMstpSlowWrflDnon".to_string()), "AmovPercMrunSlowWrflDf should inherit from AmovPercMstpSlowWrflDnon");
        
        let amov_run_fl = classes.iter().find(|c| c.name == "AmovPercMrunSlowWrflDfl").unwrap();
        assert_eq!(amov_run_fl.parent, Some("AmovPercMrunSlowWrflDf".to_string()), "AmovPercMrunSlowWrflDfl should inherit from AmovPercMrunSlowWrflDf");
        
        Ok(())
    }
    
    #[test]
    fn test_class_processor_integration() -> Result<()> {
        
        // Create a temporary test directory
        let temp_dir = tempdir()?;
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&output_dir)?;
        
        // Create a test file
        let class_file = temp_dir.path().join("test_classes.hpp");
        let class_content = r#"
        class Vehicle {
            scope = 2;
            displayName = "Vehicle";
        };
        
        class Tank: Vehicle {
            armor = 1000;
            displayName = "Tank";
            
            class Turret {
                weapons[] = {"cannon"};
            };
        };
        
        class Car: Vehicle {
            armor = 200;
            displayName = "Car";
        };
        "#;
        
        fs::write(&class_file, class_content)?;
        
        // Create a class processor with default options
        let mut processor = ClassProcessor::new(ClassScanOptions::default(), &output_dir);
        
        // Scan the test file
        let scan_result = processor.scan_specific_files(&[class_file])?;
        
        // Verify the results
        assert_eq!(scan_result.classes.len(), 4, "Should have found 4 classes");
        assert_eq!(scan_result.stats.total_classes, 4, "Stats should show 4 classes");
        assert_eq!(scan_result.stats.total_files, 1, "Stats should show 1 file");
        
        // Check class names and inheritance
        let vehicle = scan_result.classes.iter().find(|c| c.name == "Vehicle").unwrap();
        assert_eq!(vehicle.parent, None, "Vehicle should have no parent");
        
        let tank = scan_result.classes.iter().find(|c| c.name == "Tank").unwrap();
        assert_eq!(tank.parent, Some("Vehicle".to_string()), "Tank should inherit from Vehicle");
        
        let car = scan_result.classes.iter().find(|c| c.name == "Car").unwrap();
        assert_eq!(car.parent, Some("Vehicle".to_string()), "Car should inherit from Vehicle");
        
        Ok(())
    }
    
    #[test]
    fn test_config_file_scanning() -> Result<()> {
        // Get path to the test_config.cpp file
        let config_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("test_config.cpp");
        
        // Create a temporary directory for output
        let temp_dir = tempdir()?;
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&output_dir)?;
        
        // Create a class processor with default options
        let mut processor = ClassProcessor::new(ClassScanOptions::default(), &output_dir);
        
        // Scan the test config file
        let scan_result = processor.scan_specific_files(&[config_file_path.clone()])?;
        
        // Verify the basic results
        assert!(scan_result.classes.len() > 0, "Should have found some classes");
        assert_eq!(scan_result.stats.total_files, 1, "Stats should show 1 file");
        
        // Check for specific classes we know should be there
        let class_names: Vec<String> = scan_result.classes.iter().map(|c| c.name.clone()).collect();
        
        // Check for main config sections
        assert!(class_names.contains(&"CfgPatches".to_string()), "Should contain CfgPatches");
        assert!(class_names.contains(&"CfgVehicles".to_string()), "Should contain CfgVehicles");
        assert!(class_names.contains(&"CfgWeapons".to_string()), "Should contain CfgWeapons");
        
        // Check for specific classes
        assert!(class_names.contains(&"bw_gear".to_string()), "Should contain bw_gear");
        assert!(class_names.contains(&"bw_combat_fleck".to_string()), "Should contain bw_combat_fleck");
        assert!(class_names.contains(&"bw_uniform_combat_fleck".to_string()), "Should contain bw_uniform_combat_fleck");
        
        // Check parent class relationships
        let bw_combat_fleck = scan_result.classes.iter().find(|c| c.name == "bw_combat_fleck").unwrap();
        assert_eq!(bw_combat_fleck.parent, Some("I_Soldier_base_F".to_string()), 
                   "bw_combat_fleck should inherit from I_Soldier_base_F");
        
        // Check parent class relationship for a uniform
        let bw_uniform = scan_result.classes.iter().find(|c| c.name == "bw_uniform_combat_fleck").unwrap();
        assert_eq!(bw_uniform.parent, Some("Uniform_Base".to_string()), 
                   "bw_uniform_combat_fleck should inherit from Uniform_Base");
        
        Ok(())
    }
} 