const root = {
	showWeather: true,
	adjustSceneToWeather: false,
	weatherRefreshRate: 60,
	hour12: false,
	locale: {
		language: "English_WORLD",
		locale: "en-CA",
		units: "metric"
	},
	defaultSunrise: "05:20",
	defaultSunset: "22:00",
	apiData: {
		"cityName": "",
		"apiKey": ""
	},
	weatherData: getDefaultWeatherData("05:20", "22:00"),
	sceneCycle: [
		[ // night
			{
				name: "sceneS0",
				bri: 0,
				sat: 0.9,
				hue: 0
			},
			{
				name: "sceneS1",
				bri: 0.2,
				sat: 1,
				hue: 24
			},
			{
				name: "sceneS2",
				bri: 0.2,
				sat: 2,
				hue: 0
			},
			{
				name: "sceneS",
				bri: 0.55,
				sat: 0.65,
				hue: 0
			},
			{
				name: "sceneC1",
				bri: 0.3,
				sat: 1,
				hue: 0
			},
			{
				name: "sceneC2",
				bri: 0.3,
				sat: 1,
				hue: 0
			}
		],
		[ // morning
			{
				name: "sceneS0",
				bri: 0.9,
				sat: 1,
				hue: 14
			},
			{
				name: "sceneS1",
				bri: 0.8,
				sat: 0.8,
				hue: 32
			},
			{
				name: "sceneS2",
				bri: 0.8,
				sat: 1,
				hue: 49
			},
			{
				name: "sceneS",
				bri: 0.8,
				sat: 0.8,
				hue: 0
			},
			{
				name: "sceneC1",
				bri: 0.8,
				sat: 0.6,
				hue: 0
			},
			{
				name: "sceneC2",
				bri: 0.8,
				sat: 0.9,
				hue: 0
			}
		],
		[ // noon
			{
				name: "sceneS0",
				bri: 1,
				sat: 1,
				hue: 0
			},
			{
				name: "sceneS1",
				bri: 1,
				sat: 1,
				hue: 0
			},
			{
				name: "sceneS2",
				bri: 1,
				sat: 1,
				hue: 0
			},
			{
				name: "sceneS",
				bri: 1,
				sat: 1,
				hue: 0
			},
			{
				name: "sceneC1",
				bri: 1,
				sat: 1,
				hue: 0
			},
			{
				name: "sceneC2",
				bri: 1,
				sat: 1,
				hue: 0
			}
		],
		[ // afternoon
			{
				name: "sceneS0",
				bri: 0.8,
				sat: 1.4,
				hue: 28
			},
			{
				name: "sceneS1",
				bri: 1,
				sat: 1,
				hue: 34
			},
			{
				name: "sceneS2",
				bri: 0.9,
				sat: 1.1,
				hue: 59
			},
			{
				name: "sceneS",
				bri: 0.9,
				sat: 0.9,
				hue: -15
			},
			{
				name: "sceneC1",
				bri: 0.9,
				sat: 1.4,
				hue: -17
			},
			{
				name: "sceneC2",
				bri: 0.7,
				sat: 1.4,
				hue: 16
			}
		]
	]
};

const languages = [{
		language: "English_WORLD",
		locale: "en-CA",
		units: "metric"
	},
	{
		language: "Bahasa_Indonesia",
		locale: "id-ID",
		units: "metric"
	},
	{
		language: "Bahasa_Melayu",
		locale: "ms-MY",
		units: "metric"
	},
	{
		language: "Bulgarian",
		locale: "bg-BG",
		units: "metric"
	},
	{
		language: "Chinese",
		locale: "zh-CN",
		units: "metric"
	},
	{
		language: "Croatian",
		locale: "hr-HR",
		units: "metric"
	},
	{
		language: "Czech",
		locale: "cs-CZ",
		units: "metric"
	},
	{
		language: "Danish",
		locale: "da-DK",
		units: "metric"
	},
	{
		language: "Dutch",
		locale: "nl-NL",
		units: "metric"
	},
	{
		language: "English_GB",
		locale: "en-GB",
		units: "metric"
	},
	{
		language: "English_US",
		locale: "en-US",
		units: "imperial"
	},
	{
		language: "Finnish",
		locale: "fi-FI",
		units: "metric"
	},
	{
		language: "French",
		locale: "fr-FR",
		units: "metric"
	},
	{
		language: "German",
		locale: "de-DE",
		units: "metric"
	},
	{
		language: "Greek",
		locale: "el-GR",
		units: "metric"
	},
	{
		language: "Hindi",
		locale: "hi-IN",
		units: "metric"
	},
	{
		language: "Hungarian",
		locale: "hu-HU",
		units: "metric"
	},
	{
		language: "Italian",
		locale: "it-IT",
		units: "metric"
	},
	{
		language: "Japanese",
		locale: "ja-JP",
		units: "metric"
	},
	{
		language: "Korean",
		locale: "ko-KR",
		units: "metric"
	},
	{
		language: "Norwegian",
		locale: "no-NO",
		units: "metric"
	},
	{
		language: "Polish",
		locale: "pl-PL",
		units: "metric"
	},
	{
		language: "Portuguese",
		locale: "pt-PT",
		units: "metric"
	},
	{
		language: "Romanian",
		locale: "ro-RO",
		units: "metric"
	},
	{
		language: "Russian",
		locale: "ru-RU",
		units: "metric"
	},
	{
		language: "Slovak",
		locale: "sk-SK",
		units: "metric"
	},
	{
		language: "Spanish",
		locale: "es-ES",
		units: "metric"
	},
	{
		language: "Swedish",
		locale: "sv-SE",
		units: "metric"
	},
	{
		language: "Thai",
		locale: "th-TH",
		units: "metric"
	},
	{
		language: "Turkish",
		locale: "tr-TR",
		units: "metric"
	},
	{
		language: "Ukrainian",
		locale: "uk-UA",
		units: "metric"
	},
	{
		language: "Vietnamese",
		locale: "vi-VN",
		units: "metric"
	},
];

drawDate();
getWeather();
drawScene();
setInterval(drawDate, 1000);
var weatherInterval = setInterval(getWeather, root.weatherRefreshRate * 60 * 1000);
setInterval(drawScene, 60 * 1000);