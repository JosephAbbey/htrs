use serde_derive::Deserialize;
use std::sync::{Arc, Mutex};
use warp::Filter;

pub type Db = Arc<Mutex<Vec<ToDo>>>;

pub fn empty() -> Db {
    Arc::new(Mutex::new(Vec::new()))
}

pub fn with_db(
    db: Db,
) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub fn todos(db: Db) -> Vec<ToDo> {
    (*db.lock().unwrap()).clone()
}

pub fn create_todo(db: Db, new: ToDo) -> Result<(), ()> {
    let mut todos = db.lock().unwrap();

    for todo in todos.iter() {
        if todo.id == new.id {
            return Err(());
        }
    }

    todos.push(new);

    Ok(())
}

pub fn delete_todo(db: Db, id: usize) -> Result<(), ()> {
    let mut todos = db.lock().unwrap();

    let len = todos.len();
    todos.retain(|todo| todo.id != id);

    // If the vec is smaller, we found and deleted a Todo!
    if todos.len() != len {
        Ok(())
    } else {
        Err(())
    }
}

pub fn update_todo(db: Db, id: usize, new: ToDo) -> Result<(), ()> {
    let mut todos = db.lock().unwrap();

    for todo in todos.iter_mut() {
        if todo.id == id {
            *todo = new;
            return Ok(())
        }
    }

    Err(())
}

#[derive(Clone, Deserialize)]
pub struct ToDo {
    pub id: usize,
    pub text: String,
}

