# openweathercli
A small cli app using the [openweathermap fork](https://github.com/Dr-Kristau/openweathermap)

Usage:
```bash
openweathercli --lat 30.267222 --lon -97.743056 --loc Austin_TX --days 0.5 --api_key <MY_API_KEY>
```

Where:
- `--lat` `--lon` = latitude and longitude in decimal format
- `--loc` = location label 
- `--days` = number of decimal days between zero and five to subtract from present time
- `--api_key` = the OpenWeather API key
