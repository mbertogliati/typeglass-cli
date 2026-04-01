use crate::ux_model::intent::UserCommandFrom;

pub fn test() {
    let _ = UserCommandFrom { target: crate::ux_model::intent::FromTarget::PublicExports, depth: None };
}
