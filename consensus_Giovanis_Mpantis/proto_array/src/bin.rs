use proto_array::fork_choice_test_definition::*;
use std::fs::File;

/// Entry point of the program.
///
/// This function writes several test definitions to YAML files.
fn main() {
    // Write votes test definition to "votes.yaml"
    write_test_def_to_yaml("votes.yaml", get_votes_test_definition());

    // Write no votes test definition to "no_votes.yaml"
    write_test_def_to_yaml("no_votes.yaml", get_no_votes_test_definition());

    // Write FFG case 01 test definition to "ffg_01.yaml"
    write_test_def_to_yaml("ffg_01.yaml", get_ffg_case_01_test_definition());

    // Write FFG case 02 test definition to "ffg_02.yaml"
    write_test_def_to_yaml("ffg_02.yaml", get_ffg_case_02_test_definition());

    // Write execution status 01 test definition to "execution_status_01.yaml"
    write_test_def_to_yaml("execution_status_01.yaml", get_execution_status_test_definition_01());

    // Write execution status 02 test definition to "execution_status_02.yaml"
    write_test_def_to_yaml("execution_status_02.yaml", get_execution_status_test_definition_02());

    // Write execution status 03 test definition to "execution_status_03.yaml"
    write_test_def_to_yaml("execution_status_03.yaml", get_execution_status_test_definition_03());
}

/// Writes a `ForkChoiceTestDefinition` to a YAML file.
///
/// # Arguments
///
/// * `filename` - The name of the YAML file to write to.
/// * `def` - The `ForkChoiceTestDefinition` to write.
fn write_test_def_to_yaml(filename: &str, def: ForkChoiceTestDefinition) {
    let file = File::create(filename).expect("Should be able to open file");
    serde_yaml::to_writer(file, &def).expect("Should be able to write YAML to file");
}
