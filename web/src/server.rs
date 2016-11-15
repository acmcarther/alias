use database::RedisClient;
use iron::Iron;
use list::List;
use mount::Mount;
use redirect::RedirectConfig;
use redirect::Redirecter;
use router::Router;
use set::SetMapping;
use staticfile::Static;
use std::net::ToSocketAddrs;

pub fn run<A: ToSocketAddrs>(client: RedisClient, addr: A) {
  let router = make_router(client);
  Iron::new(router).http(addr);
}

pub fn make_router(client: RedisClient) -> Router {
  let mut r = Router::new();
  r.get("/", Static::new("./assets/home.html"), "home");
  r.post("/set",
         SetMapping::new(RedirectConfig::new(client.with_suffix("/g/")),
                         RedirectConfig::new(client.with_suffix("/c/"))),
         "set");
  r.get("/g/*",
        generated_redirect_mount(client.with_suffix("/g/")),
        "generated_redirect");
  r.get("/c/*",
        custom_redirect_mount(client.with_suffix("/c/")),
        "custom_redirect");
  r.get("/list", List::new(RedirectConfig::new(client)), "list_all");
  r
}

pub fn generated_redirect_mount(client: RedisClient) -> Mount {
  let mut m = Mount::new();
  m.mount("/g/", Redirecter::new(RedirectConfig::new(client)));
  m
}

pub fn custom_redirect_mount(client: RedisClient) -> Mount {
  let mut m = Mount::new();
  m.mount("/c/", Redirecter::new(RedirectConfig::new(client)));
  m
}
