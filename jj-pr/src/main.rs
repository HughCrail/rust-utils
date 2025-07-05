use jj_lib::{
    config::StackedConfig,
    local_working_copy::{LocalWorkingCopy, LocalWorkingCopyFactory},
    repo::StoreFactories,
    settings::UserSettings,
    workspace::{DefaultWorkspaceLoaderFactory, WorkingCopyFactories, WorkspaceLoaderFactory},
};
use std::{env, path::Path};

fn main() {
    let repo = (DefaultWorkspaceLoaderFactory {})
        .create(find_workspace_dir(&env::current_dir().unwrap()))
        .unwrap()
        .load(
            &UserSettings::from_config(StackedConfig::with_defaults()).unwrap(),
            &StoreFactories::default(),
            &default_working_copy_factories(),
        )
        .unwrap()
        .repo_loader()
        .load_at_head()
        .unwrap();

    for (name, _) in repo.view().bookmarks() {
        println!("bookmark: {name:?}");
    }
}

pub fn default_working_copy_factories() -> WorkingCopyFactories {
    let mut factories = WorkingCopyFactories::new();
    factories.insert(
        LocalWorkingCopy::name().to_owned(),
        Box::new(LocalWorkingCopyFactory {}),
    );
    factories
}

fn find_workspace_dir(cwd: &Path) -> &Path {
    cwd.ancestors()
        .find(|path| path.join(".jj").is_dir())
        .unwrap_or(cwd)
}
