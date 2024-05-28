function SucrosePropertyListener(name, val) {
	switch (name) {
		case "sceneMargin":
			positionDateContainer(-val.value);
			positionWeatherContainer(-val.value);
			updateSceneMargin(-val.value);
			break;
		case "sceneAnimation":
			enableAnimation(val.value);
			break;
		case "sceneColor":
			dateContainer.style.color = "#" + val.value.substring(3);
			weatherContainer.style.color = "#" + val.value.substring(3);
			weatherIcon.setAttribute("style", "fill:" + "#" + val.value.substring(3));
			break;
		case "sceneAdjustWeather":
			root.adjustSceneToWeather = val.value;
			if (!val.value) {
				document.getElementById("sceneS1").style.visibility = 'visible';
				document.getElementById("sceneS2").style.visibility = 'visible';
			} else
				showWeather(root.weatherData);
			break;
		case "weatherShow":
			root.showWeather = val.value;
			weatherContainer.style.visibility = val.value ? "visible" : "hidden";
			break;
		case "weatherCityInput":
			root.apiData.cityName = val.value;
			break;
		case "weatherApiInput":
			root.apiData.apiKey = val.value;
			break;
		case "weatherRefreshFreq":
			root.weatherRefreshRate = val.value;
			clearInterval(weatherInterval);
			getWeather();
			weatherInterval = setInterval(getWeather, root.weatherRefreshRate * 60 * 1000);
			break;
		case "weatherBtnRefresh":
			// update weather and redraw scene
			reDrawWeatherScene();
			break;
		case "miscHour12":
			// set hour format
			root.hour12 = val.value;
			drawDate();
			break;
		case "miscLanguage":
			root.locale = languages[val.value];
			drawDate();
			getWeather();
			break;
		case "miscSunrise":
			root.defaultSunrise = val.value;
			// Re-set default values of weather (when api is not used)
			if (root.apiData.cityName === "" || root.apiData.cityName === "city, country" || root.apiData.apiKey === "") {
				reDrawWeatherScene();
			}
			break;
		case "miscSunset":
			root.defaultSunset = val.value;
			// Re-set default values of weather (when api is not used)
			if (root.apiData.cityName === "" || root.apiData.cityName === "city, country" || root.apiData.apiKey === "") {
				reDrawWeatherScene();
			}
			break;
	}
}

function reDrawWeatherScene() {
	getWeather();
	drawScene();
}