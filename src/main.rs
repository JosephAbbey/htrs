mod db;

use paste::paste;
use rstml_component::{html, write_html, For, HtmlComponent, HtmlContent, HtmlFormatter};
use warp::{http, Filter};

macro_rules! components {
    (comp $name:ident render $content:expr) => {
        #[derive(HtmlComponent)]
        struct $name;

        impl HtmlContent for $name {
            fn fmt(self, formatter: &mut HtmlFormatter) -> std::fmt::Result {
                write_html!(formatter, { $content })
            }
        }
    };

    (comp $name:ident($self:ident, $($attr:ident : $type:ty),*) $({ $($extra:stmt)* })? render $content:expr) => {
        paste! {
            #[derive(HtmlComponent)]
            struct $name<$([<T $attr>]),*>
            where
                $([<T $attr>]: $type,)*
            {
                $($attr: [<T $attr>]),*
            }

            impl<$([<T $attr>]),*> HtmlContent for $name<$([<T $attr>]),*>
            where
                $([<T $attr>]: $type,)*
            {
                fn fmt($self, formatter: &mut HtmlFormatter) -> std::fmt::Result {
                    $($($extra)*)?
                    write_html!(formatter, { $content })
                }
            }
        }
    };

    ($(comp $name:ident$(($self:ident, $($attr:ident : $type:ty),*))? $({ $($extra:stmt)* })? render $content:expr)*) => {
        $(components! { comp $name$(($self, $($attr : $type),*))? $({ $($extra)* })? render $content })*
    }
}

macro_rules! reply {
    ($r:expr) => {
        warp::reply::html($r.into_string().expect("500"))
    };
}

macro_rules! routes {
    ($route:expr => $resolve:expr, $($routes:expr => $resolves:expr),*) => {
        $route.map($resolve)
            $(.or($routes.map($resolves)))*
    }
}

components! {
    comp Template(
        self,
        title: Into<String>,
        children: HtmlContent
    ) render html! {
        <!DOCTYPE html>
        <html>
            <head>
                <title>{self.title.into()}</title>
                <script
                    src="https://unpkg.com/htmx.org@1.9.5"
                    integrity="sha384-xcuj3WpfgjlKF+FXhSQFQ0ZNr39ln+hwjN3npfM9VBnUskLolQAcN80McRIVOPuO"
                    crossorigin="anonymous"
                ></script>
                <meta name="color-scheme" content="dark light" />
            </head>
            <body>
                <main>
                    {self.children}
                </main>
            </body>
        </html>
    }

    comp Main(
        self,
        todos: IntoIterator<Item = db::ToDo>
    ) render html! {
        <Template title="ToDos App">
            <h1>ToDos App</h1>
            <button hx-get="/todos" hx-target="#todos" hx-indicator="#spinner">refresh</button>
            <form
                hx-post="/todos"
                hx-vars="js:{ id: Math.max(-1, ...[...document.querySelectorAll('#todos li')].map((e) => parseInt(e.id))) + 1 }"
                hx-target="#todos > ul"
                hx-swap="beforeend"
                hx-on:htmx:after-request="this.reset()"
                hx-on:htmx:response-error="alert('error: ' + event.detail.xhr.status)"
            >
                <input type="text" name="text" />
                <button type="submit">add</button>
            </form>
            <div id="todos">
                <ToDos todos=self.todos />
            </div>
            <img id="spinner" class="htmx-indicator" src="https://i.gifer.com/ZKZg.gif" width="40" />
        </Template>
    }

    comp ToDo(
        self,
        todo: Into<db::ToDo>
    ) {
        let todo: db::ToDo = self.todo.into()
    } render html! {
        <li id=todo.id>
            <input
                value=todo.text
                name="text"
                hx-put=format!("/todos/{}", todo.id)
                hx-vals=format!("{{ \"id\": \"{}\" }}", todo.id)
            />
            <button
                hx-delete=format!("/todos/{}", todo.id)
                hx-target="closest li"
                hx-swap="outerHTML"
            >delete</button>
        </li>
    }

    comp ToDos(
        self,
        todos: IntoIterator<Item = db::ToDo>
    ) render html! {
        <ul>
            <For items={self.todos}>
                {|f, todo| write_html!(f, <ToDo todo=todo />)}
            </For>
        </ul>
    }
}

#[tokio::main]
async fn main() {
    // in-memory database
    let db: db::Db = db::empty();

    warp::serve(
        routes! {
            warp::get().and(warp::path!()).and(db::with_db(db.clone())) => |db| reply!(html!(<Main todos={db::todos(db)} />)),
            warp::get().and(warp::path!("todos")).and(db::with_db(db.clone())) => |db| reply!(html!(<ToDos todos={db::todos(db)} />)),
            warp::post().and(warp::path!("todos")).and(warp::body::form::<db::ToDo>()).and(db::with_db(db.clone())) => |body: db::ToDo, db|
                match db::create_todo(db, body.clone()) {
                    Ok(())  => warp::reply::with_status(reply!(html!(<ToDo todo=body />)), http::StatusCode::CREATED),
                    Err(()) => warp::reply::with_status(warp::reply::html("".into()), http::StatusCode::BAD_REQUEST)
                },
            warp::delete().and(warp::path!("todos" / usize)).and(db::with_db(db.clone())) => |id, db|
                match db::delete_todo(db, id) {
                    Ok(())  => warp::reply::with_status("", http::StatusCode::OK),
                    Err(()) => warp::reply::with_status("", http::StatusCode::NOT_FOUND)
                },
            warp::put().and(warp::path!("todos" / usize)).and(warp::body::form::<db::ToDo>()).and(db::with_db(db)) => |id, body: db::ToDo, db|
                match db::update_todo(db, id, body) {
                    Ok(())  => warp::reply::with_status("", http::StatusCode::NO_CONTENT),
                    Err(()) => warp::reply::with_status("", http::StatusCode::NOT_FOUND)
                }
        }
    )
    .run(([127, 0, 0, 1], 80))
    .await;
}
