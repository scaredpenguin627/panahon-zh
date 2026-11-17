fn main() {
    let config = panahon_weather_api::Config::from_env();
    eprintln!("panahon-weather-api listening on {}", config.bind);
    if let Err(error) = panahon_weather_api::run(config) {
        eprintln!("server stopped: {error}");
        std::process::exit(1);
    }
}

