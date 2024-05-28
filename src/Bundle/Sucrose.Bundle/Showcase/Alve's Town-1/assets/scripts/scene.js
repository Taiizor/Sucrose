function updateSceneMargin(val) {
	Array.from(document.getElementsByClassName("scene")).forEach((elem) => {
		elem.style.width = (100 - (2 * val)) + "vw";
		elem.style.marginLeft = val + "%";
		elem.style.marginRight = val + "%";
	});
}

function drawScene() {
	// Constants used in time calculation(minutes*seconds)
	const outerMargin = 90 * 60;;
	const innerMargin = 30 * 60;

	var nowUTC = Math.floor((new Date()).getTime() / 1000);
	var sRiseUTC = root.weatherData.sys.sunrise;
	var sSetUTC = root.weatherData.sys.sunset;

	// CSS Filter to simulate day-night cycle
	if (nowUTC < sRiseUTC - outerMargin || nowUTC >= sSetUTC + outerMargin) {
		// Night
		updateSceneFilter(-1, 0);
		updateSceneStars(1, 1, 0, 1);
	} else if (nowUTC >= sRiseUTC - outerMargin && nowUTC < sRiseUTC - innerMargin) {
		// Night to Morning
		updateSceneFilter(0, 1, sRiseUTC - innerMargin - nowUTC);
		updateSceneStars(0, nowUTC, sRiseUTC - outerMargin, sRiseUTC - innerMargin);
	} else if (nowUTC >= sRiseUTC - innerMargin && nowUTC < sRiseUTC + innerMargin) {
		// Morning
		updateSceneFilter(-1, 1);
		updateSceneStars(1, 0, 0, 1);
	} else if (nowUTC >= sRiseUTC + innerMargin && nowUTC < sRiseUTC + outerMargin) {
		// Morning to Noon
		updateSceneFilter(1, 2, sRiseUTC + outerMargin - nowUTC);
		updateSceneStars(1, 0, 0, 1);
	} else if (nowUTC >= sRiseUTC + outerMargin && nowUTC < sSetUTC - outerMargin) {
		// Noon
		updateSceneFilter(-1, 2);
		updateSceneStars(1, 0, 0, 1);
	} else if (nowUTC >= sSetUTC - outerMargin && nowUTC < sSetUTC - innerMargin) {
		// Noon to Afternoon
		updateSceneFilter(2, 3, sSetUTC - innerMargin - nowUTC);
		updateSceneStars(1, 0, 0, 1);
	} else if (nowUTC >= sSetUTC - innerMargin && nowUTC < sSetUTC + innerMargin) {
		// Afternoon
		updateSceneFilter(-1, 3);
		updateSceneStars(1, 0, 0, 1);
	} else if (nowUTC >= sSetUTC + innerMargin && nowUTC < sSetUTC + outerMargin) {
		// Afternoon to Night
		updateSceneFilter(3, 0, sSetUTC + outerMargin - nowUTC);
		updateSceneStars(1, nowUTC, sSetUTC + innerMargin, sSetUTC + outerMargin);
	}

	// Turn on/off Lights of Signs
	if (nowUTC >= sSetUTC + innerMargin * 1.5 || nowUTC < sRiseUTC - innerMargin * 1.5) {
		document.getElementById("sceneL0").style.visibility = "visible";
	} else {
		document.getElementById("sceneL0").style.visibility = "hidden";
	}

	// Turn on/off Lights Houses
	if (nowUTC >= sSetUTC + innerMargin * 2 && nowUTC < sSetUTC + outerMargin * 2) {
		document.getElementById("sceneL1").style.visibility = "visible";
	} else {
		document.getElementById("sceneL1").style.visibility = "hidden";
	}
}

// Function to update the style filter
// An element is a "fixed" state at morning, noon, afternoon and evening
// Ann element is in a "variable" state when it transforms between fixed states
function updateSceneFilter(startCycle, endCycle, diffTime) {
	// Catch empty arguments
	if (startCycle == undefined || endCycle == undefined)
		return

	if (startCycle == -1) {
		// Fixed State (ie: noon)
		root.sceneCycle[endCycle].forEach((elem) => {
			updateElementFilter(elem.name, elem.bri, elem.sat, elem.hue);
		});
	} else {
		// Variable State (ie: night to morning)
		const factor = (3600 - diffTime) / 3600;
		const end = root.sceneCycle[endCycle];
		root.sceneCycle[startCycle].forEach((elem, index) => {
			var bri = elem.bri + (end[index].bri - elem.bri) * factor;
			var sat = elem.sat + (end[index].sat - elem.sat) * factor;
			var hue = elem.hue + (end[index].hue - elem.hue) * factor;
			updateElementFilter(elem.name, bri, sat, hue);
		});
	}
}

// Set the Style filter of an element
function updateElementFilter(el, bri, sat, hue) {

	// TODO : Adjust brightness according to weather
	var filter = "brightness(" + bri + ")";
	filter += " saturate(" + sat + ")";
	filter += " hue-rotate(" + hue + "deg)";

	document.getElementById(el).style.filter = filter;
}

// Function to show stars in the sky, depending on the time of day
function updateSceneStars(i, now, start, finish) {
	const maxOp = 0.4;
	const factor = Math.abs(i - ((finish - now) / (finish - start)));
	document.getElementById("sceneS3").style.opacity = factor * maxOp;
}

// Function that shows cloud layers
function weatherCorrScene() {
	if (!root.adjustSceneToWeather)
		return;

	var vis1 = "visible";
	var vis2 = "visible";

	// Show Cloud layers, depending on weather
	switch (root.weatherData.weather[0].icon.substring(0, 2)) {
		case ("01"):
			vis1 = "hidden";
			vis2 = "hidden";
			break;
		case ("02"):
			vis2 = "hidden";
			break;
	}
	document.getElementById("sceneS1").style.visibility = vis1;
	document.getElementById("sceneS2").style.visibility = vis2;
}

// Switch animation on or off. (GIF seems to take up a lot of CPU)
function enableAnimation(b) {
	var ext = b ? "gif" : "png";
	document.getElementById("sceneC1").src = "assets/images/scene/scene_city_1." + ext;
	document.getElementById("sceneC2").src = "assets/images/scene/scene_city_2." + ext;
	document.getElementById("sceneS").src = "assets/images/scene/scene_sign." + ext;

	document.getElementsByTagName("body")[0].style.backgroundImage = "url(assets/images/scene/body_background." + ext + ")";
}