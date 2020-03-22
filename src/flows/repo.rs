use crate::filesystem;
use crate::fzf;
use crate::git;
use crate::structures::fzf::{Opts as FzfOpts, SuggestionType};
use git2::Repository;
use std::error::Error;
use std::fs;
use std::io::Write;
use walkdir::WalkDir;

pub fn browse() -> Result<(), Box<dyn Error>> {
    let repo_path_str = format!("{}/featured", filesystem::tmp_path_str());

    filesystem::remove_dir(&repo_path_str);
    filesystem::create_dir(&repo_path_str);

    match Repository::clone("https://github.com/denisidoro/cheats", &repo_path_str) {
        Ok(r) => r,
        Err(e) => panic!("failed to clone: {}", e),
    };

    let repos = fs::read_to_string(format!("{}/featured_repos.txt", &repo_path_str))
        .expect("Unable to fetch featured repos");

    let opts = FzfOpts {
        column: Some(1),
        ..Default::default()
    };

    let (repo, _) = fzf::call(opts, |stdin| {
        stdin
            .write_all(repos.as_bytes())
            .expect("Unable to prompt featured repos");
        None
    });

    filesystem::remove_dir(&repo_path_str);

    add(repo)
}

pub fn add(uri: String) -> Result<(), Box<dyn Error>> {
    let (actual_uri, user, repo) = git::meta(uri.as_str());

    let cheat_path_str = filesystem::pathbuf_to_string(filesystem::cheat_pathbuf().unwrap());
    let tmp_path_str = filesystem::tmp_path_str();
    let tmp_path_str_with_trailing_slash = format!("{}/", &tmp_path_str);

    filesystem::remove_dir(&tmp_path_str);
    filesystem::create_dir(&tmp_path_str);

    eprintln!("Cloning {} into {}...\n", &actual_uri, &tmp_path_str);

    match Repository::clone(actual_uri.as_str(), &tmp_path_str) {
        Ok(r) => r,
        Err(e) => panic!("failed to clone: {}", e),
    };

    let all_files = WalkDir::new(&tmp_path_str)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_str().unwrap_or("").to_string())
        .filter(|e| e.ends_with(".cheat"))
        .map(|e| e.replace(&tmp_path_str_with_trailing_slash, ""))
        .collect::<Vec<String>>()
        .join("\n");

    let opts = FzfOpts {
        suggestion_type: SuggestionType::MultipleSelections,
        preview: Some(format!("cat '{}/{{}}'", tmp_path_str)),
        header: Some(
            "Select the cheatsheets you want to import with <TAB> then hit <Enter>".to_string(),
        ),
        preview_window: Some("right:30%".to_string()),
        ..Default::default()
    };

    let (files, _) = fzf::call(opts, |stdin| {
        stdin
            .write_all(all_files.as_bytes())
            .expect("Unable to prompt cheats to import");
        None
    });

    for f in files.split('\n') {
        let from = format!("{}/{}", tmp_path_str, f).replace("./", "");
        let to_folder = format!("{}/{}__{}", cheat_path_str, user, repo).replace("./", "");
        let filename = f.replace("./", "").replace("/", "__");
        let to = format!("{}/{}", to_folder, filename);
        fs::create_dir_all(to_folder).unwrap_or(());
        fs::copy(from, to)?;
    }

    filesystem::remove_dir(&tmp_path_str);

    eprintln!("The following .cheat files were imported successfully:\n{}\n\nThey are now located at {}\n\nPlease run navi again to check the results.", files, cheat_path_str);

    Ok(())
}
