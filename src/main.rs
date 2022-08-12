use anyhow::{bail, Result};
use chrono::{Duration, FixedOffset, TimeZone, Utc};
use openweathermap::*;
use serde_derive::Deserialize;
use structopt::StructOpt;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TimeZoneCSV {
    zone_name: String,
    country_code: String,
    abbreviation: String,
    time_start: i64,
    gmt_offset: i32,
    dst: i32,
}

#[derive(Debug, Deserialize)]
struct CustomPlaces {
    name: String,
    lat: f64,
    lng: f64,
    time_zone: String
}

#[derive(Debug, Deserialize)]
struct CustomTimezones {
    name: String,
    time_zone: String
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct WordCities {
    city: String,
    city_ascii: String,
    lat: f64,
    lng: f64,
    country: String,
    iso2: String,
    iso3: String,
    admin_name: String,
    capital: String,
    population: Option<f64>,
    id: i64,
}

#[derive(StructOpt)]
struct Opt {
    #[structopt(long)]
    lat: Option<f64>,

    #[structopt(long)]
    lon: Option<f64>,

    #[structopt(long)]
    loc: Option<String>,

    #[structopt(long)]
    utc: Option<i32>,

    #[structopt(long, default_value = "0")]
    days: f64,

    #[structopt(long)]
    hours: Option<f64>,

    #[structopt(long, default_value = "MY_API_KEY")]
    api_key: String,
}

fn get_latlonloc(
    lat: f64,
    lon: f64,
    loc: &str,
    time: i32,
    unix: i64,
) -> Result<(f64, f64, String, Option<FixedOffset>)> {
    let mut m_lat = lat;
    let mut m_lon = lon;
    let mut timeoffset = if time < 0 {
        FixedOffset::west(-time * 3600)
    } else {
        FixedOffset::east(time * 3600)
    };

    fn match_timezone(city: &str, ux: i64, tos: FixedOffset) -> Result<FixedOffset> {
        match find_timezone(city, ux) {
            Ok(tz) => match tz {
                Some(timezone) => {
                    if timezone < 0 {
                        Ok(FixedOffset::west(-timezone * 3600))
                    } else {
                        Ok(FixedOffset::east(timezone * 3600))
                    }
                }
                None => Ok(tos),
            },
            Err(e) => bail!("Error {} loading file", e),
        }
    }

    match find_customplace(loc) {
        Ok(stp) => match stp {
            Some(spacetime) => {
                m_lat = spacetime.lat;
                m_lon = spacetime.lng;
                timeoffset = match_timezone(&spacetime.time_zone, unix, timeoffset).unwrap();
            }
            None => {
                if m_lat == 0.0 && m_lon == 0.0 {
                    match find_latlong(loc) {
                        Ok(l) => {
                            if let Some(latlon) = l {
                                m_lat = latlon.0;
                                m_lon = latlon.1;
                            }
                        }
                        Err(e) => bail!("Error {} loading file", e),
                    }
                    if time == 0 {
                        match find_timezone(loc, unix) {
                            Ok(tz) => match tz {
                                Some(timezone) => {
                                    if timezone < 0 {
                                        timeoffset = FixedOffset::west(-timezone * 3600);
                                    } else {
                                        timeoffset = FixedOffset::east(timezone * 3600);
                                    }
                                }
                                None => {
                                    return Ok((m_lat, m_lon, format!("{} [{},{}]", loc, m_lat, m_lon), None));
                                }
                            },
                            Err(e) => bail!("Error {} loading file", e),
                        }
                    }
                }
            }
        }
        Err(e) => bail!("Error {} loading file", e),
    }


    return Ok((
        m_lat,
        m_lon,
        format!("{} [{},{}]", loc, m_lat, m_lon),
        Some(timeoffset),
    ));
}

fn calc_wetbulb(temp_c: f64, humid: f64) -> f64 {
    temp_c * (0.151977 * (humid + 8.313659).powf(1.0 / 2.0)).atan() + (temp_c + humid).atan()
        - (humid - 1.676331).atan()
        + 0.00391838 * humid.powf(3.0 / 2.0) * (0.023101 * humid).atan()
        - 4.686035
}

fn celc_to_far(temp_c: f64) -> f64 {
    (temp_c * 1.8) + 32.0
}

fn print_current(current: Current, location: String, timezone: Option<FixedOffset>) -> Result<()> {
    let wet_bulb_c = calc_wetbulb(current.temp, current.humidity);
    let wet_bulb_f = celc_to_far(wet_bulb_c);

    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    if wet_bulb_f <= 80.0 {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    } else if wet_bulb_f <= 85.0 {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
    } else if wet_bulb_f <= 88.0 {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
    } else if wet_bulb_f <= 90.0 {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
    } else if wet_bulb_f > 90.0 {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
    }

    match timezone {
        Some(timezone) => {
            println!(
                "Weather for {} on {}",
                location,
                Utc.timestamp(current.dt, 0).with_timezone(&timezone)
            );
        }
        None => {
            println!(
                "Weather for {} on {}",
                location,
                Utc.timestamp(current.dt, 0)
            );
        }
    }
    for elem in current.weather {
        println!("Short weather: {}", elem.main);
        println!("Weather description: {}", elem.description);
    }
    println!("Temperature: {}ºC", current.temp);
    println!("Humidity: {}%", current.humidity);
    println!("Pressure: {}hPa", current.pressure);
    println!("Cloud cover: {}%", current.clouds);
    println!("Dew Point: {}ºC", current.dew_point);
    println!("Heat Index: {}ºC", current.feels_like);
    println!("Wet Bulb: {:.2}ºC", wet_bulb_c);
    if let Some(snow) = current.snow {
        if let Some(h1) = snow.h1 {
            println!("Snow one-hour: {}mm", h1);
        }
        if let Some(h3) = snow.h3 {
            println!("Snow three-hour: {}mm", h3);
        }
    }

    if let Some(rain) = current.rain {
        if let Some(h1) = rain.h1 {
            println!("Rain one-hour: {}mm", h1);
        }
        if let Some(h3) = rain.h3 {
            println!("Rain three-hour: {}mm", h3);
        }
    }

    match timezone {
        Some(tz) => {
            if let Some(sunrise) = current.sunrise {
                println!("Sunrise: {}", Utc.timestamp(sunrise, 0).with_timezone(&tz));
            }
            if let Some(sunset) = current.sunset {
                println!("Sunset: {}", Utc.timestamp(sunset, 0).with_timezone(&tz));
            }
        }
        None => {
            if let Some(sunrise) = current.sunrise {
                println!("Sunrise: {}", Utc.timestamp(sunrise, 0));
            }
            if let Some(sunset) = current.sunset {
                println!("Sunset: {}", Utc.timestamp(sunset, 0));
            }
        }
    }
    println!("UV Index: {}", current.uvi);
    println!("Visibility: {}m", current.visibility);
    println!("Wind degrees: {}º", current.wind_deg);

    if let Some(gust) = current.wind_gust {
        println!("Wind gust: {}m/s", gust);
    }
    println!("Wind speed: {}m/s", current.wind_speed);

    stdout.reset()?;

    Ok(())
}

fn load_customtz() -> Result<Vec<CustomTimezones>> {
    let bytes = std::include_bytes!("data/customtimezones.csv");
    let mut vec = Vec::new();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(bytes.as_ref());

    for result in rdr.deserialize() {
        let record: CustomTimezones = result?;
        vec.push(record);
    }

    Ok(vec)
}

fn load_customplace() -> Result<Vec<CustomPlaces>> {
    let bytes = std::include_bytes!("data/customplaces.csv");
    let mut vec = Vec::new();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(bytes.as_ref());

    for result in rdr.deserialize() {
        let record: CustomPlaces = result?;
        vec.push(record);
    }

    Ok(vec)
}

fn load_timezone() -> Result<Vec<TimeZoneCSV>> {
    let bytes = std::include_bytes!("data/timezone.csv");
    let mut vec = Vec::new();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(bytes.as_ref());

    for result in rdr.deserialize() {
        let record: TimeZoneCSV = result?;
        vec.push(record);
    }

    Ok(vec)
}

fn load_cities() -> Result<Vec<WordCities>> {
    let bytes = std::include_bytes!("data/worldcities.csv");
    let mut vec = Vec::new();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes.as_ref());

    for result in rdr.deserialize() {
        let record: WordCities = result?;
        vec.push(record);
    }

    Ok(vec)
}

fn find_customplace(loc: &str) -> Result<Option<CustomPlaces>> {
    match load_customplace() {
        Ok(v) => {
            let uu = v.into_iter().find(|y| &y.name == loc);
            match uu {
                Some(ci) => Ok(Some(ci)),
                None => Ok(None),
            }
        } 
        Err(e) => bail!("Error {} loading file", e),
    }
}

fn find_timezone(city: &str, unix_time: i64) -> Result<Option<i32>> {
    let mut no_space_city = city.replace(' ', "_");

    match load_customtz() {
        Ok(tz) => {
            let custom: Option<CustomTimezones> = tz.into_iter().filter(|y| y.name == city).next();
            match custom {
                Some(yes) => {
                    no_space_city = yes.time_zone;
                },
                None => ()
            }
        }
        Err(e) => bail!("Error {} loading file", e),
    }

    match load_timezone() {
        Ok(v) => {
            let mut ww: Vec<TimeZoneCSV> =
                v.into_iter().filter(|y| y.zone_name.ends_with(&no_space_city)).collect();

            ww.sort_by(|a, b| b.time_start.cmp(&a.time_start));
            let uu = ww.iter().find(|z| z.time_start <= unix_time);
            match uu {
                Some(ci) => Ok(Some(ci.gmt_offset / 3600)),
                None => Ok(None),
            }
        }
        Err(e) => bail!("Error {} loading file", e),
    }
}

fn find_latlong(city: &str) -> Result<Option<(f64, f64)>> {
    match load_cities() {
        Ok(v) => {
            let uu = v.into_iter().find(|y| (&y.city == city) | (&y.city_ascii == city));
            match uu {
                Some(ci) => Ok(Some((ci.lat, ci.lng))),
                None => Ok(None),
            }
        }
        Err(e) => bail!("Error {} loading file", e),
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let location = &opt.loc.unwrap_or_default();
    let days = opt.days;
    let hours = opt.hours.unwrap_or_default();

    if !(0.0..=5.0).contains(&days) {
        bail!("Day offset '{}' not between one and five", days);
    }

    if !(0.0..=120.0).contains(&hours) {
        bail!("Hour offset '{}' not between one and one-hundred and twenty", days);
    }

    let now = Utc::now();
    let seconds = (days * 24.0 * 60.0 * 60.0) + (hours * 60.0 * 60.0);
    let yesterday = now
        .checked_sub_signed(Duration::seconds(seconds.round() as i64))
        .unwrap();
    let yesterday_unix = yesterday.timestamp();

    let latlonloc = get_latlonloc(
        opt.lat.unwrap_or_default(),
        opt.lon.unwrap_or_default(),
        location,
        opt.utc.unwrap_or_default(),
        yesterday_unix,
    )
    .unwrap();

    if latlonloc.0 == 0.0 && latlonloc.1 == 0.0 {
        bail!(
            "Location '{}' is not recognized, and both latitude and longitude are zero.",
            latlonloc.2
        );
    }

    let api_result = blocking::timemachine(
        &latlonloc.0,
        &latlonloc.1,
        &yesterday_unix,
        "metric",
        "en",
        &opt.api_key,
    )
    .expect("Error from OpenWeather Server");

    print_current(api_result.current, latlonloc.2, latlonloc.3).unwrap();

    Ok(())
}
