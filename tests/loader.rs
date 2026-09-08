use dotenv::{find_dotenv, load_from_path};

#[test]
fn finds_nearest_dotenv_in_ancestors() {
    let root = tempfile::tempdir().unwrap();
    let child = root.path().join("a/b");
    std::fs::create_dir_all(&child).unwrap();
    std::fs::write(root.path().join(".env"), "ROOT=yes\n").unwrap();
    assert_eq!(find_dotenv(&child), Some(root.path().join(".env")));
    assert_eq!(
        load_from_path(root.path().join(".env")).unwrap()["ROOT"],
        "yes"
    );
}
