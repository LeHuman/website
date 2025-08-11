use std::{env, fs};

use reposcrape::{
    LocalSaveFile, LocalSaveFileCommon, color,
    date::Epoch,
    reposcrape::{
        cache::{ExpandedRepoCache, RepoScrapeCache, Update},
        query::{GHQuery, QueryInterface},
    },
};
use tracing::{Level, info, trace};
use tracing_subscriber::FmtSubscriber;

#[cfg(feature = "dot-env")]
use dotenvy::dotenv;

mod page;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    #[cfg(feature = "dot-env")]
    {
        if let Err(err) = dotenv() {
            eprintln!("Warning: Could not load .env file: {}", err);
        }
    }

    let mut cache = RepoScrapeCache::load_file_or_default("./.cache");

    if cache.is_empty() || cache.repos.is_outdated() {
        info!("Fetching repos");
        let colors = color::fetch_language_colors().await;
        let gh_token = env::var("GITHUB_TOKEN").expect("No Github token");
        let query = GHQuery::from_personal_token(gh_token);

        let fetched = query.fetch_latest("LeHuman", 64).await?;

        cache.repos.update(&fetched);
        if let Ok(colors) = colors {
            // TODO: Should colors be obtained through query? Or keep this as a general resource?
            cache.colors.update(&colors);
        }
        cache.save_file("./.cache")?;
    }

    let expanded = ExpandedRepoCache::new(cache).await;

    trace!("{}", expanded);

    let mut pages = page::PageCollection::default();

    let date = Epoch::get_local();

    for (_name, project) in &expanded.projects {
        pages += page::project(project);
        if date - page::get_project_epoch(project) < 2629743000 {
            pages += page::latest_project(project);
        }
    }

    for (_name, repo) in &expanded.repos {
        if repo.details.is_some() {
            if date - repo.last_update < 2629743000 {
                pages.insert(page::latest_repo(repo));
            }
            pages.insert(page::repo(repo));
        }
    }

    for path in &pages.directories {
        let _ = fs::create_dir_all(path);
    }

    for (path, data) in &pages.files {
        let _ = fs::write(path, data);
    }

    Ok(())
}
