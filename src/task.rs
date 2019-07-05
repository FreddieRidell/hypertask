use crate::engine::{Mutation, Mutations, Queries, Query};
use crate::id::Id;
use crate::prop::Prop;
use crate::recur::Recur;
use crate::tag::{Sign, Tag};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use time::Duration;

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    created_at: DateTime<Utc>,
    description: Option<String>,
    done: Option<DateTime<Utc>>,
    due: Option<DateTime<Utc>>,
    id: Id,
    recur: Option<Recur>,
    snooze: Option<DateTime<Utc>>,
    tags: HashSet<String>,
    updated_at: DateTime<Utc>,
    wait: Option<DateTime<Utc>>,
}

impl Task {
    pub fn generate() -> Self {
        Self {
            created_at: Utc::now(),
            description: None,
            done: None,
            due: None,
            id: Id::generate(),
            recur: None,
            snooze: None,
            tags: HashSet::new(),
            updated_at: Utc::now(),
            wait: None,
        }
    }

    pub fn to_renderable_hash_map(&self) -> HashMap<&str, String> {
        let mut hm = HashMap::<&str, String>::new();

        let Id(id) = &self.id;
        hm.insert("id", id.to_string());

        if let Some(description) = &self.description {
            hm.insert("description", description.to_string());
        }

        hm.insert(
            "tags",
            self.tags
                .iter()
                .map(|t| format!("+{}", t))
                .collect::<Vec<String>>()
                .join(" "),
        );

        if let Some(due) = &self.due {
            hm.insert("due", due.format("%Y-%m-%d %H:%M:%S").to_string());
        }

        if let Some(recur) = &self.recur {
            hm.insert("recur", format!("{}", recur));
        }

        hm
    }

    pub fn get_id(&self) -> &Id {
        &(self.id)
    }

    pub fn satisfies_queries(&self, queries: &Queries) -> bool {
        for q in queries {
            if self.satisfies_query(q) {
                continue;
            } else {
                return false;
            }
        }

        true
    }

    pub fn satisfies_query(&self, query: &Query) -> bool {
        match query {
            Query::Id(id) => id == &self.id,
            Query::Tag(Tag { sign: _, name }) => self.tags.contains(name),
        }
    }

    pub fn apply_mutations(&mut self, mutations: &Mutations) -> &Self {
        for m in mutations {
            self.apply_mutation(m);
        }

        self
    }

    pub fn apply_mutation(&mut self, mutation: &Mutation) -> &Self {
        match mutation {
            Mutation::SetTag(Tag {
                sign: Sign::Plus,
                name,
            }) => {
                self.tags.insert(name.to_string());
            }
            Mutation::SetTag(Tag {
                sign: Sign::Minus,
                name,
            }) => {
                self.tags.remove(name);
            }
            Mutation::SetProp(Prop::Description(description)) => {
                self.description = Some(description.to_string());
            }
            Mutation::SetProp(Prop::Done(done)) => {
                if let Some(recur) = &self.recur {
                    let dt: Duration = Duration::from(recur);

                    if let Some(due) = self.due {
                        self.due = Some(due + dt);
                    }
                    if let Some(wait) = self.wait {
                        self.wait = Some(wait + dt);
                    }
                } else {
                    self.done = Some(done.clone());
                }
            }
            Mutation::SetProp(Prop::Due(due)) => {
                self.due = due.clone();
            }
            Mutation::SetProp(Prop::Snooze(snooze)) => {
                self.snooze = snooze.clone();
            }
            Mutation::SetProp(Prop::Wait(wait)) => {
                self.wait = wait.clone();
            }
            Mutation::SetProp(Prop::Recur(recur)) => self.recur = recur.clone(),
        }

        self.updated_at = Utc::now();

        self
    }

    pub fn get_score(&self) -> u64 {
        //this is perfectly fine for now, but I'd like to aim to replace this with
        //something user-configureable, possibly https://github.com/jonathandturner/rhai

        let mut score: u64 = 0;

        if let Some(wait) = self.due {
            if wait > Utc::now() {
                return 0;
            }
        }

        if let Some(snooze) = self.due {
            if snooze > Utc::now() {
                return 0;
            }
        }

        score = score + ((Utc::now() - self.updated_at).num_minutes() as u64).pow(2);

        if let Some(due) = self.due {
            score = score + (2147483647 - (due.timestamp() as u64));
        }

        score
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Task) -> Option<Ordering> {
        Some(self.get_score().cmp(&other.get_score()).reverse())
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Task) -> Ordering {
        self.get_score().cmp(&other.get_score()).reverse()
    }
}

impl Eq for Task {}
impl PartialEq for Task {
    fn eq(&self, other: &Task) -> bool {
        self.get_score() == other.get_score()
    }
}