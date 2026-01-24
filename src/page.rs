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

type PageData = String;
type ColorMap = HashMap<String, String>;
type MetaMap = HashMap<&'static str, String>;

#[derive(Clone, Default, Debug)]
pub struct Page {
    pub path: PathBuf,
    pub data: PageData,
}

#[derive(Clone, Default, Debug)]
pub struct PageCollection {
    pub files: HashMap<PathBuf, PageData>,
    pub directories: HashSet<PathBuf>,
}

impl PageCollection {
    pub fn insert(&mut self, mut page: Page) {
        self.files
            .insert(page.path.to_owned(), page.data.to_owned());
        page.path.pop();
        self.directories.insert(page.path);
    }
}

impl AddAssign for PageCollection {
    fn add_assign(&mut self, rhs: Self) {
        self.files.extend(rhs.files);
        self.directories.extend(rhs.directories);
    }
}

impl Add for PageCollection {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result.files.extend(rhs.files);
        result.directories.extend(rhs.directories);
        result
    }
}

fn build_page(map: MetaMap, extra: Option<MetaMap>, ignore: Option<HashSet<String>>) -> String {
    let mut result = String::from(STR_META);
    for (k, v) in map.iter() {
        if ignore.as_ref().is_some_and(|x| x.contains(k.to_owned())) {
            result += &format!("{} = {}\n", k, v);
        } else {
            result += &format!("{} = \"{}\"\n", k, v);
        }
    }

    if let Some(extra) = extra
        && !extra.is_empty() {
            result += "[extra]\n";
            for (k, v) in extra.iter() {
                if ignore.as_ref().is_some_and(|x| x.contains(k.to_owned())) {
                    result += &format!("{} = {}\n", k, v);
                } else {
                    result += &format!("{} = \"{}\"\n", k, v);
                }
            }
        }

    result += STR_META;
    result
}


/// Convert epoch time to a rfc3339 string
/// 
/// # Arguments
/// 
/// - `epoch` (`EpochType`) - Epoch time
/// 
/// # Returns
/// 
/// - `String` - The Epoch as a string. Defaults to 1970-1-1 if invalid
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

    build_page(
        metadata,
        None,
        Some(HashSet::from(["date", "keywords"].map(String::from))),
    )
}

pub fn latest_project(project: &Project) -> PageCollection {
    let mut result = PageCollection::default();
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

fn get_language_string(
    lang_color_map: &HashMap<String, String>,
    languages: &Vec<String>,
) -> String {
    let mut entries = Vec::new();

    for lang in languages {
        match lang_color_map.get(lang) {
            Some(color) => {
                entries.push(format!("{{ name = \"{}\", color = \"{}\" }}", lang, color));
            }
            None => match lang_color_map.get(&lang.to_lowercase()) {
                Some(color) => {
                    entries.push(format!("{{ name = \"{}\", color = \"{}\" }}", lang, color));
                }
                None => {
                    entries.push(format!("{{ name = \"{}\", color = \"#cfcfcf\" }}", lang));
                }
            },
        }
    }

    format!("[{}]", entries.join(","))
}

fn get_repo_strings(lang_colors: &ColorMap, repo: &Repo, latest: bool) -> (String, String) {
    let mut metadata: MetaMap = HashMap::new();
    let mut extra: MetaMap = HashMap::new();

    let mut title = repo.name.to_owned();
    let mut description = String::from("No Description");
    let mut why: Option<String> = None;

    if let Some(details) = &repo.details {
        if let Some(set_title) = &details.title {
            title = set_title.to_owned();
        }
        if let Some(desc) = &details.description {
            description = desc.to_owned();
        }
        if let Some(why_str) = &details.why {
            why = Some(String::from("\n\n") + &why_str.to_owned());
        }
    }

    metadata.insert("title", title.to_owned());
    metadata.insert("description", description.to_owned());
    extra.insert("url", repo.url.to_owned());
    metadata.insert("date", epoch_to_date(repo.last_update));

    if latest {
        let path = String::from("./") + STR_DIR_REPOS + &slugify(&title);
        metadata.insert("path", path);
    }

    if let Some(details) = &repo.details {
        let mut keywords = Vec::new();

        if let Some(technology) = &details.technology {
            keywords.extend(technology.to_owned());
        }

        if let Some(languages) = &details.languages {
            keywords.extend(languages.to_owned());
        }

        if !keywords.is_empty() {
            extra.insert("keywords", get_language_string(lang_colors, &keywords));
        }

        details.color.as_ref().and_then(|colors| {
            if !colors.is_empty() {
                extra.insert("color", format!("#{:x}", colors[0]))
            } else {
                None
            }
        });

        details
            .logo
            .as_ref()
            .and_then(|logo| extra.insert("logo", logo.to_owned()));

        details.demo.as_ref().and_then(|demo| {
            if [".mp4", ".webm", ".ogg"]
                .iter()
                .any(|ext| demo.ends_with(ext))
            {
                extra.insert("demo_video", demo.to_owned())
            } else {
                extra.insert("demo", demo.to_owned())
            }
        });

        details
            .highlight
            .as_ref()
            .and_then(|h| extra.insert("highlight", h.to_owned()));
    }

    let mut page = build_page(
        metadata,
        Some(extra),
        Some(HashSet::from(["date", "keywords"].map(String::from))),
    );

    page += &description;

    if let Some(why) = why {
        page += &why;
    }

    page += "\n";

    (title, page)
}

pub fn project(lang_colors: &ColorMap, project: &Project) -> PageCollection {
    let mut result = PageCollection::default();
    let mut extra: MetaMap = HashMap::new();
    let title = project.name.to_owned();
    let slug = slugify(&title);

    let mut metadata: MetaMap = HashMap::new();
    metadata.insert("title", slug.to_owned());
    metadata.insert("date", epoch_to_date(get_project_epoch(project)));
    metadata.insert("template", String::from("404.html"));
    let page = build_page(
        metadata,
        None,
        Some(HashSet::from(["date", "keywords"].map(String::from))),
    );

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
    let mut index_page = build_page(
        metadata,
        Some(extra),
        Some(HashSet::from(["date", "keywords"].map(String::from))),
    );

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
                .clone().map(|s| s.trim().to_string() + "\n")
                .unwrap_or(String::default());
        }
        dir.push(STR_FILE_PROJECT_MAIN_REPO);
        result.files.insert(dir.to_owned(), index_page);
        dir.pop();
    }

    for repo in &project.repo_sub {
        let (title, page) = get_repo_strings(lang_colors, repo, false);
        dir.push(STR_FILE_PROJECT_REPO.replace("{}", &title));
        result.files.insert(dir.to_owned(), page);
        dir.pop();
    }

    result
}

pub fn repo(lang_colors: &ColorMap, repo: &Repo) -> Page {
    let (title, data) = get_repo_strings(lang_colors, repo, false);

    let mut path: PathBuf = [STR_DIR_ZOLA, STR_DIR_REPOS].iter().collect();
    path = path.canonicalize().unwrap();
    path.push(STR_FILE_REPO.replace("{}", &title));

    Page { path, data }
}

pub fn latest_repo(lang_colors: &ColorMap, repo: &Repo) -> Page {
    let (title, data) = get_repo_strings(lang_colors, repo, true);

    let mut path: PathBuf = [STR_DIR_ZOLA, STR_DIR_LATEST].iter().collect();
    path = path.canonicalize().unwrap();
    path.push(STR_FILE_LATEST.replace("{}", &title));

    Page { path, data }
}
