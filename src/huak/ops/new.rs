use git2::Repository;

use crate::{
    errors::{HuakError, HuakResult},
    project::Project,
};

/// TODO: Dow we need to mutate here?
pub fn create_project(project: &mut Project) -> HuakResult<()> {
    project.init_project_file()?;
    project.bootstrap()?;
    project.project_file.serialize()
}

/// Initializes VCS (currently git) in the project
pub fn init_vcs(project: &Project) -> HuakResult<()> {
    if let Err(e) = Repository::init(project.root()) {
        return Err(HuakError::GitError(e));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    use crate::{config::pyproject::toml::Toml, project::ProjectType};

    // TODO
    #[test]
    fn creates_project() {
        let directory = tempdir().unwrap().into_path();
        // Default should be Library
        let mut project = Project::new(directory, ProjectType::default());

        let toml_path = project.root().join("pyproject.toml");
        let had_toml = toml_path.exists();

        create_project(&mut project).unwrap();

        assert!(!had_toml);
        assert!(toml_path.exists());
    }

    #[test]
    fn create_app_project() {
        let directory = tempdir().unwrap().into_path().join("project");
        let mut project = Project::new(directory, ProjectType::Application);
        let toml_path = project.root().join("pyproject.toml");

        create_project(&mut project).unwrap();
        let toml = Toml::open(&toml_path).unwrap();
        let main_file_filepath =
            project.root().join("src").join("project").join("main.py");
        let main_file = fs::read_to_string(&main_file_filepath).unwrap();
        let expected_main_file = "\
def main():
    print(\"Hello, World!\")


if __name__ == \"__main__\":
    main()
";

        assert!(toml.project.scripts.is_some());
        assert_eq!(
            toml.project.scripts.unwrap()[&toml.project.name],
            format!("{}.main:main", toml.project.name)
        );
        assert_eq!(main_file, expected_main_file);
    }

    #[test]
    fn create_lib_project() {
        let directory = tempdir().unwrap().into_path().join("project");
        let mut project = Project::new(directory, ProjectType::Library);
        let toml_path = project.root().join("pyproject.toml");

        create_project(&mut project).unwrap();
        let toml = Toml::open(&toml_path).unwrap();
        let test_file_filepath =
            project.root().join("tests").join("test_version.py");
        let test_file = fs::read_to_string(&test_file_filepath).unwrap();
        let expected_test_file = format!(
            r#"from {} import __version__


def test_version():
    __version__
"#,
            project.project_file.project_name().unwrap()
        );
        let init_file_filepath = project
            .root()
            .join("src")
            .join("project")
            .join("__init__.py");
        let init_file = fs::read_to_string(&init_file_filepath).unwrap();
        let expected_init_file = "__version__ = \"0.0.1\"
";

        assert!(toml.project.scripts.is_none());
        assert_eq!(test_file, expected_test_file);
        assert_eq!(init_file, expected_init_file);
    }

    #[test]
    fn initialize_git() {
        let directory = tempdir().unwrap().into_path().join("project");
        let project = Project::new(directory.clone(), ProjectType::Application);
        super::init_vcs(&project).unwrap();
        assert!(directory.join(".git").is_dir());
    }
}
