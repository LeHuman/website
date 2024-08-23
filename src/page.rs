use std::{
    collections::{HashMap, HashSet},
    ops::{Add, AddAssign},
    path::PathBuf,
};

use slug::slugify;

use reposcrape::{
    date::{Epoch, EpochType},
    reposcrape::{Project, Repo},
};

const STR_META: &str = "+++\n";

const STR_DIR_ZOLA: &str = "zola/content/";
const STR_DIR_LATEST: &str = "latest/";
const STR_DIR_PROJECTS: &str = "projects/";
const STR_DIR_REPOS: &str = "repos/";

const STR_FILE_LATEST: &str = "__latest_listing_{}.md";
const STR_FILE_PROJECT: &str = "__project_listing_{}.md";
const STR_FILE_PROJECT_REPO: &str = "__repo_{}.md";
const STR_FILE_PROJECT_MAIN_REPO: &str = "_index.md";
const STR_FILE_REPO: &str = "__{}.md";

type MetaMap = HashMap<&'static str, String>;

#[derive(Clone, Default, Debug)]
pub struct Paths {
    pub files: HashMap<PathBuf, String>,
    pub directories: HashSet<PathBuf>,
}

impl AddAssign for Paths {
    fn add_assign(&mut self, rhs: Self) {
        self.files.extend(rhs.files);
        self.directories.extend(rhs.directories);
    }
}

impl Add for Paths {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result.files.extend(rhs.files);
        result.directories.extend(rhs.directories);
        result
    }
}

fn build_page(map: MetaMap, extra: Option<MetaMap>) -> String {
    let mut result = String::from(STR_META);
    for (k, v) in map.iter() {
        if *k == "date" {
            result += &format!("date = {}\n", v);
        } else {
            result += &format!("{} = \"{}\"\n", k, v);
        }
    }

    if let Some(extra) = extra {
        if !extra.is_empty() {
            result += "[extra]\n";
            for (k, v) in extra.iter() {
                result += &format!("{} = \"{}\"\n", k, v);
            }
        }
    }

    result += STR_META;
    result
}

// fn rfc3339_date(epoch: EpochType) -> Option<String> {
//     let datetime: chrono::DateTime<chrono::Utc> = Epoch::to_rfc3339(epoch)?.parse().ok()?;
//     Some(datetime.date_naive().to_string())
// }

fn epoch_to_date(epoch: EpochType) -> String {
    Epoch::to_rfc3339(epoch).unwrap_or(String::from("1970-1-1"))
}

pub fn get_project_epoch(project: &Project) -> EpochType {
    let mut last_update = match &project.repo_main {
        Some(main) => main.last_update,
        None => 0,
    };

    for repo in &project.repo_sub {
        if repo.last_update > last_update {
            last_update = repo.last_update;
        }
    }
    last_update
}

fn build_latest(title: String, update: String, path: String) -> String {
    let mut metadata: MetaMap = HashMap::new();
    metadata.insert("title", title);
    metadata.insert("date", update); // Use "updated" instead?
    metadata.insert("path", String::from("./") + &path);

    build_page(metadata, None)
}

// #[test]
// fn fds() {
//     let mut dir: PathBuf = [STR_DIR_ZOLA, STR_DIR_PROJECTS].iter().collect();
//     dir = dir.canonicalize().unwrap();
//     dir.push("pmpi");
//     println!("{:?}", dir);
// }

pub fn latest_project(project: &Project) -> Paths {
    let mut result = Paths::default();
    let title = project.name.to_owned();
    let page = build_latest(
        title.to_owned(),
        epoch_to_date(get_project_epoch(project)),
        STR_DIR_PROJECTS.to_string() + &slugify(&title),
    );

    let mut dir: PathBuf = [STR_DIR_ZOLA, STR_DIR_LATEST].iter().collect();
    dir = dir.canonicalize().unwrap();
    result.directories.insert(dir.to_owned());

    dir.push(STR_FILE_LATEST.replace("{}", &title));
    result.files.insert(dir, page);

    result
}

pub fn latest_repo(repo: &Repo) -> Paths {
    let mut result = Paths::default();
    let mut title = repo.name.to_owned();
    if let Some(details) = &repo.details {
        if let Some(set_title) = &details.title {
            title = set_title.to_owned();
        }
    }
    let page = build_latest(
        title.to_owned(),
        epoch_to_date(repo.last_update),
        STR_DIR_REPOS.to_string() + &slugify(&title),
    );

    let mut dir: PathBuf = [STR_DIR_ZOLA, STR_DIR_LATEST].iter().collect();
    dir = dir.canonicalize().unwrap();
    result.directories.insert(dir.to_owned());

    dir.push(STR_FILE_LATEST.replace("{}", &title));
    result.files.insert(dir, page);

    result
}

fn get_repo_page(repo: &Repo) -> (String, String) {
    let mut metadata: MetaMap = HashMap::new();
    let mut extra: MetaMap = HashMap::new();

    let mut title = repo.name.to_owned();
    let mut description = String::from("No Description");

    if let Some(details) = &repo.details {
        if let Some(set_title) = &details.title {
            title = set_title.to_owned();
        }
        if let Some(desc) = &details.description {
            description = desc.to_owned();
        }
    }

    metadata.insert("title", title.to_owned());
    metadata.insert("description", description.to_owned());
    metadata.insert("date", epoch_to_date(repo.last_update));
    if let Some(details) = &repo.details {
        details
            .logo
            .as_ref()
            .and_then(|logo| extra.insert("logo", logo.to_owned()));
        details
            .highlight
            .as_ref()
            .and_then(|h| extra.insert("highlight", h.to_owned()));
    }

    let mut page = build_page(metadata, Some(extra));

    page += &description;
    page += "\n";

    (title, page)
}

pub fn project(project: &Project) -> Paths {
    let mut result = Paths::default();
    let mut extra: MetaMap = HashMap::new();
    let title = project.name.to_owned();
    let slug = slugify(&title);

    let mut metadata: MetaMap = HashMap::new();
    metadata.insert("title", slug.to_owned());
    metadata.insert("date", epoch_to_date(get_project_epoch(project)));
    metadata.insert("template", String::from("404.html"));
    let page = build_page(metadata, None);

    let mut metadata: MetaMap = HashMap::new();
    metadata.insert("title", title.to_owned());
    metadata.insert("sort_by", "title".to_string());
    metadata.insert("template", "project.html".to_string());
    let _ = project.repo_main.as_ref().and_then(|r| {
        r.details.as_ref().and_then(|d| {
            d.logo
                .as_ref()
                .and_then(|l| extra.insert("logo", l.to_owned()))
        })
    });
    let mut index_page = build_page(metadata, Some(extra));

    let mut dir: PathBuf = [STR_DIR_ZOLA, STR_DIR_PROJECTS].iter().collect();
    dir = dir.canonicalize().unwrap();
    result.directories.insert(dir.to_owned());

    dir.push(STR_FILE_PROJECT.replace("{}", &title));
    result.files.insert(dir.to_owned(), page);
    dir.pop();
    dir.push(slug);
    result.directories.insert(dir.to_owned());

    if let Some(repo) = &project.repo_main {
        if let Some(details) = &repo.details {
            index_page += &details
                .description
                .clone()
                .and_then(|s| Some(s.trim().to_string() + "\n"))
                .unwrap_or(String::default());
        }
        dir.push(STR_FILE_PROJECT_MAIN_REPO);
        result.files.insert(dir.to_owned(), index_page);
        dir.pop();
    }

    for repo in &project.repo_sub {
        let (title, page) = get_repo_page(repo);
        dir.push(STR_FILE_PROJECT_REPO.replace("{}", &title));
        result.files.insert(dir.to_owned(), page);
        dir.pop();
    }

    result
}

pub fn repo(repo: &Repo) -> Paths {
    let (title, page) = get_repo_page(repo);

    let mut result = Paths::default();
    let mut dir: PathBuf = [STR_DIR_ZOLA, STR_DIR_REPOS].iter().collect();
    dir = dir.canonicalize().unwrap();
    result.directories.insert(dir.to_owned());

    dir.push(STR_FILE_REPO.replace("{}", &title));
    result.files.insert(dir, page);

    result
}
