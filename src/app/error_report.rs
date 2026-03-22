use crate::core::error::HniError;

#[must_use = "the error string must be used"]
pub fn render_error(error: &HniError) -> String {
    format!("hni: {error}")
}
