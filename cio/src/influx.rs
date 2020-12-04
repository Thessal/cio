use std::env;

use chrono::offset::Utc;
use chrono::{DateTime, Duration};
use influxdb::InfluxDbWriteable;
use influxdb::{Client as InfluxClient, Query as InfluxQuery};

use crate::utils::{authenticate_github_jwt, list_all_github_repos};

pub struct Client(pub InfluxClient);

impl Client {
    pub fn new_from_env() -> Self {
        Client(
            InfluxClient::new(
                env::var("INFLUX_DB_URL").unwrap(),
                "github_webhooks",
            )
            .with_auth(
                env::var("GADMIN_SUBJECT").unwrap(),
                env::var("INFLUX_DB_TOKEN").unwrap(),
            ),
        )
    }

    pub async fn event_exists(
        &self,
        table: &str,
        github_id: i64,
        action: &str,
        time: DateTime<Utc>,
    ) -> bool {
        let flux_date_format = "%Y-%m-%dT%H:%M:%SZ";

        let read_query = InfluxQuery::raw_read_query(&format!(
            r#"from(bucket:"github_webhooks")
                    |> range(start: {}, stop: {})
                    |> filter(fn: (r) => r._measurement == "{}")
                    |> filter(fn: (r) => r.github_id == {})
                    |> filter(fn: (r) => r.action == "{}")
                    "#,
            time.format(flux_date_format),
            (time + Duration::seconds(1)).format(flux_date_format),
            table,
            github_id,
            action
        ));
        let read_result = self.0.query(&read_query).await;

        read_result.is_ok()
    }

    pub async fn update_pull_request_events(&self) {
        let github = authenticate_github_jwt();
        let repos = list_all_github_repos(&github).await;

        // For each repo, get information on the pull requests.
        for repo in repos {
            let r = github.repo(repo.owner.login, repo.name.to_string());
            // TODO: paginate.
            let pulls = r
                .pulls()
                .list(
                    &hubcaps::pulls::PullListOptions::builder()
                        .state(hubcaps::issues::State::All)
                        .build(),
                )
                .await
                .unwrap();

            for pull in pulls {
                // Add events for each pull request if it does not already exist.
                // Check if this event already exists.
                // Let's see if the data we wrote is there.
                let github_id = pull.id.to_string().parse::<i64>().unwrap();
                let exists = self
                    .event_exists(
                        "pull_request",
                        github_id,
                        "opened",
                        pull.created_at,
                    )
                    .await;

                if !exists {
                    // Add the event.
                    let pull_request_created = PullRequest {
                        time: pull.created_at,
                        repo_name: repo.name.to_string(),
                        sender: pull.user.login.to_string(),
                        action: "opened".to_string(),
                        head_reference: pull.head.commit_ref.to_string(),
                        base_reference: pull.base.commit_ref.to_string(),
                        number: pull.number.to_string().parse::<i64>().unwrap(),
                        github_id,
                    };
                    self.0
                        .query(
                            &pull_request_created
                                .clone()
                                .into_query("pull_request"),
                        )
                        .await
                        .unwrap();
                    println!("added event: {:?}", pull_request_created);
                }

                if pull.merged_at.is_some() {
                    let merged_at = pull.merged_at.unwrap();

                    // Check if we already have the event.
                    let exists = self
                        .event_exists(
                            "pull_request",
                            github_id,
                            "merged",
                            merged_at,
                        )
                        .await;

                    if !exists {
                        // Add the event.
                        let pull_request_merged = PullRequest {
                            time: merged_at,
                            repo_name: repo.name.to_string(),
                            sender: pull.user.login.to_string(),
                            action: "merged".to_string(),
                            head_reference: pull.head.commit_ref.to_string(),
                            base_reference: pull.base.commit_ref.to_string(),
                            number: pull
                                .number
                                .to_string()
                                .parse::<i64>()
                                .unwrap(),
                            github_id,
                        };
                        self.0
                            .query(
                                &pull_request_merged
                                    .clone()
                                    .into_query("pull_request"),
                            )
                            .await
                            .unwrap();
                        println!("added event: {:?}", pull_request_merged);
                    }
                }
            }
        }
    }
}

/// FROM: https://docs.github.com/en/free-pro-team@latest/developers/webhooks-and-events/webhook-events-and-payloads#push
#[derive(InfluxDbWriteable, Clone, Debug)]
pub struct Push {
    pub time: DateTime<Utc>,
    #[tag]
    pub repo_name: String,
    #[tag]
    pub sender: String,
    #[tag]
    pub reference: String,
    pub added: String,
    pub modified: String,
    pub removed: String,
    pub before: String,
    pub after: String,
    pub commit_shas: String,
}

/// FROM: https://docs.github.com/en/free-pro-team@latest/developers/webhooks-and-events/webhook-events-and-payloads#pull_request
#[derive(InfluxDbWriteable, Clone, Debug)]
pub struct PullRequest {
    pub time: DateTime<Utc>,
    #[tag]
    pub repo_name: String,
    #[tag]
    pub sender: String,
    #[tag]
    pub action: String,
    #[tag]
    pub head_reference: String,
    #[tag]
    pub base_reference: String,
    #[tag]
    pub number: i64,
    pub github_id: i64,
}

#[cfg(test)]
mod tests {
    use crate::influx::Client;

    #[tokio::test(threaded_scheduler)]
    async fn test_cron_influx() {
        let influx = Client::new_from_env();
        influx.update_pull_request_events().await;
    }
}