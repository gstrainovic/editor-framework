use std::path::PathBuf;

#[test]
fn test_plugins_dir_created() {
    let tmp = tempfile::tempdir().unwrap();
    let pm = ef_pm::PluginManager::new(tmp.path().to_path_buf());
    pm.init().unwrap();
    assert!(tmp.path().join("plugins").exists());
}

#[test]
fn test_list_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let pm = ef_pm::PluginManager::new(tmp.path().to_path_buf());
    pm.init().unwrap();
    let list = pm.list().unwrap();
    assert!(list.is_empty());
}
