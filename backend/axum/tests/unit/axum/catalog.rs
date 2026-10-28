use super::*;
use crate::test_support::fake_launcher;

#[test]
fn an_item_is_one_of_the_system_or_a_cache_of_an_application() {
    assert!(is_item("old-kernels"));
    assert!(is_item("cache:nas-file-manager:previews"));
    assert_eq!(
        cache_of("cache:nas-file-manager:search-index"),
        Some(("nas-file-manager", "search-index"))
    );
    for wrong in [
        "rm -rf",
        "cache:",
        "cache:a",
        "cache:a:b:c",
        "cache:../x:y",
        "/var/log",
    ] {
        assert!(!is_item(wrong), "{wrong}");
    }
    assert_eq!(cache_item("a", "b"), "cache:a:b");
}

#[test]
fn the_first_choice_leaves_out_every_item_with_a_warning() {
    let chosen = defaults();
    assert!(chosen.contains(&"temporary".to_owned()));
    for risky in ["apt-orphans", "apt-configs", "old-kernels"] {
        assert!(!chosen.contains(&risky.to_owned()), "{risky}");
    }
}

#[test]
fn a_list_of_items_names_known_ones_once() {
    valid(&["journal".to_owned(), "temporary".to_owned()]).expect("valid");
    assert!(valid(&["journal".to_owned(), "journal".to_owned()]).is_err());
    assert!(valid(&["everything".to_owned()]).is_err());
    assert!(valid(&vec!["journal".to_owned(); 201]).is_err());
}

#[tokio::test]
async fn the_caches_the_launcher_lists_join_the_items() {
    let fake = fake_launcher().await;
    let listed = entries(&fake.launcher()).await;
    assert_eq!(listed.len(), SYSTEM.len() + 2);
    let cache = &listed[SYSTEM.len()];
    assert_eq!(cache.id, "cache:nas-file-manager:previews");
    assert_eq!(cache.group, "caches");
    assert!(!cache.risky);
    assert_eq!(cache.bytes, Some(1000));
    let trash = &listed[SYSTEM.len() + 1];
    assert_eq!(trash.id, "cache:nas-file-manager:trash");
    assert_eq!((trash.group.as_str(), trash.risky), ("trash", true));
    assert_eq!(entries(&Launcher::new(None)).await.len(), SYSTEM.len());
}
