# openweathercli
A small cli app using the [openweathermap fork](https://github.com/Dr-Kristau/openweathermap)

Usage:
```bash
openweathercli --lat 30.267222 --lon -97.743056 --loc Austin_TX --days 0.5 --utc=-5 --api_key <MY_API_KEY>
```

Where:
- `--lat` `--lon` = latitude and longitude in decimal format
- `--loc` = [optional] location label 
- `--days` = number of decimal days between zero and five to subtract from present time
- `--utc`= [optional] offset from UTC, otherwise time displayed in UTC
- `--api_key` = the OpenWeather API key

In the screenshot below the `--api_key` has been defined by default in the code:
![alt text](docs/Austin_TX.png)
