use std::fs::read;

use diesel::RunQueryDsl;
use juniper::FieldResult;

use super::*;

use crate::file::FileContentInput;
use content::{FileContent, FileName, FileVariant, MediaType, TextVariant};
use root::ApiContext;

use crate::api::award::ScoreAwardGrading;
use crate::api::contest_evaluation::Evaluation;
use crate::api::contest_problem::{Problem, ProblemView};
use crate::api::messages::Message;
use crate::api::root::Query;
use crate::data::award::{ScoreAwardDomain, ScoreAwardGrade, ScoreAwardValue, ScoreRange};
use crate::data::contest::ContestMaterial;
use chrono::DateTime;
use schema::contest;
use std::path::PathBuf;
use user::{User, UserId};

/// A user authorization token
#[derive(juniper::GraphQLObject)]
pub struct UserToken {
    /// The user token encoded as a JWT
    pub token: String,
    /// The ID of the user associated with the given credentials, if any
    pub user_id: Option<UserId>,
}

#[derive(juniper::GraphQLInputObject)]
pub struct ContestUpdateInput {
    pub archive_content: Option<FileContentInput>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(juniper::GraphQLInputObject)]
pub struct ContestInput {
    pub archive_content: FileContentInput,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Clone)]
pub struct Contest {
    data: ContestData,
}

impl Contest {
    pub fn current(context: &ApiContext) -> FieldResult<Self> {
        let data = contest::table.first(&context.database)?;
        Ok(Contest { data })
    }

    pub fn init(context: &ApiContext) -> FieldResult<()> {
        let now = chrono::Local::now();
        let configuration = ContestDataInput {
            archive_integrity: context.create_blob(include_bytes!(concat!(
                env!("OUT_DIR"),
                "/initial-contest.tar.xz"
            )))?,
            start_time: now.to_rfc3339(),
            end_time: (now + chrono::Duration::hours(4)).to_rfc3339(),
        };
        diesel::insert_into(schema::contest::table)
            .values(configuration)
            .execute(&context.database)?;
        Ok(())
    }

    /// Initializes a new contest with a specified configuration
    pub fn insert(context: &ApiContext, contest: ContestInput) -> FieldResult<()> {
        let contest = ContestDataInput {
            archive_integrity: context.create_blob(&contest.archive_content.decode()?)?,
            start_time: contest.start_time,
            end_time: contest.end_time,
        };
        diesel::insert_into(schema::contest::table)
            .values(contest)
            .execute(&context.database)?;
        Ok(())
    }

    pub fn update(&self, context: &ApiContext, input: ContestUpdateInput) -> FieldResult<()> {
        let changeset = ContestChangeset {
            archive_integrity: if let Some(ref content) = input.archive_content {
                Some(context.create_blob(&content.decode()?)?)
            } else {
                None
            },
            start_time: match input.start_time {
                Some(time) => Some(chrono::DateTime::parse_from_rfc3339(&time)?.to_rfc3339()),
                None => None,
            },
            end_time: match input.end_time {
                Some(time) => Some(chrono::DateTime::parse_from_rfc3339(&time)?.to_rfc3339()),
                None => None,
            },
        };

        diesel::update(schema::contest::table)
            .set(changeset)
            .execute(&context.database)?;

        Ok(())
    }

    fn contest_path(&self, context: &ApiContext) -> FieldResult<PathBuf> {
        context.unpack_archive(&self.data.archive_integrity, "contest")
    }
}

#[derive(juniper::GraphQLEnum)]
pub enum ContestStatus {
    NotStarted,
    Running,
    Ended,
}

#[juniper_ext::graphql(Context = ApiContext)]
impl Contest {
    /// Start time of the contest, as RFC3339 date
    ///
    /// Used only for display, e.g., of the elapsed time
    pub fn start_time(&self) -> &str {
        &self.data.start_time
    }

    /// End time of the contest, as RFC3339 date
    ///
    /// Used only for display, e.g., of the elapsed time
    pub fn end_time(&self) -> &str {
        &self.data.end_time
    }

    pub fn status(&self) -> ContestStatus {
        let now = DateTime::parse_from_rfc3339(&Query::server_time()).unwrap();
        let start = DateTime::parse_from_rfc3339(self.start_time()).unwrap();
        let end = DateTime::parse_from_rfc3339(self.end_time()).unwrap();

        if now < start {
            ContestStatus::NotStarted
        } else if now < end {
            ContestStatus::Running
        } else {
            ContestStatus::Ended
        }
    }

    pub fn material(&self, context: &ApiContext) -> FieldResult<ContestMaterial> {
        let contest_path = self.contest_path(context)?;

        Ok(ContestMaterial {
            title: contest_path
                .read_dir()
                .unwrap()
                .flat_map(|result| {
                    let entry = result.unwrap();
                    if entry.file_type().unwrap().is_dir() {
                        return None;
                    }

                    if let (Some(stem), Some(extension)) =
                        (entry.path().file_stem(), entry.path().extension())
                    {
                        if extension.to_str() != Some("txt") {
                            return None;
                        }
                        if stem.to_str() != Some("title") {
                            return None;
                        }

                        // TODO: handle multiple languages

                        return Some(TextVariant {
                            attributes: vec![],
                            value: String::from_utf8(read(entry.path()).unwrap()).unwrap(),
                        });
                    }
                    None
                })
                .collect(),
            description: contest_path
                .read_dir()
                .unwrap()
                .flat_map(|result| {
                    let entry = result.unwrap();
                    if entry.file_type().unwrap().is_dir() {
                        return None;
                    }

                    if let (Some(stem), Some(extension)) =
                        (entry.path().file_stem(), entry.path().extension())
                    {
                        if stem != "home" {
                            return None;
                        }

                        return Some(FileVariant {
                            attributes: vec![],
                            name: Some(FileName(entry.file_name().to_str().unwrap().to_owned())),
                            content: FileContent(read(entry.path()).unwrap()),
                            r#type: match extension.to_str().unwrap() {
                                "pdf" => Some(MediaType("application/pdf".to_owned())),
                                "md" => Some(MediaType("text/markdown".to_owned())),
                                "html" => Some(MediaType("text/html".to_owned())),
                                _ => None,
                            },
                        });
                    }
                    None
                })
                .collect(),
            resources: vec![],   // TODO
            attachments: vec![], // TODO
        })
    }

    /// All the problems
    fn problems(&self, context: &ApiContext) -> FieldResult<Vec<Problem>> {
        // Contestants can see problems only through `view`
        context.authorize_admin()?;
        Problem::all(context)
    }

    pub fn problem_set(&self, context: &ApiContext) -> FieldResult<ProblemSet> {
        context.authorize_admin()?;
        Ok(ProblemSet {
            contest: (*self).clone(),
        })
    }

    /// Get the information about this contest visible to a user
    fn view(&self, context: &ApiContext, user_id: Option<UserId>) -> FieldResult<ContestView> {
        context.authorize_user(&user_id)?;
        ContestView::new((*self).clone(), user_id)
    }

    pub fn score_range(&self, context: &ApiContext) -> FieldResult<ScoreRange> {
        Ok(ScoreRange::total(
            self.problems(context)?
                .iter()
                .map(|problem| problem.score_range(context))
                .collect::<FieldResult<Vec<_>>>()?,
        ))
    }

    pub fn score_domain(&self, context: &ApiContext) -> FieldResult<ScoreAwardDomain> {
        context.authorize_admin()?;
        Ok(ScoreAwardDomain {
            range: self.score_range(context)?,
        })
    }

    fn users(&self, context: &ApiContext) -> FieldResult<Vec<User>> {
        context.authorize_admin()?;
        User::list(context)
    }

    fn submissions(
        &self,
        context: &ApiContext,
    ) -> FieldResult<Vec<contest_submission::Submission>> {
        context.authorize_admin()?;
        contest_submission::Submission::list(context)
    }

    fn evaluations(&self, context: &ApiContext) -> FieldResult<Vec<Evaluation>> {
        context.authorize_admin()?;
        Evaluation::list(context)
    }

    fn messages(&self, context: &ApiContext) -> FieldResult<Vec<Message>> {
        context.authorize_admin()?;
        Message::list(context)
    }
}

/// Information visible to a contestant
pub struct ContestView {
    contest: Contest,
    user_id: Option<UserId>,
}

impl ContestView {
    pub fn new(contest: Contest, user_id: Option<UserId>) -> FieldResult<Self> {
        Ok(Self { contest, user_id })
    }

    pub fn user_id(&self) -> &Option<UserId> {
        &self.user_id
    }
}

#[derive(Clone)]
pub struct ProblemSet {
    #[allow(dead_code)]
    contest: Contest,
}

/// Set of problems currently active
#[juniper_ext::graphql(Context = ApiContext)]
impl ProblemSet {
    /// The list of problems
    fn problems(&self, context: &ApiContext) -> FieldResult<Vec<Problem>> {
        // TODO: return only the problems that only the user can access
        Ok(Problem::all(context)?)
    }

    /// Range of the total score, obtained as the sum of score range of each problem
    fn score_range(&self, context: &ApiContext) -> FieldResult<ScoreRange> {
        Ok(ScoreRange::total(
            self.problems(context)?
                .iter()
                .map(|problem| problem.score_range(context))
                .collect::<FieldResult<Vec<_>>>()?,
        ))
    }

    fn score_domain(&self, context: &ApiContext) -> FieldResult<ScoreAwardDomain> {
        Ok(ScoreAwardDomain {
            range: self.score_range(context)?,
        })
    }

    /// Information about this problem set visible to a user
    pub fn view(
        &self,
        context: &ApiContext,
        user_id: Option<UserId>,
    ) -> FieldResult<ProblemSetView> {
        context.authorize_user(&user_id)?;
        Ok(ProblemSetView {
            problem_set: (*self).clone(),
            user_id,
        })
    }
}

pub struct ProblemSetView {
    problem_set: ProblemSet,
    user_id: Option<UserId>,
}

#[juniper_ext::graphql(Context = ApiContext)]
impl ProblemSetView {
    fn grading(&self, context: &ApiContext) -> FieldResult<ScoreAwardGrading> {
        Ok(ScoreAwardGrading {
            domain: self.problem_set.score_domain(context)?,
            grade: match self.tackling() {
                Some(t) => Some(t.grade(context)?),
                None => None,
            },
        })
    }

    /// Current progress of user in solving the problems in this problem set
    fn tackling(&self) -> Option<ProblemSetTackling> {
        // TODO: return `None` if user is not participating in the contest
        self.user_id.as_ref().map(|user_id| ProblemSetTackling {
            problem_set: self.problem_set.clone(),
            user_id: (*user_id).clone(),
        })
    }

    fn problem_views(&self, context: &ApiContext) -> FieldResult<Vec<ProblemView>> {
        Ok(self
            .problem_set
            .problems(context)?
            .into_iter()
            .map(|problem| problem.view(context, self.user_id.clone()))
            .collect::<FieldResult<Vec<_>>>()?)
    }
}

pub struct ProblemSetTackling {
    problem_set: ProblemSet,
    user_id: UserId,
}

#[juniper_ext::graphql(Context = ApiContext)]
impl ProblemSetTackling {
    /// Total score, obtained as the sum of score of each problem
    fn score(&self, context: &ApiContext) -> FieldResult<ScoreAwardValue> {
        Ok(ScoreAwardValue::total(
            self.problem_set
                .problems(context)?
                .iter()
                .map(|problem| problem.view(context, Some(self.user_id.clone())))
                .collect::<FieldResult<Vec<_>>>()?
                .iter()
                .filter_map(|view| view.tackling())
                .map(|tackling| tackling.score(context))
                .collect::<FieldResult<Vec<_>>>()?,
        ))
    }

    fn grade(&self, context: &ApiContext) -> FieldResult<ScoreAwardGrade> {
        Ok(ScoreAwardGrade {
            domain: self.problem_set.score_domain(context)?,
            value: self.score(context)?,
        })
    }
}

#[juniper_ext::graphql(Context = ApiContext)]
impl ContestView {
    fn problem_set(&self) -> Option<ProblemSet> {
        match self.contest.status() {
            ContestStatus::NotStarted => None,
            ContestStatus::Running | ContestStatus::Ended => Some(ProblemSet {
                contest: self.contest.clone(),
            }),
        }
    }

    /// Messages visible by the current user
    fn messages(&self, context: &ApiContext) -> FieldResult<Vec<Message>> {
        Message::for_user(context, &self.user_id)
    }

    pub fn can_send_message(&self) -> bool {
        self.user_id.is_some()
    }
}

/// The configuration of a contest
#[derive(Queryable, Clone)]
pub struct ContestData {
    /// Primary key of the table. Should be *always* 0!
    #[allow(dead_code)]
    id: i32,

    archive_integrity: String,

    /// Starting time of the contest, as RFC3339 date
    start_time: String,

    /// End time of the contest, as RFC3339 date
    end_time: String,
}

#[derive(Insertable)]
#[table_name = "contest"]
pub struct ContestDataInput {
    pub archive_integrity: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(AsChangeset)]
#[table_name = "contest"]
struct ContestChangeset {
    pub archive_integrity: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}