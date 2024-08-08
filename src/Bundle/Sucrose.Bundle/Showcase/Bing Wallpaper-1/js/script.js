var bingwallpaper = document.getElementById("bingwallpaper");

async function GetBingWallpaper() {
	try {
		const response = await fetch("https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en-GB");
		const data = await response.json();
		const wallpaper = data.images[0];

		const wallpaperUrl = `https://www.bing.com${wallpaper.url}`;

		bingwallpaper.src = wallpaperUrl;
		bingwallpaper.alt = wallpaper.title;
	} catch (error) {
		console.error("An error occurred while loading Bing Wallpaper: ", error);
	}
}

function SucroseStretchMode(Type) {
	if (bingwallpaper.alt != "Bing Wallpaper") {
		switch (Type) {
			case "None":
				bingwallpaper.style.objectFit = "none";
				break;
			case "Fill":
				bingwallpaper.style.objectFit = "fill";
				break;
			case "Uniform":
				bingwallpaper.style.objectFit = "contain";
				break;
			case "UniformToFill":
				bingwallpaper.style.objectFit = "cover";
				break;
			default:
				break;
		}
	}
}

window.addEventListener("load", GetBingWallpaper);

setInterval(GetBingWallpaper, 5000);