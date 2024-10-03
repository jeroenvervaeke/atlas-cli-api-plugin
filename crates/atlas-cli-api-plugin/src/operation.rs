use thiserror::Error;

use crate::path::Path;

#[derive(Debug)]
pub struct Operation {
    pub description: Option<String>,
    pub flags: Vec<OperationFlag>,
    pub operation_id: String,
    pub path: Path<'static>,
    pub tag: String,
    pub verb: String,
}

#[derive(Debug)]
pub struct OperationFlag {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Error)]
pub enum OperationCreationError {
    #[error("[{verb} {path}] Unexpected number of tags, expected 1, got {got}")]
    UnexpectedNumberOfTags {
        got: usize,
        path: String,
        verb: String,
    },
    #[error("[{verb} {path}] OperationId is missing")]
    OperationIdMissing { path: String, verb: String },
    #[error("[{operation_id}] Empty path")]
    EmptyPath { operation_id: String },
    #[error("components are missing from spec")]
    ComponentsAreMissing,
    #[error("parameter not found on spec: {parameter_name}")]
    ParameterNotFound { parameter_name: String },
    #[error("parameter is a reference, expected an item, parameter name: {parameter_name}")]
    ParameterIsAReference { parameter_name: String },
}

const PARAMETER_PREFIX: &str = "#/components/parameters/";

impl Operation {
    pub fn new(
        full_spec: &openapiv3::OpenAPI,
        path: &str,
        verb: &str,
        operation: &openapiv3::Operation,
    ) -> Result<Self, OperationCreationError> {
        let path = path.to_string();
        let verb = verb.to_string();

        if operation.tags.len() != 1 {
            return Err(OperationCreationError::UnexpectedNumberOfTags {
                got: operation.tags.len(),
                path,
                verb,
            });
        }

        let tag = operation.tags[0].trim().to_owned();
        let operation_id = operation
            .operation_id
            .as_ref()
            .ok_or(OperationCreationError::OperationIdMissing {
                path: path.clone(),
                verb: verb.clone(),
            })?
            .trim()
            .to_owned();

        let path = Path::from_str_owned(&path).ok_or(OperationCreationError::EmptyPath {
            operation_id: operation_id.clone(),
        })?;

        let description = operation.description.to_owned();

        let mut flags = vec![];
        for parameter in &operation.parameters {
            let parameter_data = match parameter {
                openapiv3::ReferenceOr::Reference { reference } => {
                    let parameter_name = reference.trim_start_matches(PARAMETER_PREFIX);

                    let parameter = full_spec
                        .components
                        .as_ref()
                        .ok_or(OperationCreationError::ComponentsAreMissing)?
                        .parameters
                        .get(parameter_name)
                        .ok_or(OperationCreationError::ParameterNotFound {
                            parameter_name: reference.clone(),
                        })?;

                    let parameter = parameter.as_item().ok_or(
                        OperationCreationError::ParameterIsAReference {
                            parameter_name: parameter_name.to_owned(),
                        },
                    )?;

                    parameter.parameter_data_ref()
                }
                openapiv3::ReferenceOr::Item(item) => item.parameter_data_ref(),
            };

            flags.push(OperationFlag {
                name: parameter_data.name.trim().to_owned(),
                description: parameter_data
                    .description
                    .as_ref()
                    .map(|d| d.trim().to_owned()),
            });
        }

        Ok(Self {
            description,
            flags,
            operation_id,
            tag,
            path,
            verb,
        })
    }
}
