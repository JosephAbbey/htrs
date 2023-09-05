use paste::paste;
use std::sync::{Arc, Mutex};
use rstml_component::{
    html, write_html, HtmlComponent, HtmlContent, HtmlFormatter,
};
use warp::Filter;

macro_rules! components {
    (comp $name:ident $content:block) => {
        #[derive(HtmlComponent)]
        struct $name;

        impl HtmlContent for $name {
            fn fmt(self, formatter: &mut HtmlFormatter) -> std::fmt::Result {
                write_html!(formatter, { $content })
            }
        }
    };

    (comp $name:ident($self:ident, $($attr:ident : $type:ty),*) $content:block) => {
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
                    write_html!(formatter, { $content })
                }
            }
        }
    };

    ($(comp $name:ident$(($self:ident, $($attr:ident : $type:ty),*))? $content:block)*) => {
        $(components! { comp $name$(($self, $($attr : $type),*))? $content })*
    }
}

macro_rules! reply {
    ($r:expr) => {
        warp::reply::html($r.into_string().expect("500"))
    };
}

macro_rules! routes {
    ($route:expr => $resolve:expr; $($routes:expr => $resolves:expr;)*) => {
        warp::get().and(
            $route.map($resolve)
                $(.or($routes.map($resolves)))*
        )
    };
}

components! {
    comp NavBar {
        html! {
            <nav>
                <ul>
                    <li><a href="/">Home</a></li>
                    <li><a href="/about">About</a></li>
                </ul>
            </nav>
        }
    }

    comp Template(
            self,
            title: Into<String>,
            children: HtmlContent
        ) {
        html! {
            <!DOCTYPE html>
            <html>
                <head>
                    <title>{self.title.into()}</title>
                    <script
                        src="https://unpkg.com/htmx.org@1.9.5"
                        integrity="sha384-xcuj3WpfgjlKF+FXhSQFQ0ZNr39ln+hwjN3npfM9VBnUskLolQAcN80McRIVOPuO" 
                        crossorigin="anonymous"
                    ></script>
                </head>
                <body>
                    <NavBar />
                    <main>
                        {self.children}
                    </main>
                </body>
            </html>
        }
    }

    comp Page(
        self,
        title: Into<String>,
        heading: Into<String>,
        children: HtmlContent
    ) {
        html! {
            <Template title=self.title>
                <h1>{self.heading.into()}</h1>
                <p>This is a test</p>
                {self.children}
            </Template>
        }
    }

    comp Counter(
        self,
        count: Into<i32>
    ) {
        html! {
            <div id="counter">
                <button hx-get="/counter/-" hx-swap="outerHTML" hx-target="#counter">-</button>
                {self.count.into()}
                <button hx-get="/counter/+" hx-swap="outerHTML" hx-target="#counter">+</button>
            </div>
        }
    }
}

#[tokio::main]
async fn main() {
    let count = Arc::new(Mutex::new(0i32));

    warp::serve(
        routes! {
            warp::path!() => {
                    let count = count.clone();
                    move || reply!(html!(
                        <Page title="Hello" heading="Hello world">
                            <button hx-get="/hello/there" hx-swap="outerHTML">click me!</button>
                            <Counter count={*count.lock().unwrap()} />
                        </Page>
                    ))
                };
            warp::path!("hello" / String) => |name| format!("Hello, {}!", name);
            warp::path!("counter" / "-") => { let count = count.clone(); move || { *count.lock().unwrap() -= 1; reply!(html!(<Counter count={*count.lock().unwrap()} />)) } };
            warp::path!("counter" / "+") => { let count = count.clone(); move || { *count.lock().unwrap() += 1; reply!(html!(<Counter count={*count.lock().unwrap()} />)) } };
        }
    )
    .run(([127, 0, 0, 1], 3030))
    .await;
}
