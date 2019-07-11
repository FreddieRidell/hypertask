use crate::id::Id;
use crate::prop::Prop;
use crate::tag::Tag;
use crate::task::Task;

#[derive(Debug)]
pub enum Mutation {
    SetProp(Prop),
    SetTag(Tag),
}

#[derive(Debug)]
pub enum Query {
    Id(Id),
    Tag(Tag),
}

pub type Mutations = Vec<Mutation>;
pub type Queries = Vec<Query>;

#[derive(Debug)]
pub enum Engine {
    Create(Mutations),
    Read(Queries),
    Update(Queries, Mutations),
    Delete(Queries),
}

impl Engine {
    pub fn run(
        self,
        input_tasks_iter: impl Iterator<Item = Result<Task, String>>,
        put_task: impl Fn(&Task) -> Result<(), String>,
    ) -> Vec<Task> {
        let mut tasks_collection = match &self {
            Engine::Create(mutations) => {
                let mut new_task = Task::generate();

                new_task.apply_mutations(mutations);

                put_task(&new_task).unwrap();

                vec![new_task]
            }

            Engine::Read(queries) => input_tasks_iter
                .map(|r| r.unwrap())
                .filter(|t| queries.len() == 0 || t.satisfies_queries(queries))
                .filter(|t| t.get_score() != 0)
                .collect::<Vec<Task>>(),

            Engine::Update(queries, mutations) => input_tasks_iter
                .map(|r| r.unwrap())
                .filter(|t| t.satisfies_queries(queries))
                .map(|mut t| {
                    t.apply_mutations(mutations);
                    put_task(&t).unwrap();
                    t
                })
                .collect::<Vec<Task>>(),

            Engine::Delete(queries) => input_tasks_iter
                .map(|r| r.unwrap())
                .filter(|t| t.satisfies_queries(queries))
                .collect::<Vec<Task>>(),
        };

        tasks_collection.sort_unstable();

        tasks_collection
    }
}