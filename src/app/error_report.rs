use crate::core::error::HniError;

pub fn render_error(error: &HniError) -> String {
    format!("hni: {error}")
}
