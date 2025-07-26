/// Compile and transform Solidity to Smart Contract IR.
pub fn compile_solidity(
    input_file: &str,
    base_path: Option<&str>,
    include_paths: &[String],
    solc_ver: Option<&str>,
) -> Result<Vec<ir::SourceUnit>> {
    let parsed_sunits = compile_input_file(input_file, base_path, include_paths, solc_ver)?;
    let normalized_sunits = normalize::normalize_source_units(&parsed_sunits);
    let mut output_sunits = vec![];
    for sunit in normalized_sunits.iter() {
        match transform::transform_source_unit(sunit) {
            Ok(sunit) => output_sunits.push(sunit.clone()),
            Err(err) => return Err(err),
        }
    }
    Ok(output_sunits)
}
