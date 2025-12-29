mod github_icon;
mod x_icon;

pub(crate) use github_icon::GitHubIcon;
pub(crate) use x_icon::XIcon;

use stylance::import_style;

import_style!(pub(super) sns_icon_style, "sns_icon.module.scss");
