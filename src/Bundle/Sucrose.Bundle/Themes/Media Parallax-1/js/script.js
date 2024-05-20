// #region parallax

let _parallax = true,
	parallaxVal = 5;
const backgroundImageFade = document.getElementById("backgroundImageFade");
const backgroundImage = document.getElementById("backgroundImage");

//ref: https://www.geeksforgeeks.org/how-to-create-mousemove-parallax-effects-using-html-css-javascript/
document.addEventListener("mousemove", function(event) {
	if (!_parallax) return;

	const position = parallaxVal;
	const x = (window.innerWidth - event.pageX * position) / 90;
	const y = (window.innerHeight - event.pageY * position) / 90;

	backgroundImage.style.transform = `translateX(${x}px) translateY(${y}px)`;
	backgroundImageFade.style.transform = `translateX(${x}px) translateY(${y}px)`;
});

// #endreion parallax

// #region visualizer

const rainbowColors = ["#bc626b", "#cf876f", "#eacb8a", "#a3bd8d", "#88c0cf", "#b38ead"];

const TAU = Math.PI * 2;

let canvas = document.getElementById("canvas");
let max_height, startPos, vizWidth, midY;

let numDots = 128;
let dotSize = 10;
let dotScale = 0.5;
let backgroundColor = "rgb(0,0,0)";
let foregroundColor = "rgb(255,255,255)";
let rainbow = false;

let ctx = canvas.getContext("2d");
let verticalScale = 8;
let sortSoundArray = null;

function hexToRgb(hex) {
	var result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
	return result ? {
			r: parseInt(result[1], 16),
			g: parseInt(result[2], 16),
			b: parseInt(result[3], 16),
		} :
		null;
}

function calcDotSize() {
	dotSize = dotScale * (canvas.width / numDots);
}

function setCanvasSize() {
	canvas.width = window.innerWidth;
	canvas.height = window.innerHeight;
	max_height = window.innerHeight * 0.5;
	startPos = 0;
	vizWidth = window.innerWidth;
	midY = canvas.height - canvas.height * 0.5;
	calcDotSize();
}

window.onload = setCanvasSize;
window.onresize = setCanvasSize;

function concentratedSort(audioArray) {
	audioArray.sort().splice(0, audioArray.length * 0.75);
	audioArray.push(...[...audioArray].reverse());
	return audioArray;
}

function centeredSort(audioArray) {
	const newHalf = audioArray.splice(0, audioArray.length * 0.5);
	audioArray.reverse().push(...newHalf);
	return audioArray;
}

function renderAnimation(audioArray) {
	ctx.fillStyle = foregroundColor;

	for (let x = 0; x < audioArray.length; x++) {
		const soundVal = audioArray[x];

		if (rainbow) ctx.fillStyle = rainbowColors[x % rainbowColors.length] || foregroundColor;

		for (let y = 0; y < soundVal * verticalScale; y++) {
			const vertIndex = soundVal - y / verticalScale;
			const xPos = (x / audioArray.length) * canvas.width;
			const yPos = y * dotSize * 2;
			const arcRadius = Math.min(vertIndex * dotSize, dotSize);

			ctx.beginPath();
			ctx.arc(xPos, midY - yPos, arcRadius, 0, TAU);
			ctx.fill();

			ctx.beginPath();
			ctx.arc(xPos, midY + yPos, arcRadius, 0, TAU);
			ctx.fill();
		}
	}
}

// #endregion visualizer

// #region nowplaying

const colorThief = new ColorThief();
let defaultBackground = "res/background.jpg",
	defaultAlbumArt = "res/record.gif";
document.getElementById("albumart").src = defaultAlbumArt;
backgroundImage.src = defaultBackground;

function SucroseAudioData(obj) {
	if (obj != null && obj.State) {
		document.querySelector("h2").innerText = obj.AlbumArtist;
		document.querySelector("h2").innerText = obj.Artist;
		document.querySelector("h1").innerText = obj.Title;
		$("#backgroundImageFade").css("opacity", 1.0);
		backgroundImageFade.src = backgroundImage.src;
		if (obj.ThumbnailString != null) {
			document.getElementById("albumart").src = "data:image/png;base64," + obj.ThumbnailString;
			backgroundImage.src = "data:image/png;base64," + obj.ThumbnailString;
		} else {
			document.getElementById("albumart").src = defaultAlbumArt;
			backgroundImage.src = defaultBackground;
		}
	} else if (document.querySelector("h1").innerText != "Waiting for media...") {
		document.querySelector("h2").innerText = "";
		document.querySelector("h1").innerText = "Waiting for media...";
		document.getElementById("albumart").src = defaultAlbumArt;
		$("#backgroundImageFade").css("opacity", 1.0);
		backgroundImageFade.src = backgroundImage.src;
		backgroundImage.src = defaultBackground;
	}

	if (sortSoundArray)
		var audioArray = sortSoundArray?.(obj.Data);
	else
		var audioArray = obj.Data;

	ctx.clearRect(0, 0, canvas.width, canvas.height);

	renderAnimation(audioArray);
}

//background transition animation
backgroundImageFade.addEventListener("load", function() {
	$("#backgroundImageFade").css("opacity", 1.0);
	$("#backgroundImageFade").animate({
		opacity: 0.0
	}, 400);
});

backgroundImage.addEventListener("load", function() {
	let col = colorThief.getPalette(backgroundImage, 6);
	col.unshift(colorThief.getColor(backgroundImage));
	setColor(col);
});

// color[0] should be dominant color
function setColor(color) {
	let mainColor = `rgb(${color[1].toString()}`; //assume
	let minc = -999;
	for (let i = 1; i < color.length; i++) {
		let tmp = contrast(color[0], color[i]);
		if (tmp > minc) {
			minc = tmp;
			mainColor = `rgb(${color[i].toString()}`;
		}
	}

	foregroundColor = mainColor;
	document.documentElement.style.setProperty("--mainColor", mainColor); //highest contrast compared to dominant color
	document.documentElement.style.setProperty("--shadowColor", `rgb(${color[0].toString()}`); //dominant color
}

const rgbToHex = (r, g, b) =>
	"#" + [r, g, b]
	.map((x) => {
		const hex = x.toString(16);
		return hex.length === 1 ? "0" + hex : hex;
	})
	.join("");

function getFileExtension(filename) {
	return filename.slice(((filename.lastIndexOf(".") - 1) >>> 0) + 2);
}

//ref: https://stackoverflow.com/questions/9733288/how-to-programmatically-calculate-the-contrast-ratio-between-two-colors
function luminance(r, g, b) {
	var a = [r, g, b].map(function(v) {
		v /= 255;
		return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
	});
	return a[0] * 0.2126 + a[1] * 0.7152 + a[2] * 0.0722;
}

function contrast(rgb1, rgb2) {
	var lum1 = luminance(rgb1[0], rgb1[1], rgb1[2]);
	var lum2 = luminance(rgb2[0], rgb2[1], rgb2[2]);
	var brightest = Math.max(lum1, lum2);
	var darkest = Math.min(lum1, lum2);
	return (brightest + 0.05) / (darkest + 0.05);
}

// #endregion nowplaying

// #region customisation
function SucrosePropertyListener(name, val) {
	switch (name) {
		case "parallaxToggle":
			_parallax = val.value;
			break;
		case "parallax":
			parallaxVal = val.value;
			break;
		case "sortingMode":
			switch (val.value) {
				case 0:
					sortSoundArray = concentratedSort;
					numDots = 128;
					break;
				case 1:
					sortSoundArray = centeredSort;
					numDots = 256;
					break;
				default:
					sortSoundArray = null;
					numDots = 256;
					break;
			}
			calcDotSize();
			break;
		case "verticalScale":
			verticalScale = val.value;
			break;
		case "dotScale":
			dotScale = val.value;
			calcDotSize();
			break;
	}
}

// #endregion customisation