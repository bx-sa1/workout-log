use std::sync::{Arc, Mutex};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use sqlite::State;
use time::{
    OffsetDateTime,
};

use time::format_description::well_known::Iso8601;

pub type AsyncDB = Arc<Mutex<DB>>;

#[derive(Serialize)]
pub struct WorkoutType {
    name: String,
    progressions: String,
}

#[derive(Serialize, Deserialize)]
pub enum WorkoutDifficulty {
    Easy,
    Medium,
    Hard,
}

impl WorkoutDifficulty {
    pub fn from(s: String) -> Option<WorkoutDifficulty> {
        match s.to_lowercase().as_str() {
            "easy" => Some(WorkoutDifficulty::Easy),
            "medium" => Some(WorkoutDifficulty::Medium),
            "hard" => Some(WorkoutDifficulty::Hard),
            _ => None
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            WorkoutDifficulty::Easy => "easy".to_string(),
            WorkoutDifficulty::Medium => "medium".to_string(),
            WorkoutDifficulty::Hard => "hard".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Workout {
    #[serde(with = "time::serde::iso8601")]
    date: OffsetDateTime,
    exercise: String,
    progression: String,
    sets: i64,
    reps: i64,
    weight: i64,
    difficulty: WorkoutDifficulty,
    notes: String,
}

pub struct DB {
    sqlite: sqlite::Connection,
}

impl DB {
    pub fn new() -> DB {
        let query = r#"
            CREATE TABLE IF NOT EXISTS workouts (
                date TEXT PRIMARY KEY,
                exercise TEXT NOT NULL,
                progression TEXT DEFAULT "",
                sets INTEGER NOT NULL,
                reps INTEGER NOT NULL,
                weight INTEGER DEFAULT 0,
                difficulty TEXT NOT NULL,
                notes TEXT DEFAULT ""
            );
            CREATE TABLE IF NOT EXISTS exercises (
                name TEXT PRIMARY KEY,
                progressions TEXT
            );
        "#;

        let sqlite = sqlite::open("workouts.db").unwrap();
        sqlite.execute(query).unwrap();
        Self { sqlite }
    }

    pub fn get_workout(&self, date: String) -> sqlite::Result<Workout> {
        let query = "SELECT * FROM workouts WHERE date = \"?\"";

        let mut statement = self.sqlite.prepare(query)?;
        statement.bind((1, date.as_str())).unwrap();

        let workout = if let Ok(State::Row) = statement.next() {
            Workout {
                date: OffsetDateTime::parse(&statement.read::<String, _>("date")?, &Iso8601::PARSING)
                    .unwrap(),
                exercise: statement.read::<String, _>("exercise")?,
                progression: statement.read::<String, _>("progression")?,
                sets: statement.read::<i64, _>("sets")?,
                reps: statement.read::<i64, _>("reps")?,
                weight: statement.read::<i64, _>("weight")?,
                difficulty: match WorkoutDifficulty::from(statement.read::<String, _>("difficulty")?) {
                    Some(d) => d,
                    None => return Err(sqlite::Error {
                        code: Some(-1),
                        message: Some("Not a valid difficulty".to_string())
                    })
                },
                notes: statement.read::<String, _>("notes")?,
            }
        } else {
            return Err(sqlite::Error {
                code: Some(-1),
                message: Some("row with id not found".to_string()),
            });
        };

        Ok(workout)
    }

    pub fn add_workout(&self, workout: Workout) -> sqlite::Result<()> {
        let query = format!(
            r#"
            INSERT INTO workouts (date, exercise, progression, sets, reps, weight, difficulty, notes)
            VALUES("{}", "{}", "{}", {}, {}, {}, "{}", "{}")
            "#,
            workout.date.format(&Iso8601::DEFAULT).unwrap(),
            workout.exercise,
            workout.progression,
            workout.sets,
            workout.reps,
            workout.weight,
            workout.difficulty.to_string(),
            workout.notes
        );

        println!("Executing \"{}\" on DB", query);

        self.sqlite.execute(query)
    }

    pub fn update_workout(&self, date: String, workout: Workout) -> sqlite::Result<()> {
        let query = format!(
            r#"UPDATE workouts
            SET date = {}
                exercise = {}
                progression = {}
                sets = {}
                reps = {}
                weight = {}
                difficulty = {}
                notes = {}
            WHERE date = {}"#,
            workout.date.format(&Iso8601::DEFAULT).unwrap(),
            workout.exercise,
            workout.progression,
            workout.sets,
            workout.reps,
            workout.weight,
            workout.difficulty.to_string(),
            workout.notes,
            date
        );

        self.sqlite.execute(query)
    }

    pub fn delete_workout(&self, date: String) -> sqlite::Result<()> {
        let query = format!("DELETE FROM workouts WHERE date = \"{}\"", date);
        
        self.sqlite.execute(query)
    }

    pub fn get_workouts(&self, limit: i64) -> sqlite::Result<Vec<Workout>> {
        let query = "SELECT * FROM workouts ORDER BY date DESC LIMIT ?;";

        let mut statement = self.sqlite.prepare(query)?;
        statement.bind((1, limit)).unwrap();

        let mut workout_list: Vec<Workout> = Vec::new();

        while let Ok(State::Row) = statement.next() {
            workout_list.push(Workout {
                date: OffsetDateTime::parse(&statement.read::<String, _>("date")?, &Iso8601::PARSING)
                    .unwrap(),
                exercise: statement.read::<String, _>("exercise")?,
                progression: statement.read::<String, _>("progression")?,
                sets: statement.read::<i64, _>("sets")?,
                reps: statement.read::<i64, _>("reps")?,
                weight: statement.read::<i64, _>("weight")?,
                difficulty: match WorkoutDifficulty::from(statement.read::<String, _>("difficulty")?) {
                    Some(d) => d,
                    None => return Err(sqlite::Error {
                        code: Some(-1),
                        message: Some("Not a valid difficulty".to_string())
                    })
                },
                notes: statement.read::<String, _>("notes")?,
            });
        } 

        Ok(workout_list)
    }
}
