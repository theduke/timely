use timely_server::Config;

fn main() {
    let config = Config::from_env().expect("invalid configuration");
    let ctx = timely_server::Context::new(config).expect("could not build server context");
    wcgi::serve_once(move |req| timely_server::handler(&ctx, req));
}
