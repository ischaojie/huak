use std::{fs, path::Path};

use crate::errors::{HuakError, HuakResult};
use pyproject_toml::{BuildSystem, Project};

use serde_derive::{Deserialize, Serialize};

use super::build_system::BuildSystemBuilder;
use super::project::ProjectBuilder;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Toml {
    pub project: Project,
    #[serde(rename = "build-system")]
    pub build_system: BuildSystem,
}

impl Default for Toml {
    fn default() -> Toml {
        Toml {
            project: ProjectBuilder::default(),
            build_system: BuildSystemBuilder::default(),
        }
    }
}

impl Toml {
    pub(crate) fn from(string: &str) -> Result<Toml, toml_edit::de::Error> {
        toml_edit::de::from_str(string)
    }

    pub(crate) fn open(path: &Path) -> HuakResult<Toml> {
        let toml = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => return Err(HuakError::IOError(e)),
        };

        let toml = match Toml::from(&toml) {
            Ok(t) => t,
            Err(e) => return Err(HuakError::TOMLDeserializationError(e)),
        };

        Ok(toml)
    }

    pub(crate) fn to_string(&self) -> Result<String, toml_edit::ser::Error> {
        toml_edit::ser::to_string_pretty(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = ["click==8.1.3", "black==22.8.0"]

[project.optional-dependencies]
test = ["pytest>=6", "mock"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;

        // toml_edit does not preserve the ordering of the tables
        let expected_output = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = [
    "click==8.1.3",
    "black==22.8.0",
]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[project.optional-dependencies]
test = [
    "pytest>=6",
    "mock",
]

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;

        let toml = Toml::from(string).unwrap();
        let res = toml.to_string().unwrap();
        assert_eq!(expected_output, &res);
    }

    #[test]
    fn deserialize() {
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = ["click==8.1.3", "black==22.8.0"]

[project.optional-dependencies]
test = ["pytest>=6", "mock"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;
        let toml = Toml::from(string).unwrap();

        assert_eq!(toml.project.name, "Test");
        assert_eq!(
            toml.project.authors.unwrap()[0]
                .name
                .as_ref()
                .unwrap()
                .clone(),
            "Chris Pryer"
        );

        assert_eq!(toml.build_system.requires, &["huak-core>=1.0.0"]);
        assert_eq!(
            toml.build_system.build_backend,
            Some(String::from("huak.core.build.api"))
        );

        assert_eq!(toml.project.version, Some(String::from("0.1.0")));
        assert_eq!(toml.project.description, Some(String::from("")));
        assert_eq!(
            toml.project.dependencies,
            Some(vec![
                String::from("click==8.1.3"),
                String::from("black==22.8.0")
            ])
        );
    }

    #[test]
    fn deserialize_array_of_authors() {
        let string = r#"[project]
name = "Test"
version = "0.1.0"
description = ""
dependencies = ["click==8.1.3", "black==22.8.0"]

[[project.authors]]
name = "Chris Pryer"
email = "cnpryer@gmail.com"

[[project.authors]]
name = "Chris Pryer"
email = "test@email.com"

[build-system]
requires = ["huak-core>=1.0.0"]
build-backend = "huak.core.build.api"
"#;

        let toml = Toml::from(string).unwrap();

        assert_eq!(
            "Chris Pryer",
            toml.project
                .authors
                .as_ref()
                .unwrap()
                .get(1)
                .unwrap()
                .name
                .as_ref()
                .unwrap()
        );
        assert_eq!(
            "test@email.com",
            toml.project
                .authors
                .as_ref()
                .unwrap()
                .get(1)
                .unwrap()
                .email
                .as_ref()
                .unwrap()
        );
    }
}
