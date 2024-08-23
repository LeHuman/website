use std::{env, fs};

use reposcrape::{
    color,
    date::Epoch,
    reposcrape::{
        cache::{ExpandedRepoCache, RepoScrapeCache, Update},
        query::{GHQuery, QueryInterface},
    },
    LocalSaveFile, LocalSaveFileCommon,
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod page;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    let mut cache = RepoScrapeCache::load_file_or_default("./.cache");

    if cache.is_empty() || cache.repos.is_outdated() {
        info!("Fetching repos");
        let colors = color::fetch_language_colors().await;
        let query =
            GHQuery::from_personal_token(env::var("GITHUB_TOKEN").expect("No Github token"));

        let fetched = query.fetch_latest("LeHuman", 16).await?;

        cache.repos.update(&fetched);
        if let Ok(colors) = colors {
            // TODO: Should colors be obtained through query? Or keep this as a general resource?
            cache.colors.update(&colors);
        }
        cache.save_file("./.cache")?;
    }

    let expanded = ExpandedRepoCache::new(cache).await;

    let mut paths = page::Paths::default();

    let date = Epoch::get_local();

    for (_name, project) in &expanded.projects {
        paths += page::project(project);
        if date - page::get_project_epoch(project) < 2629743000 {
            paths += page::latest_project(project);
        }
    }

    for (_name, repo) in &expanded.repos {
        if repo.details.is_some() {
            paths += page::repo(repo);

            if date - repo.last_update < 2629743000 {
                paths += page::latest_repo(repo);
            }
        }
    }

    for path in &paths.directories {
        let _ = fs::create_dir_all(path);
    }

    for (path, data) in &paths.files {
        let _ = fs::write(path, data);
    }

    Ok(())
}
