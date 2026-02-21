use crate::{error::CJDicError, service::AppService};

pub fn list_entries_json(service: &AppService) -> Result<String, CJDicError> {
    let entries = service.list_entries()?;
    Ok(serde_json::to_string(&entries)?)
}
